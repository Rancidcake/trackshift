//! Reinforcement Learning for Network Routing and Handover
//!
//! Implements Q-Learning and Policy Gradient methods to learn optimal:
//! - Path selection (WiFi/Starlink/Multipath/5G)
//! - Handover timing
//! - Parameter tuning (chunk sizes, FEC levels, etc.)
//!
//! The RL agent learns from:
//! - Transfer success/failure
//! - Network performance metrics
//! - User feedback (if available)

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Instant, Duration};
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};

use super::{RouteDecision, NetworkMetricsInput, AiDecision};

/// State representation for RL
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkState {
    /// Discretized RTT (0-9: 0=<50ms, 9=>500ms)
    pub rtt_bucket: u8,
    /// Discretized loss rate (0-9: 0=<0.1%, 9=>10%)
    pub loss_bucket: u8,
    /// Discretized throughput (0-9: 0=<10Mbps, 9=>1000Mbps)
    pub throughput_bucket: u8,
    /// Discretized jitter (0-9: 0=<5ms, 9=>100ms)
    pub jitter_bucket: u8,
    /// Current path (0=WiFi, 1=Starlink, 2=Multipath, 3=5G)
    pub current_path: u8,
    /// Priority level (0=Critical, 1=High, 2=Medium, 3=Bulk)
    pub priority: u8,
}

impl NetworkState {
    /// Create state from network metrics
    pub fn from_metrics(metrics: &NetworkMetricsInput, current_path: RouteDecision, priority: u8) -> Self {
        Self {
            rtt_bucket: Self::discretize_rtt(metrics.rtt_ms),
            loss_bucket: Self::discretize_loss(metrics.loss_rate),
            throughput_bucket: Self::discretize_throughput(metrics.throughput_mbps),
            jitter_bucket: Self::discretize_jitter(metrics.jitter_ms),
            current_path: current_path as u8,
            priority,
        }
    }

    /// Discretize RTT into 10 buckets
    fn discretize_rtt(rtt_ms: f32) -> u8 {
        match rtt_ms {
            x if x < 50.0 => 0,
            x if x < 100.0 => 1,
            x if x < 150.0 => 2,
            x if x < 200.0 => 3,
            x if x < 250.0 => 4,
            x if x < 300.0 => 5,
            x if x < 350.0 => 6,
            x if x < 400.0 => 7,
            x if x < 500.0 => 8,
            _ => 9,
        }
    }

    /// Discretize loss rate into 10 buckets
    fn discretize_loss(loss_rate: f32) -> u8 {
        match loss_rate {
            x if x < 0.001 => 0,  // <0.1%
            x if x < 0.005 => 1,  // 0.1-0.5%
            x if x < 0.01 => 2,   // 0.5-1%
            x if x < 0.02 => 3,   // 1-2%
            x if x < 0.05 => 4,   // 2-5%
            x if x < 0.1 => 5,    // 5-10%
            x if x < 0.15 => 6,   // 10-15%
            x if x < 0.2 => 7,    // 15-20%
            x if x < 0.5 => 8,    // 20-50%
            _ => 9,               // >50%
        }
    }

    /// Discretize throughput into 10 buckets
    fn discretize_throughput(throughput_mbps: f32) -> u8 {
        match throughput_mbps {
            x if x < 10.0 => 0,
            x if x < 25.0 => 1,
            x if x < 50.0 => 2,
            x if x < 100.0 => 2,
            x if x < 200.0 => 3,
            x if x < 300.0 => 4,
            x if x < 400.0 => 5,
            x if x < 500.0 => 6,
            x if x < 750.0 => 7,
            x if x < 1000.0 => 8,
            _ => 9,
        }
    }

    /// Discretize jitter into 10 buckets
    fn discretize_jitter(jitter_ms: f32) -> u8 {
        match jitter_ms {
            x if x < 5.0 => 0,
            x if x < 10.0 => 1,
            x if x < 20.0 => 2,
            x if x < 30.0 => 3,
            x if x < 40.0 => 4,
            x if x < 50.0 => 5,
            x if x < 60.0 => 6,
            x if x < 70.0 => 7,
            x if x < 80.0 => 8,
            _ => 9,
        }
    }

    /// Convert state to feature vector for neural network
    pub fn to_features(&self) -> Vec<f32> {
        vec![
            self.rtt_bucket as f32 / 9.0,
            self.loss_bucket as f32 / 9.0,
            self.throughput_bucket as f32 / 9.0,
            self.jitter_bucket as f32 / 9.0,
            self.current_path as f32 / 3.0,
            self.priority as f32 / 3.0,
        ]
    }
}

