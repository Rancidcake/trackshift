//! User Control Dashboard
//!
//! Provides user controls for:
//! - System configuration
//! - Transfer management
//! - Fallback system control
//! - Network settings
//! - Performance tuning

use serde::{Serialize, Deserialize};
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::Instant;

use quic_fec::{
    FallbackManager, FallbackStrategy, SystemState, FallbackConfig,
    FallbackEvent,
};
use quic_fec::scheduler::PacketPriority;
use quic_fec::handover::{NetworkPath, HandoverStrategy};

/// Dashboard control interface
pub struct DashboardController {
    fallback_manager: Arc<FallbackManager>,
    system_config: Arc<RwLock<SystemConfig>>,
    transfer_controls: Arc<RwLock<HashMap<String, TransferControl>>>,
    network_settings: Arc<RwLock<NetworkSettings>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
}

/// System configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub max_concurrent_transfers: usize,
    pub default_priority: PacketPriority,
    pub default_chunk_size: usize,
    pub enable_ai_routing: bool,
    pub enable_compression: bool,
    pub compression_algorithm: CompressionAlgorithm,
    pub enable_encryption: bool,
}

/// Compression algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    None,
    Lz4,
    Zstd,
}

/// Transfer control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferControl {
    pub transfer_id: String,
    pub can_pause: bool,
    pub can_resume: bool,
    pub can_cancel: bool,
    pub can_change_priority: bool,
    pub current_priority: PacketPriority,
}

/// Network settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub preferred_path: NetworkPath,
    pub handover_strategy: HandoverStrategy,
    pub enable_multipath: bool,
    pub max_rtt_threshold: f32,
    pub max_loss_threshold: f32,
    pub bandwidth_limit: Option<u64>, // bytes/sec
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_transfers: u64,
    pub active_transfers: usize,
    pub completed_transfers: u64,
    pub failed_transfers: u64,
    pub average_speed_mbps: f32,
    pub peak_speed_mbps: f32,
    pub total_bytes_transferred: u64,
    pub network_efficiency: f32, // 0.0 - 1.0
    pub fec_recovery_rate: f32,
    pub handover_count: u64,
    pub last_updated: Instant,
}

impl DashboardController {
    /// Create new dashboard controller
    pub fn new(fallback_manager: Arc<FallbackManager>) -> Self {
        Self {
            fallback_manager,
            system_config: Arc::new(RwLock::new(SystemConfig::default())),
            transfer_controls: Arc::new(RwLock::new(HashMap::new())),
            network_settings: Arc::new(RwLock::new(NetworkSettings::default())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        }
    }

    /// Get complete dashboard state
    pub fn get_dashboard_state(&self) -> DashboardState {
        let fallback_stats = self.fallback_manager.get_stats();
        let fallback_config = self.fallback_manager.get_current_config();

        DashboardState {
            system_state: fallback_stats.current_state,
            fallback_strategy: fallback_stats.strategy,
            fallback_config,
            system_config: self.system_config.read().clone(),
            network_settings: self.network_settings.read().clone(),
            performance_metrics: self.performance_metrics.read().clone(),
            active_transfers: self.transfer_controls.read().values().cloned().collect(),
            fallback_history: self.fallback_manager.get_history(),
        }
    }

    /// Update system configuration
    pub fn update_system_config(&self, config: SystemConfig) {
        *self.system_config.write() = config;
    }

    /// Update network settings
    pub fn update_network_settings(&self, settings: NetworkSettings) {
        *self.network_settings.write() = settings;
    }

    /// Set fallback strategy
    pub fn set_fallback_strategy(&self, strategy: FallbackStrategy) {
        self.fallback_manager.set_strategy(strategy);
    }

    /// Manual fallback
    pub fn trigger_fallback(&self, target_state: Option<SystemState>) -> Result<SystemState> {
        self.fallback_manager.manual_fallback(target_state)
    }

    /// Try to recover (upgrade system state)
    pub fn try_recover(&self) -> Result<Option<SystemState>> {
        self.fallback_manager.try_recover()
    }

    /// Register transfer for control
    pub fn register_transfer(&self, transfer_id: String, control: TransferControl) {
        self.transfer_controls.write().insert(transfer_id, control);
    }

    /// Update transfer control
    pub fn update_transfer_control(&self, transfer_id: &str, control: TransferControl) {
        self.transfer_controls.write().insert(transfer_id.to_string(), control);
    }

    /// Remove transfer control
    pub fn remove_transfer(&self, transfer_id: &str) {
        self.transfer_controls.write().remove(transfer_id);
    }

    /// Update performance metrics
    pub fn update_metrics(&self, metrics: PerformanceMetrics) {
        *self.performance_metrics.write() = metrics;
    }

    /// Get system health status
    pub fn get_system_health(&self) -> SystemHealth {
        let state = self.fallback_manager.current_state();
        let metrics = self.performance_metrics.read();
        let fallback_stats = self.fallback_manager.get_stats();

        let health_score = match state {
            SystemState::FullExperimental => 100,
            SystemState::QuicWithFec => 80,
            SystemState::QuicBasic => 60,
            SystemState::TcpFallback => 40,
            SystemState::MinimalFallback => 20,
        };

        let status = if fallback_stats.total_fallbacks > 10 {
            HealthStatus::Degraded
        } else if metrics.failed_transfers > metrics.completed_transfers / 10 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };

        SystemHealth {
            status,
            health_score,
            current_state: state,
            active_transfers: metrics.active_transfers,
            failure_rate: if metrics.total_transfers > 0 {
                metrics.failed_transfers as f32 / metrics.total_transfers as f32
            } else {
                0.0
            },
            network_efficiency: metrics.network_efficiency,
            last_fallback: fallback_stats.recent_fallbacks.first().map(|e| e.timestamp),
        }
    }
}

/// Dashboard state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardState {
    pub system_state: SystemState,
    pub fallback_strategy: FallbackStrategy,
    pub fallback_config: FallbackConfig,
    pub system_config: SystemConfig,
    pub network_settings: NetworkSettings,
    pub performance_metrics: PerformanceMetrics,
    pub active_transfers: Vec<TransferControl>,
    pub fallback_history: Vec<FallbackEvent>,
}

/// System health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub health_score: u8,
    pub current_state: SystemState,
    pub active_transfers: usize,
    pub failure_rate: f32,
    pub network_efficiency: f32,
    pub last_fallback: Option<Instant>,
}

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Degraded,
    Critical,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            max_concurrent_transfers: 10,
            default_priority: PacketPriority::Medium,
            default_chunk_size: 64 * 1024, // 64KB
            enable_ai_routing: true,
            enable_compression: true,
            compression_algorithm: CompressionAlgorithm::Lz4,
            enable_encryption: true,
        }
    }
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            preferred_path: NetworkPath::WiFi,
            handover_strategy: HandoverStrategy::Smooth,
            enable_multipath: true,
            max_rtt_threshold: 200.0, // ms
            max_loss_threshold: 0.05, // 5%
            bandwidth_limit: None,
        }
    }
}

use anyhow::Result;

