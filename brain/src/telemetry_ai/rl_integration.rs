//! RL Integration Helpers
//!
//! Provides helper functions to integrate RL learning into the telemetry system

use super::{RLManager, NetworkState, RLAction, Reward, NetworkMetricsInput, RouteDecision};
use std::sync::Arc;
use std::time::Instant;
use anyhow::Result;

/// Track transfer episode for RL learning
pub struct TransferEpisode {
    start_state: NetworkState,
    start_time: Instant,
    actions_taken: Vec<(RLAction, Instant)>,
    final_metrics: Option<NetworkMetricsInput>,
    success: bool,
}

impl TransferEpisode {
    /// Create new transfer episode
    pub fn new(
        initial_metrics: &NetworkMetricsInput,
        current_path: RouteDecision,
        priority: u8,
    ) -> Self {
        let start_state = NetworkState::from_metrics(initial_metrics, current_path, priority);
        
        Self {
            start_state,
            start_time: Instant::now(),
            actions_taken: Vec::new(),
            final_metrics: None,
            success: false,
        }
    }

    /// Record action taken during episode
    pub fn record_action(&mut self, action: RLAction) {
        self.actions_taken.push((action, Instant::now()));
    }

    /// End episode with outcome
    pub fn end(
        &mut self,
        final_metrics: NetworkMetricsInput,
        success: bool,
    ) {
        self.final_metrics = Some(final_metrics);
        self.success = success;
    }

    /// Calculate reward and learn from episode
    pub fn learn(&self, rl_manager: &RLManager) -> Result<()> {
        let final_metrics = self.final_metrics.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Episode not ended"))?;
        
        let duration = self.start_time.elapsed().as_secs_f32();
        
        // Calculate reward
        let reward = Reward::calculate(
            self.success,
            final_metrics.throughput_mbps,
            final_metrics.rtt_ms,
            final_metrics.loss_rate,
            duration,
        );
        
        // Create final state
        let final_state = NetworkState::from_metrics(
            final_metrics,
            // Use the last action's route, or default to WiFi
            self.actions_taken.last()
                .and_then(|(action, _)| action.to_route_decision())
                .unwrap_or(RouteDecision::WiFi),
            0, // Priority doesn't change
        );
        
        // Learn from each action in sequence
        let mut current_state = self.start_state.clone();
        for (action, _) in &self.actions_taken {
            // Use intermediate reward (can be improved with TD learning)
            let intermediate_reward = if self.success {
                reward.total / self.actions_taken.len() as f32
            } else {
                reward.total / self.actions_taken.len() as f32
            };
            
            // Update state based on action
            if let Some(route) = action.to_route_decision() {
                current_state.current_path = route as u8;
            }
            
            // Learn (Q-Learning update)
            rl_manager.learn(
                &current_state,
                *action,
                intermediate_reward,
                &final_state,
            );
        }
        
        Ok(())
    }
}

/// Helper to record transfer outcomes for RL
pub struct RLRecorder {
    rl_manager: Arc<RLManager>,
    active_episodes: std::collections::HashMap<String, TransferEpisode>,
}

impl RLRecorder {
    /// Create new RL recorder
    pub fn new(rl_manager: Arc<RLManager>) -> Self {
        Self {
            rl_manager,
            active_episodes: std::collections::HashMap::new(),
        }
    }

    /// Start tracking a transfer
    pub fn start_transfer(
        &mut self,
        transfer_id: &str,
        metrics: &NetworkMetricsInput,
        current_path: RouteDecision,
        priority: u8,
    ) {
        let episode = TransferEpisode::new(metrics, current_path, priority);
        self.active_episodes.insert(transfer_id.to_string(), episode);
    }

    /// Record action during transfer
    pub fn record_action(&mut self, transfer_id: &str, action: RLAction) {
        if let Some(episode) = self.active_episodes.get_mut(transfer_id) {
            episode.record_action(action);
        }
    }

    /// End transfer and learn
    pub fn end_transfer(
        &mut self,
        transfer_id: &str,
        final_metrics: NetworkMetricsInput,
        success: bool,
    ) -> Result<()> {
        if let Some(mut episode) = self.active_episodes.remove(transfer_id) {
            episode.end(final_metrics, success);
            episode.learn(&self.rl_manager)?;
        }
        Ok(())
    }
}