/// Action space for RL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RLAction {
    /// Select WiFi path
    SelectWiFi = 0,
    /// Select Starlink path
    SelectStarlink = 1,
    /// Select Multipath
    SelectMultipath = 2,
    /// Select 5G path
    SelectFiveG = 3,
    /// Handover to WiFi
    HandoverToWiFi = 4,
    /// Handover to Starlink
    HandoverToStarlink = 5,
    /// Handover to 5G
    HandoverToFiveG = 6,
    /// Increase FEC redundancy
    IncreaseFec = 7,
    /// Decrease FEC redundancy
    DecreaseFec = 8,
    /// Increase chunk size
    IncreaseChunkSize = 9,
    /// Decrease chunk size
    DecreaseChunkSize = 10,
}

impl RLAction {
    /// Convert to RouteDecision
    pub fn to_route_decision(&self) -> Option<RouteDecision> {
        match self {
            RLAction::SelectWiFi | RLAction::HandoverToWiFi => Some(RouteDecision::WiFi),
            RLAction::SelectStarlink | RLAction::HandoverToStarlink => Some(RouteDecision::Starlink),
            RLAction::SelectMultipath => Some(RouteDecision::Multipath),
            RLAction::SelectFiveG | RLAction::HandoverToFiveG => Some(RouteDecision::FiveG),
            _ => None,
        }
    }

    /// All possible actions
    pub fn all() -> Vec<RLAction> {
        vec![
            RLAction::SelectWiFi,
            RLAction::SelectStarlink,
            RLAction::SelectMultipath,
            RLAction::SelectFiveG,
            RLAction::HandoverToWiFi,
            RLAction::HandoverToStarlink,
            RLAction::HandoverToFiveG,
            RLAction::IncreaseFec,
            RLAction::DecreaseFec,
            RLAction::IncreaseChunkSize,
            RLAction::DecreaseChunkSize,
        ]
    }
}

/// Reward calculation
#[derive(Debug, Clone)]
pub struct Reward {
    /// Base reward from transfer success
    pub base: f32,
    /// Throughput bonus (higher is better)
    pub throughput_bonus: f32,
    /// Latency penalty (lower is better)
    pub latency_penalty: f32,
    /// Loss penalty (lower is better)
    pub loss_penalty: f32,
    /// Total reward
    pub total: f32,
}

impl Reward {
    /// Calculate reward from transfer outcome
    pub fn calculate(
        success: bool,
        throughput_mbps: f32,
        rtt_ms: f32,
        loss_rate: f32,
        transfer_time_secs: f32,
    ) -> Self {
        let base = if success { 100.0 } else { -200.0 };
        
        // Throughput bonus: normalized to 0-50
        let throughput_bonus = (throughput_mbps / 1000.0).min(1.0) * 50.0;
        
        // Latency penalty: lower RTT is better
        let latency_penalty = -(rtt_ms / 500.0).min(1.0) * 30.0;
        
        // Loss penalty: lower loss is better
        let loss_penalty = -(loss_rate * 10.0).min(1.0) * 20.0;
        
        let total = base + throughput_bonus + latency_penalty + loss_penalty;
        
        Self {
            base,
            throughput_bonus,
            latency_penalty,
            loss_penalty,
            total,
        }
    }
}

/// Q-Learning agent for path selection
pub struct QLearningAgent {
    /// Q-table: state -> action -> Q-value
    q_table: Arc<RwLock<HashMap<(NetworkState, RLAction), f32>>>,
    /// Learning rate (alpha)
    learning_rate: f32,
    /// Discount factor (gamma)
    discount_factor: f32,
    /// Exploration rate (epsilon)
    exploration_rate: Arc<RwLock<f32>>,
    /// Minimum exploration rate
    min_exploration: f32,
    /// Exploration decay rate
    exploration_decay: f32,
    /// Statistics
    stats: Arc<RwLock<RLStats>>,
}

/// RL Statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RLStats {
    pub total_episodes: u64,
    pub total_rewards: f32,
    pub average_reward: f32,
    pub best_reward: f32,
    pub exploration_rate: f32,
    pub q_table_size: usize,
    pub successful_decisions: u64,
    pub failed_decisions: u64,
}

