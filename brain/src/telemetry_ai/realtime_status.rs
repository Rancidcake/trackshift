/// Real-time status monitoring system
/// 
/// Provides real-time status tracking for:
/// - Transfer progress
/// - Network conditions
/// - Queue status
/// - Integrity checks
/// - System health

use crate::telemetry_ai::{
    TransferStatus, IntegrityMethod, ChunkPriority, NetworkMetricsInput,
    PriorityScheduler, SchedulerStats,
};
use anyhow::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Real-time transfer status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferStatusInfo {
    pub transfer_id: String,
    pub status: TransferStatus,
    pub progress: f32,  // 0.0 - 1.0
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub speed_mbps: f32,
    pub eta_seconds: Option<u64>,
    pub integrity_method: IntegrityMethod,
    pub integrity_status: IntegrityCheckStatus,
    pub priority: ChunkPriority,
    pub route: Option<u32>,
    pub retry_count: u32,
    pub error_message: Option<String>,
    pub started_at: u64,
    pub updated_at: u64,
}

/// Integrity check status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrityCheckStatus {
    NotStarted,
    InProgress,
    Passed,
    Failed,
    Skipped,
}

/// Network status snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub rtt_ms: f32,
    pub jitter_ms: f32,
    pub loss_rate: f32,
    pub throughput_mbps: f32,
    pub wifi_signal: f32,
    pub starlink_latency: f32,
    pub quality_score: f32,
    pub is_patchy: bool,
    pub timestamp: u64,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub cpu_usage: f32,  // 0.0 - 1.0
    pub memory_usage: f32,  // 0.0 - 1.0
    pub active_transfers: usize,
    pub queue_size: usize,
    pub buffer_size: usize,
    pub error_rate: f32,  // 0.0 - 1.0
    pub timestamp: u64,
}

/// Real-time status monitor
pub struct RealtimeStatusMonitor {
    transfers: Arc<RwLock<HashMap<String, TransferStatusInfo>>>,
    network_status: Arc<RwLock<Option<NetworkStatus>>>,
    system_health: Arc<RwLock<SystemHealth>>,
    scheduler: Option<Arc<PriorityScheduler>>,
}

impl RealtimeStatusMonitor {
    /// Create a new real-time status monitor
    pub fn new() -> Self {
        Self {
            transfers: Arc::new(RwLock::new(HashMap::new())),
            network_status: Arc::new(RwLock::new(None)),
            system_health: Arc::new(RwLock::new(SystemHealth {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                active_transfers: 0,
                queue_size: 0,
                buffer_size: 0,
                error_rate: 0.0,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            })),
            scheduler: None,
        }
    }

    /// Create with scheduler reference
    pub fn with_scheduler(scheduler: Arc<PriorityScheduler>) -> Self {
        let mut monitor = Self::new();
        monitor.scheduler = Some(scheduler);
        monitor
    }

    /// Register a new transfer
    pub fn register_transfer(
        &self,
        transfer_id: String,
        total_bytes: u64,
        priority: ChunkPriority,
        integrity_method: IntegrityMethod,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let status_info = TransferStatusInfo {
            transfer_id: transfer_id.clone(),
            status: TransferStatus::Pending,
            progress: 0.0,
            bytes_transferred: 0,
            total_bytes,
            speed_mbps: 0.0,
            eta_seconds: None,
            integrity_method,
            integrity_status: IntegrityCheckStatus::NotStarted,
            priority,
            route: None,
            retry_count: 0,
            error_message: None,
            started_at: now,
            updated_at: now,
        };

        self.transfers.write().insert(transfer_id, status_info);
        self.update_system_health();
    }

    /// Update transfer progress
    pub fn update_transfer(
        &self,
        transfer_id: &str,
        bytes_transferred: u64,
        status: TransferStatus,
        speed_mbps: Option<f32>,
    ) -> Result<()> {
        let mut transfers = self.transfers.write();
        
        if let Some(transfer) = transfers.get_mut(transfer_id) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            transfer.bytes_transferred = bytes_transferred;
            transfer.status = status;
            transfer.updated_at = now;
            
            if transfer.total_bytes > 0 {
                transfer.progress = bytes_transferred as f32 / transfer.total_bytes as f32;
            }
            
            if let Some(speed) = speed_mbps {
                transfer.speed_mbps = speed;
                
                // Calculate ETA
                if speed > 0.0 && transfer.progress < 1.0 {
                    let remaining_bytes = transfer.total_bytes - bytes_transferred;
                    let remaining_mb = remaining_bytes as f32 / 1_000_000.0;
                    transfer.eta_seconds = Some((remaining_mb / speed) as u64);
                }
            }
            
            self.update_system_health();
        }
        
