//! Fallback System for Experimental Features
//!
//! Provides graceful degradation when experimental features fail:
//! - QUIC → TCP fallback
//! - Multipath → Single path fallback
//! - FEC → No FEC fallback
//! - Compression → No compression fallback

use anyhow::Result;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Instant, Duration};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::handover::NetworkPath;

/// Fallback strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackStrategy {
    /// No fallback (fail fast)
    None,
    /// Automatic fallback on any failure
    Automatic,
    /// Fallback only on critical failures
    Conservative,
    /// Aggressive fallback (try alternatives quickly)
    Aggressive,
}

/// System state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemState {
    /// All features enabled (experimental)
    FullExperimental,
    /// QUIC with FEC, no multipath
    QuicWithFec,
    /// QUIC without FEC
    QuicBasic,
    /// TCP fallback
    TcpFallback,
    /// Complete fallback (minimal features)
    MinimalFallback,
}

/// Fallback reason
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FallbackReason {
    ConnectionFailure,
    FecFailure,
    MultipathFailure,
    HandoverFailure,
    Timeout,
    ErrorRateTooHigh,
    Manual,
}

/// Fallback event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackEvent {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub from_state: SystemState,
    pub to_state: SystemState,
    pub reason: FallbackReason,
    pub error_message: Option<String>,
}

/// Fallback manager
pub struct FallbackManager {
    current_state: Arc<RwLock<SystemState>>,
    strategy: Arc<RwLock<FallbackStrategy>>,
    fallback_history: Arc<RwLock<Vec<FallbackEvent>>>,
    failure_count: Arc<RwLock<HashMap<SystemState, u32>>>,
    last_failure_time: Arc<RwLock<HashMap<SystemState, Instant>>>,
    cooldown_period: Duration,
}

impl FallbackManager {
    /// Create new fallback manager
    pub fn new(strategy: FallbackStrategy) -> Self {
        Self {
            current_state: Arc::new(RwLock::new(SystemState::FullExperimental)),
            strategy: Arc::new(RwLock::new(strategy)),
            fallback_history: Arc::new(RwLock::new(Vec::new())),
            failure_count: Arc::new(RwLock::new(HashMap::new())),
            last_failure_time: Arc::new(RwLock::new(HashMap::new())),
            cooldown_period: Duration::from_secs(60), // 1 minute cooldown
        }
    }

    /// Get current system state
    pub fn current_state(&self) -> SystemState {
        *self.current_state.read()
    }

    /// Get fallback strategy
    pub fn strategy(&self) -> FallbackStrategy {
        *self.strategy.read()
    }

    /// Set fallback strategy
    pub fn set_strategy(&self, strategy: FallbackStrategy) {
        *self.strategy.write() = strategy;
    }

    /// Report a failure and trigger fallback if needed
    pub fn report_failure(
        &self,
        reason: FallbackReason,
        error_message: Option<String>,
    ) -> Result<Option<SystemState>> {
        let strategy = self.strategy();
        
        if strategy == FallbackStrategy::None {
            return Ok(None);
        }

        let current = self.current_state();
        
        // Increment failure count
        {
            let mut failures = self.failure_count.write();
            *failures.entry(current).or_insert(0) += 1;
        }

        // Record failure time
        {
            let mut failure_times = self.last_failure_time.write();
            failure_times.insert(current, Instant::now());
        }

        // Determine if fallback is needed
        let should_fallback = match strategy {
            FallbackStrategy::Automatic => true,
            FallbackStrategy::Conservative => {
                // Only fallback on critical failures
                matches!(
                    reason,
                    FallbackReason::ConnectionFailure | FallbackReason::ErrorRateTooHigh
                )
            }
            FallbackStrategy::Aggressive => {
                // Fallback quickly on any failure
                let failures = self.failure_count.read();
                let count = failures.get(&current).copied().unwrap_or(0);
                count >= 2 // Fallback after 2 failures
            }
            FallbackStrategy::None => false,
        };

        if should_fallback {
            let next_state = self.determine_next_state(current, &reason)?;
            
            if next_state != current {
                self.perform_fallback(current, next_state, reason, error_message)?;
                return Ok(Some(next_state));
            }
        }

        Ok(None)
    }

    /// Determine next system state based on current state and failure reason
    fn determine_next_state(
        &self,
        current: SystemState,
        reason: &FallbackReason,
    ) -> Result<SystemState> {
        match (current, reason) {
            // From FullExperimental
            (SystemState::FullExperimental, FallbackReason::MultipathFailure) => {
                Ok(SystemState::QuicWithFec) // Disable multipath, keep FEC
            }
            (SystemState::FullExperimental, FallbackReason::FecFailure) => {
                Ok(SystemState::QuicBasic) // Disable FEC, keep QUIC
            }
            (SystemState::FullExperimental, FallbackReason::ConnectionFailure) => {
                Ok(SystemState::QuicBasic) // Try basic QUIC first
            }
            (SystemState::FullExperimental, FallbackReason::HandoverFailure) => {
                Ok(SystemState::QuicWithFec) // Disable handover
            }

            // From QuicWithFec
            (SystemState::QuicWithFec, FallbackReason::FecFailure) => {
                Ok(SystemState::QuicBasic) // Disable FEC
            }
            (SystemState::QuicWithFec, FallbackReason::ConnectionFailure) => {
                Ok(SystemState::TcpFallback) // Fallback to TCP
            }

            // From QuicBasic
            (SystemState::QuicBasic, FallbackReason::ConnectionFailure) => {
                Ok(SystemState::TcpFallback) // Fallback to TCP
            }

            // From TcpFallback
            (SystemState::TcpFallback, FallbackReason::ConnectionFailure) => {
                Ok(SystemState::MinimalFallback) // Last resort
            }

            // Manual fallback
            (_, FallbackReason::Manual) => {
                Ok(self.get_next_available_state(current))
            }

            // Default: no change
            _ => Ok(current),
        }
    }