impl QLearningAgent {
    /// Create new Q-Learning agent
    pub fn new(
        learning_rate: f32,
        discount_factor: f32,
        initial_exploration: f32,
        min_exploration: f32,
        exploration_decay: f32,
    ) -> Self {
        Self {
            q_table: Arc::new(RwLock::new(HashMap::new())),
            learning_rate,
            discount_factor,
            exploration_rate: Arc::new(RwLock::new(initial_exploration)),
            min_exploration,
            exploration_decay,
            stats: Arc::new(RwLock::new(RLStats::default())),
        }
    }

    /// Get Q-value for state-action pair
    fn get_q_value(&self, state: &NetworkState, action: RLAction) -> f32 {
        let q_table = self.q_table.read();
        *q_table.get(&(state.clone(), action)).unwrap_or(&0.0)
    }

    /// Update Q-value using Q-Learning update rule
    fn update_q_value(&self, state: &NetworkState, action: RLAction, reward: f32, next_state: &NetworkState) {
        let current_q = self.get_q_value(state, action);
        
        // Find max Q-value for next state
        let max_next_q = RLAction::all()
            .iter()
            .map(|a| self.get_q_value(next_state, *a))
            .fold(0.0, f32::max);
        
        // Q-Learning update: Q(s,a) = Q(s,a) + α[r + γ*max(Q(s',a')) - Q(s,a)]
        let new_q = current_q + self.learning_rate * (reward + self.discount_factor * max_next_q - current_q);
        
        let mut q_table = self.q_table.write();
        q_table.insert((state.clone(), action), new_q);
    }

    /// Select action using epsilon-greedy policy
    pub fn select_action(&self, state: &NetworkState) -> RLAction {
        let exploration_rate = *self.exploration_rate.read();
        
        // Exploration: random action
        let random_val = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() % 10000) as f32 / 10000.0;
        
        if random_val < exploration_rate {
            let actions = RLAction::all();
            let idx = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() % actions.len() as u128) as usize;
            return actions[idx];
        }
        
        // Exploitation: best action according to Q-table
        let actions = RLAction::all();
        let mut best_action = actions[0];
        let mut best_q = self.get_q_value(state, best_action);
        
        for action in actions.iter().skip(1) {
            let q_value = self.get_q_value(state, *action);
            if q_value > best_q {
                best_q = q_value;
                best_action = *action;
            }
        }
        
        best_action
    }

    /// Learn from experience (state, action, reward, next_state)
    pub fn learn(&self, state: &NetworkState, action: RLAction, reward: f32, next_state: &NetworkState) {
        // Update Q-value
        self.update_q_value(state, action, reward, next_state);
        
        // Decay exploration rate
        let mut exploration = self.exploration_rate.write();
        *exploration = (*exploration * self.exploration_decay).max(self.min_exploration);
        
        // Update statistics
        let mut stats = self.stats.write();
        stats.total_episodes += 1;
        stats.total_rewards += reward;
        stats.average_reward = stats.total_rewards / stats.total_episodes as f32;
        stats.best_reward = stats.best_reward.max(reward);
        stats.exploration_rate = *exploration;
        stats.q_table_size = self.q_table.read().len();
        
        if reward > 0.0 {
            stats.successful_decisions += 1;
        } else {
            stats.failed_decisions += 1;
        }
    }

    /// Get statistics
    pub fn get_stats(&self) -> RLStats {
        let mut stats = self.stats.write();
        stats.q_table_size = self.q_table.read().len();
        stats.exploration_rate = *self.exploration_rate.read();
        stats.clone()
    }

    /// Save Q-table to disk (for persistence)
    pub fn save(&self, path: &str) -> Result<()> {
        let q_table = self.q_table.read();
        let data = serde_json::to_string_pretty(&*q_table)
            .context("Failed to serialize Q-table")?;
        std::fs::write(path, data)
            .context("Failed to write Q-table to disk")?;
        Ok(())
    }

    /// Load Q-table from disk
    pub fn load(&self, path: &str) -> Result<()> {
        let data = std::fs::read_to_string(path)
            .context("Failed to read Q-table from disk")?;
        let q_table: HashMap<(NetworkState, RLAction), f32> = serde_json::from_str(&data)
            .context("Failed to deserialize Q-table")?;
        *self.q_table.write() = q_table;
        Ok(())
    }
}

/// Policy Gradient agent for continuous parameter tuning
pub struct PolicyGradientAgent {
    /// Policy parameters (weights for state features)
    policy_weights: Arc<RwLock<Vec<f32>>>,
    /// Learning rate
    learning_rate: f32,
    /// Statistics
    stats: Arc<RwLock<RLStats>>,
}

