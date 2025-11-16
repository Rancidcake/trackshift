/// Network quality assessment and adaptive strategies for patchy networks
use super::NetworkMetricsInput;

/// Network quality score (0.0 = terrible, 1.0 = excellent)
#[derive(Debug, Clone, Copy)]
pub struct NetworkQuality {
    pub score: f32,
    pub is_patchy: bool,
    pub is_connected: bool,
    pub recommended_action: NetworkAction,
}

/// Recommended actions based on network quality
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkAction {
    Normal,        // Normal operation
    Aggressive,    // Aggressive optimization (bad network)
    Conservative,  // Conservative (good network, send more)
    Emergency,     // Emergency mode (very bad network)
}

impl NetworkQuality {
    /// Assess network quality from metrics
    pub fn assess(metrics: &NetworkMetricsInput) -> Self {
        let mut score: f32 = 1.0;
        let mut issues = 0;
        
        // Factor 1: Packet Loss (major impact)
        if metrics.loss_rate > 0.1 {
            score -= 0.4;  // High loss = bad
            issues += 1;
        } else if metrics.loss_rate > 0.05 {
            score -= 0.2;
            issues += 1;
        } else if metrics.loss_rate > 0.01 {
            score -= 0.1;
        }
        
        // Factor 2: High RTT (latency)
        if metrics.rtt_ms > 500.0 {
            score -= 0.3;  // Very high latency
            issues += 1;
        } else if metrics.rtt_ms > 200.0 {
            score -= 0.15;
            issues += 1;
        } else if metrics.rtt_ms > 100.0 {
            score -= 0.05;
        }
        
        // Factor 3: High Jitter (unstable)
        if metrics.jitter_ms > 50.0 {
            score -= 0.2;
            issues += 1;
        } else if metrics.jitter_ms > 20.0 {
            score -= 0.1;
        }
        
        // Factor 4: Low Throughput
        if metrics.throughput_mbps < 1.0 {
            score -= 0.3;  // Very slow
            issues += 1;
        } else if metrics.throughput_mbps < 5.0 {
            score -= 0.15;
        }
        
        // Factor 5: High Retransmissions
        if metrics.retransmissions > 10.0 {
            score -= 0.2;
            issues += 1;
        } else if metrics.retransmissions > 5.0 {
            score -= 0.1;
        }
        
        // Factor 6: Poor Signal Strength (WiFi only)
        if metrics.wifi_signal < -90.0 {
            score -= 0.3;  // Very weak signal
            issues += 1;
        } else if metrics.wifi_signal < -80.0 {
            score -= 0.15;
        }
        
        // Factor 7: Session Break
        if metrics.session_state > 0.5 {
            score -= 0.4;  // Connection broken
            issues += 1;
        }
        
        // Ensure score is in valid range
        score = score.max(0.0_f32).min(1.0_f32);
        
        // Determine if patchy
        let is_patchy = score < 0.6 || issues >= 2;
        let is_connected = metrics.session_state < 0.5 && score > 0.3;
        
        // Determine recommended action
        let recommended_action = if score < 0.3 {
            NetworkAction::Emergency
        } else if score < 0.6 {
            NetworkAction::Aggressive
        } else if score > 0.9 {
            NetworkAction::Conservative
        } else {
            NetworkAction::Normal
        };
        
        Self {
            score,
            is_patchy,
            is_connected,
            recommended_action,
        }
    }
    
    /// Get adaptive redundancy threshold based on network quality
    /// Bad network = lower threshold (more aggressive redundancy detection)
    pub fn adaptive_redundancy_threshold(&self) -> f32 {
        match self.recommended_action {
            NetworkAction::Emergency => 0.85,   // Very aggressive (skip more)
            NetworkAction::Aggressive => 0.90,  // Aggressive (skip similar)
            NetworkAction::Normal => 0.95,      // Normal threshold
            NetworkAction::Conservative => 0.98, // Conservative (send more)
        }
    }
    
    /// Should use compression for patchy networks
    pub fn should_compress(&self) -> bool {
        self.is_patchy || self.score < 0.7
    }
    
    /// Should prioritize critical data only
    pub fn prioritize_critical_only(&self) -> bool {
        self.score < 0.5
    }
}