    /// Get next available state (for manual fallback)
    fn get_next_available_state(&self, current: SystemState) -> SystemState {
        match current {
            SystemState::FullExperimental => SystemState::QuicWithFec,
            SystemState::QuicWithFec => SystemState::QuicBasic,
            SystemState::QuicBasic => SystemState::TcpFallback,
            SystemState::TcpFallback => SystemState::MinimalFallback,
            SystemState::MinimalFallback => SystemState::MinimalFallback, // Can't go lower
        }
    }

    /// Perform fallback to new state
    fn perform_fallback(
        &self,
        from: SystemState,
        to: SystemState,
        reason: FallbackReason,
        error_message: Option<String>,
    ) -> Result<()> {
        println!("⚠️  Fallback: {:?} → {:?} (Reason: {:?})", from, to, reason);

        // Record fallback event
        let event = FallbackEvent {
            timestamp: Utc::now(),
            from_state: from,
            to_state: to,
            reason: reason.clone(),
            error_message,
        };

        self.fallback_history.write().push(event);

        // Update current state
        *self.current_state.write() = to;

        // Reset failure count for new state
        self.failure_count.write().remove(&from);

        Ok(())
    }

    /// Get configuration for current state
    pub fn get_config_for_state(&self, state: SystemState) -> FallbackConfig {
        match state {
            SystemState::FullExperimental => FallbackConfig {
                enable_quic: true,
                enable_fec: true,
                enable_multipath: true,
                enable_handover: true,
                enable_compression: true,
                enable_encryption: true,
                use_tcp_fallback: false,
            },
            SystemState::QuicWithFec => FallbackConfig {
                enable_quic: true,
                enable_fec: true,
                enable_multipath: false,
                enable_handover: false,
                enable_compression: true,
                enable_encryption: true,
                use_tcp_fallback: false,
            },
            SystemState::QuicBasic => FallbackConfig {
                enable_quic: true,
                enable_fec: false,
                enable_multipath: false,
                enable_handover: false,
                enable_compression: true,
                enable_encryption: true,
                use_tcp_fallback: false,
            },
            SystemState::TcpFallback => FallbackConfig {
                enable_quic: false,
                enable_fec: false,
                enable_multipath: false,
                enable_handover: false,
                enable_compression: true,
                enable_encryption: true,
                use_tcp_fallback: true,
            },
            SystemState::MinimalFallback => FallbackConfig {
                enable_quic: false,
                enable_fec: false,
                enable_multipath: false,
                enable_handover: false,
                enable_compression: false,
                enable_encryption: true, // Keep encryption even in minimal
                use_tcp_fallback: true,
            },
        }
    }

    /// Get current configuration
    pub fn get_current_config(&self) -> FallbackConfig {
        self.get_config_for_state(self.current_state())
    }

    /// Manually trigger fallback
    pub fn manual_fallback(&self, target_state: Option<SystemState>) -> Result<SystemState> {
        let current = self.current_state();
        let target = target_state.unwrap_or_else(|| self.get_next_available_state(current));

        if target != current {
            self.perform_fallback(
                current,
                target,
                FallbackReason::Manual,
                Some("Manual fallback requested".to_string()),
            )?;
        }

        Ok(target)
    }

    /// Try to recover to higher state (upgrade)
    pub fn try_recover(&self) -> Result<Option<SystemState>> {
        let current = self.current_state();
        
        // Check if we can upgrade
        let upgraded = match current {
            SystemState::MinimalFallback => SystemState::TcpFallback,
            SystemState::TcpFallback => SystemState::QuicBasic,
            SystemState::QuicBasic => SystemState::QuicWithFec,
            SystemState::QuicWithFec => SystemState::FullExperimental,
            SystemState::FullExperimental => return Ok(None), // Already at max
        };

        // Check cooldown period
        let last_failure = self.last_failure_time.read()
            .get(&upgraded)
            .copied();

        if let Some(last) = last_failure {
            if last.elapsed() < self.cooldown_period {
                return Ok(None); // Still in cooldown
            }
        }

        // Try to upgrade
        self.perform_fallback(
            current,
            upgraded,
            FallbackReason::Manual,
            Some("Recovery attempt".to_string()),
        )?;

        Ok(Some(upgraded))
    }

    /// Get fallback statistics
    pub fn get_stats(&self) -> FallbackStats {
        let history = self.fallback_history.read();
        let failures = self.failure_count.read();

        FallbackStats {
            current_state: self.current_state(),
            strategy: self.strategy(),
            total_fallbacks: history.len(),
            failure_counts: failures.clone(),
            recent_fallbacks: history.iter().rev().take(10).cloned().collect(),
        }
    }

    /// Get fallback history
    pub fn get_history(&self) -> Vec<FallbackEvent> {
        self.fallback_history.read().clone()
    }
}

/// Fallback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    pub enable_quic: bool,
    pub enable_fec: bool,
    pub enable_multipath: bool,
    pub enable_handover: bool,
    pub enable_compression: bool,
    pub enable_encryption: bool,
    pub use_tcp_fallback: bool,
}

/// Fallback statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackStats {
    pub current_state: SystemState,
    pub strategy: FallbackStrategy,
    pub total_fallbacks: usize,
    pub failure_counts: HashMap<SystemState, u32>,
    pub recent_fallbacks: Vec<FallbackEvent>,
}

use std::collections::HashMap;