impl PolicyGradientAgent {
    /// Create new policy gradient agent
    pub fn new(learning_rate: f32) -> Self {
        // Initialize with 6 weights (one per state feature)
        let policy_weights = vec![0.0; 6];
        
        Self {
            policy_weights: Arc::new(RwLock::new(policy_weights)),
            learning_rate,
            stats: Arc::new(RwLock::new(RLStats::default())),
        }
    }

    /// Select action using policy (softmax)
    pub fn select_action(&self, state: &NetworkState) -> RLAction {
        let features = state.to_features();
        let weights = self.policy_weights.read();
        
        // Compute action scores
        let mut scores = Vec::new();
        for action in RLAction::all() {
            let mut score = 0.0;
            for (i, &feature) in features.iter().enumerate() {
                // Simple linear policy (can be enhanced with neural network)
                score += weights[i] * feature;
            }
            scores.push(score);
        }
        
        // Softmax to get probabilities
        let max_score: f32 = scores.iter().fold(0.0_f32, |a, &b| a.max(b));
        let exp_scores: Vec<f32> = scores.iter().map(|s| (s - max_score).exp()).collect();
        let sum: f32 = exp_scores.iter().sum();
        let probabilities: Vec<f32> = exp_scores.iter().map(|p| p / sum).collect();
        
        // Sample from distribution
        let random_val = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() % 10000) as f32 / 10000.0;
        
        let mut rng = random_val;
        for (i, &prob) in probabilities.iter().enumerate() {
            rng -= prob;
            if rng <= 0.0 {
                return RLAction::all()[i];
            }
        }
        
        RLAction::all()[0] // Fallback
    }

    /// Update policy using REINFORCE algorithm
    pub fn learn(&self, state: &NetworkState, action: RLAction, reward: f32) {
        // Simplified REINFORCE update
        // In practice, you'd accumulate gradients over an episode
        let features = state.to_features();
        let mut weights = self.policy_weights.write();
        
        // Update weights based on reward
        for (i, &feature) in features.iter().enumerate() {
            weights[i] += self.learning_rate * reward * feature;
        }
        
        // Update statistics
        let mut stats = self.stats.write();
        stats.total_episodes += 1;
        stats.total_rewards += reward;
        stats.average_reward = stats.total_rewards / stats.total_episodes as f32;
    }

    /// Get statistics
    pub fn get_stats(&self) -> RLStats {
        self.stats.read().clone()
    }
}

/// Reinforcement Learning Manager
/// Combines Q-Learning and Policy Gradient for comprehensive learning
pub struct RLManager {
    /// Q-Learning agent for discrete actions (path selection)
    q_agent: Arc<QLearningAgent>,
    /// Policy Gradient agent for continuous parameters
    policy_agent: Arc<PolicyGradientAgent>,
    /// Current state
    current_state: Arc<RwLock<Option<NetworkState>>>,
    /// Last action taken
    last_action: Arc<RwLock<Option<RLAction>>>,
    /// Episode start time
    episode_start: Arc<RwLock<Option<Instant>>>,
}

impl RLManager {
    /// Create new RL manager
    pub fn new() -> Self {
        Self {
            q_agent: Arc::new(QLearningAgent::new(
                0.1,   // learning_rate
                0.9,   // discount_factor
                0.3,   // initial_exploration
                0.01,  // min_exploration
                0.995, // exploration_decay
            )),
            policy_agent: Arc::new(PolicyGradientAgent::new(0.01)),
            current_state: Arc::new(RwLock::new(None)),
            last_action: Arc::new(RwLock::new(None)),
            episode_start: Arc::new(RwLock::new(None)),
        }
    }

    /// Get recommended action based on current state
    pub fn recommend_action(&self, state: &NetworkState) -> RLAction {
        // Use Q-Learning for path selection actions
        let path_actions = vec![
            RLAction::SelectWiFi,
            RLAction::SelectStarlink,
            RLAction::SelectMultipath,
            RLAction::SelectFiveG,
        ];
        
        // Check if action is a path selection
        let action = self.q_agent.select_action(state);
        if path_actions.contains(&action) {
            action
        } else {
            // Use policy gradient for other actions
            self.policy_agent.select_action(state)
        }
    }

    /// Start new episode (transfer or decision sequence)
    pub fn start_episode(&self, initial_state: NetworkState) {
        *self.current_state.write() = Some(initial_state.clone());
        *self.episode_start.write() = Some(Instant::now());
    }