        Ok(())
    }

    /// Update integrity check status
    pub fn update_integrity_status(
        &self,
        transfer_id: &str,
        status: IntegrityCheckStatus,
    ) -> Result<()> {
        let mut transfers = self.transfers.write();
        
        if let Some(transfer) = transfers.get_mut(transfer_id) {
            transfer.integrity_status = status;
            transfer.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if status == IntegrityCheckStatus::Failed {
                transfer.status = TransferStatus::Corrupted;
                transfer.error_message = Some("Integrity check failed".to_string());
            } else if status == IntegrityCheckStatus::Passed {
                transfer.status = TransferStatus::Completed;
            }
        }
        
        Ok(())
    }

    /// Update network status
    pub fn update_network_status(&self, metrics: NetworkMetricsInput, quality_score: f32, is_patchy: bool) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let status = NetworkStatus {
            rtt_ms: metrics.rtt_ms,
            jitter_ms: metrics.jitter_ms,
            loss_rate: metrics.loss_rate,
            throughput_mbps: metrics.throughput_mbps,
            wifi_signal: metrics.wifi_signal,
            starlink_latency: metrics.starlink_latency,
            quality_score,
            is_patchy,
            timestamp: now,
        };

        *self.network_status.write() = Some(status);
    }

    /// Update system health
    pub fn update_system_health(&self) {
        let transfers = self.transfers.read();
        let active_transfers = transfers.values()
            .filter(|t| t.status == TransferStatus::InProgress || t.status == TransferStatus::Verifying)
            .count();
        
        let queue_size = if let Some(ref scheduler) = self.scheduler {
            scheduler.total_queued()
        } else {
            0
        };

        let error_rate = if transfers.len() > 0 {
            let failed = transfers.values()
                .filter(|t| t.status == TransferStatus::Failed || t.status == TransferStatus::Corrupted)
                .count();
            failed as f32 / transfers.len() as f32
        } else {
            0.0
        };

        let mut health = self.system_health.write();
        health.active_transfers = active_transfers;
        health.queue_size = queue_size;
        health.error_rate = error_rate;
        health.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Get transfer status
    pub fn get_transfer_status(&self, transfer_id: &str) -> Option<TransferStatusInfo> {
        self.transfers.read().get(transfer_id).cloned()
    }

    /// Get all active transfers
    pub fn get_active_transfers(&self) -> Vec<TransferStatusInfo> {
        self.transfers.read()
            .values()
            .filter(|t| t.status == TransferStatus::InProgress || t.status == TransferStatus::Verifying)
            .cloned()
            .collect()
    }

    /// Get network status
    pub fn get_network_status(&self) -> Option<NetworkStatus> {
        self.network_status.read().clone()
    }

    /// Get system health
    pub fn get_system_health(&self) -> SystemHealth {
        self.system_health.read().clone()
    }

    /// Get comprehensive status snapshot
    pub fn get_status_snapshot(&self) -> StatusSnapshot {
        StatusSnapshot {
            transfers: self.transfers.read().values().cloned().collect(),
            network: self.get_network_status(),
            health: self.get_system_health(),
            scheduler_stats: self.scheduler.as_ref()
                .map(|s| {
                    let stats = s.stats();
                    SchedulerStatusSnapshot {
                        critical_queue: stats.critical_queue_size,
                        high_queue: stats.high_queue_size,
                        normal_queue: stats.normal_queue_size,
                        low_queue: stats.low_queue_size,
                        bulk_queue: stats.bulk_queue_size,
                        p0_weight: stats.p0_weight,
                        p1_weight: stats.p1_weight,
                        p2_weight: stats.p2_weight,
                    }
                }),
        }
    }

    /// Remove completed/failed transfers (cleanup)
    pub fn cleanup_old_transfers(&self, max_age_seconds: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut transfers = self.transfers.write();
        transfers.retain(|_, transfer| {
            let age = now.saturating_sub(transfer.updated_at);
            age < max_age_seconds || 
            (transfer.status != TransferStatus::Completed && 
             transfer.status != TransferStatus::Failed &&
             transfer.status != TransferStatus::Corrupted)
        });
    }
}

impl Default for RealtimeStatusMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive status snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusSnapshot {
    pub transfers: Vec<TransferStatusInfo>,
    pub network: Option<NetworkStatus>,
    pub health: SystemHealth,
    pub scheduler_stats: Option<SchedulerStatusSnapshot>,
}

/// Scheduler status snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStatusSnapshot {
    pub critical_queue: usize,
    pub high_queue: usize,
    pub normal_queue: usize,
    pub low_queue: usize,
    pub bulk_queue: usize,
    pub p0_weight: u32,
    pub p1_weight: u32,
    pub p2_weight: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_registration() {
        let monitor = RealtimeStatusMonitor::new();
        monitor.register_transfer(
            "test-1".to_string(),
            1000,
            ChunkPriority::High,
            IntegrityMethod::SHA256,
        );

        let status = monitor.get_transfer_status("test-1");
        assert!(status.is_some());
        assert_eq!(status.unwrap().total_bytes, 1000);
    }

    #[test]
    fn test_transfer_update() {
        let monitor = RealtimeStatusMonitor::new();
        monitor.register_transfer(
            "test-1".to_string(),
            1000,
            ChunkPriority::High,
            IntegrityMethod::SHA256,
        );

        monitor.update_transfer(
            "test-1",
            500,
            TransferStatus::InProgress,
            Some(10.0),
        ).unwrap();

        let status = monitor.get_transfer_status("test-1").unwrap();
        assert_eq!(status.progress, 0.5);
        assert_eq!(status.speed_mbps, 10.0);
    }
}