    /// Record action taken
    pub fn record_action(&self, action: RLAction) {
        *self.last_action.write() = Some(action);
    }

    /// End episode with reward
    pub fn end_episode(
        &self,
        final_state: NetworkState,
        reward: f32,
        _metrics: &NetworkMetricsInput,
    ) {
        let current_state = self.current_state.read().clone();
        let last_action = self.last_action.read().clone();
        
        if let (Some(state), Some(action)) = (current_state, last_action) {
            // Learn from experience
            self.q_agent.learn(&state, action, reward, &final_state);
            self.policy_agent.learn(&state, action, reward);
        }
        
        // Reset for next episode
        *self.current_state.write() = None;
        *self.last_action.write() = None;
        *self.episode_start.write() = None;
    }

    /// Get enhanced decision with RL recommendations
    pub fn enhance_decision(
        &self,
        base_decision: &AiDecision,
        metrics: &NetworkMetricsInput,
        priority: u8,
    ) -> AiDecision {
        let state = NetworkState::from_metrics(metrics, base_decision.route, priority);
        let rl_action = self.recommend_action(&state);
        
        let mut enhanced = base_decision.clone();
        
        // Apply RL recommendations
        if let Some(route) = rl_action.to_route_decision() {
            enhanced.route = route;
        }
        
        // Adjust parameters based on RL action
        match rl_action {
            RLAction::IncreaseFec => {
                // Increase FEC redundancy (would need to modify decision structure)
            }
            RLAction::DecreaseFec => {
                // Decrease FEC redundancy
            }
            RLAction::IncreaseChunkSize => {
                enhanced.recommended_chunk_size = (enhanced.recommended_chunk_size as f32 * 1.2) as u32;
            }
            RLAction::DecreaseChunkSize => {
                enhanced.recommended_chunk_size = (enhanced.recommended_chunk_size as f32 * 0.8) as u32;
            }
            _ => {}
        }
        
        enhanced
    }

    /// Learn from experience (delegates to Q-agent)
    pub fn learn(&self, state: &NetworkState, action: RLAction, reward: f32, next_state: &NetworkState) {
        self.q_agent.learn(state, action, reward, next_state);
    }

    /// Get statistics from both agents
    pub fn get_stats(&self) -> (RLStats, RLStats) {
        (self.q_agent.get_stats(), self.policy_agent.get_stats())
    }

    /// Save learned models
    pub fn save(&self, q_table_path: &str) -> Result<()> {
        self.q_agent.save(q_table_path)?;
        // Policy weights could also be saved here
        Ok(())
    }

    /// Load learned models
    pub fn load(&self, q_table_path: &str) -> Result<()> {
        self.q_agent.load(q_table_path)?;
        // Policy weights could also be loaded here
        Ok(())
    }
}

impl Default for RLManager {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_state_discretization() {
        let metrics = NetworkMetricsInput {
            rtt_ms: 150.0,
            loss_rate: 0.02,
            throughput_mbps: 200.0,
            jitter_ms: 15.0,
            ..Default::default()
        };
        
        let state = NetworkState::from_metrics(&metrics, RouteDecision::WiFi, 0);
        assert_eq!(state.rtt_bucket, 2);
        assert_eq!(state.loss_bucket, 3);
        assert_eq!(state.throughput_bucket, 3);
    }

    #[test]
    fn test_q_learning_update() {
        let agent = QLearningAgent::new(0.1, 0.9, 0.3, 0.01, 0.995);
        
        let state1 = NetworkState {
            rtt_bucket: 1,
            loss_bucket: 1,
            throughput_bucket: 5,
            jitter_bucket: 1,
            current_path: 0,
            priority: 0,
        };
        
        let state2 = NetworkState {
            rtt_bucket: 2,
            loss_bucket: 2,
            throughput_bucket: 4,
            jitter_bucket: 2,
            current_path: 0,
            priority: 0,
        };
        
        let action = RLAction::SelectWiFi;
        let reward = 50.0;
        
        agent.learn(&state1, action, reward, &state2);
        
        let q_value = agent.get_q_value(&state1, action);
        assert!(q_value > 0.0);
    }

    #[test]
    fn test_reward_calculation() {
        let reward = Reward::calculate(true, 100.0, 50.0, 0.01, 10.0);
        assert!(reward.total > 0.0);
        
        let bad_reward = Reward::calculate(false, 10.0, 500.0, 0.1, 100.0);
        assert!(bad_reward.total < 0.0);
    }
}

