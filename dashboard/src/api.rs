/// REST API endpoints for dashboard

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use crate::state::DashboardState;
use trackshift::{
    TransferStatus, AiDecision, RouteDecision, Severity, OptimizationHint,
    RetryStrategy, IntegrityMethod, TransferAction, NetworkQuality, NetworkMetricsInput,
    ChunkPriority,
};
use std::sync::Arc;

/// Get system status snapshot
pub async fn status(state: web::Data<Arc<DashboardState>>) -> ActixResult<HttpResponse> {
    let snapshot = state.monitor.get_status_snapshot();
    
    let response = serde_json::json!({
        "transfers": snapshot.transfers.len(),
        "active_transfers": snapshot.health.active_transfers,
        "queue_size": snapshot.health.queue_size,
        "error_rate": snapshot.health.error_rate,
        "network": snapshot.network,
        "scheduler": snapshot.scheduler_stats,
    });
    
    Ok(HttpResponse::Ok().json(response))
}

/// Get current metrics (for Next.js dashboard)
pub async fn metrics_current(state: web::Data<Arc<DashboardState>>) -> ActixResult<HttpResponse> {
    let snapshot = state.monitor.get_status_snapshot();
    let network_status = state.monitor.get_network_status();
    
    // Get active transfer if any
    let active_transfer = snapshot.transfers.iter()
        .find(|t| t.status == TransferStatus::InProgress)
        .or_else(|| snapshot.transfers.iter().next());
    
    // Only return real data - no fake values
    let transfer_json = active_transfer.map(|t| {
        serde_json::json!({
            "file_name": format!("transfer-{}", t.transfer_id),
            "file_size": t.total_bytes,
            "bytes_transferred": t.bytes_transferred,
            "speed_mbps": t.speed_mbps,
            "eta_seconds": t.eta_seconds,
            "status": format!("{:?}", t.status).to_lowercase(),
            "network_rtt_ms": network_status.as_ref().map(|n| n.rtt_ms),
            "priority": format!("{:?}", t.priority),
            "route": t.route.map(|r| {
                // Convert u32 route to RouteDecision string
                match r {
                    0 => "WiFi",
                    1 => "Starlink",
                    2 => "Multipath",
                    _ => "Unknown",
                }.to_string()
            }),
            "integrity_method": format!("{:?}", t.integrity_method),
            "integrity_status": format!("{:?}", t.integrity_status),
        })
    });
    
    // Build network paths from network status
    let paths = if let Some(network) = network_status.as_ref() {
        let mut paths_vec = vec![];
        
        // WiFi path
        if network.wifi_signal > -100.0 {
            paths_vec.push(serde_json::json!({
                "name": "WiFi",
                "type": "wifi",
                "rtt": network.rtt_ms,
                "loss": network.loss_rate * 100.0,
                "jitter": network.jitter_ms,
                "throughput": network.throughput_mbps,
                "status": if network.quality_score > 0.7 { "active" } else { "backup" },
            }));
        }
        
        // Starlink path (only if actually available - use real data)
        if network.starlink_latency > 0.0 {
            paths_vec.push(serde_json::json!({
                "name": "Starlink",
                "type": "starlink",
                "rtt": network.starlink_latency,
                "loss": network.loss_rate * 100.0,
                "jitter": network.jitter_ms,
                "throughput": network.throughput_mbps,
                "status": if network.quality_score > 0.7 { "active" } else { "backup" },
            }));
        }
        
        paths_vec
    } else {
        vec![]
    };
    
    let response = serde_json::json!({
        "transfer": transfer_json,
        "paths": paths,
        "network": network_status.as_ref().map(|n| {
            serde_json::json!({
                "rtt_ms": n.rtt_ms,
                "jitter_ms": n.jitter_ms,
                "loss_rate": n.loss_rate,
                "throughput_mbps": n.throughput_mbps,
                "wifi_signal": n.wifi_signal,
                "wifi_available": n.wifi_signal > -100.0,
                "quality_score": n.quality_score,
                "is_patchy": n.is_patchy,
            })
        }),
        // Only return real FEC data if available (from actual FEC system)
        // Return null if no FEC data available
        "fec": Option::<serde_json::Value>::None,
        
        // Only return real integrity data if available
        // Return null if no integrity data available
        "integrity": Option::<serde_json::Value>::None,
    });
    
    Ok(HttpResponse::Ok().json(response))
}

/// Get all transfers
pub async fn transfers(state: web::Data<Arc<DashboardState>>) -> ActixResult<HttpResponse> {
    let snapshot = state.monitor.get_status_snapshot();
    
    let transfers: Vec<_> = snapshot.transfers.iter().map(|t| {
        serde_json::json!({
            "id": t.transfer_id,
            "status": format!("{:?}", t.status),
            "progress": t.progress,
            "bytes_transferred": t.bytes_transferred,
            "total_bytes": t.total_bytes,
            "speed_mbps": t.speed_mbps,
            "eta_seconds": t.eta_seconds,
            "priority": format!("{:?}", t.priority),
            "route": t.route.map(|r| {
                // Convert u32 route to RouteDecision string
                match r {
                    0 => "WiFi",
                    1 => "Starlink",
                    2 => "Multipath",
                    _ => "Unknown",
                }.to_string()
            }),
            "integrity_method": format!("{:?}", t.integrity_method),
            "retry_count": t.retry_count,
            "error_message": t.error_message,
        })
    }).collect();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "transfers": transfers,
        "count": transfers.len(),
    })))
}

/// Get network status
pub async fn network(state: web::Data<Arc<DashboardState>>) -> ActixResult<HttpResponse> {
    if let Some(network) = state.monitor.get_network_status() {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "rtt_ms": network.rtt_ms,
            "jitter_ms": network.jitter_ms,
            "loss_rate": network.loss_rate,
            "throughput_mbps": network.throughput_mbps,
            "wifi_signal": network.wifi_signal,
            "starlink_latency": network.starlink_latency,
            "quality_score": network.quality_score,
            "is_patchy": network.is_patchy,
            "timestamp": network.timestamp,
        })))
    } else {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "no_data"
        })))
    }
}

/// Get system health
pub async fn health(state: web::Data<Arc<DashboardState>>) -> ActixResult<HttpResponse> {
    let health = state.monitor.get_system_health();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "cpu_usage": health.cpu_usage,
        "memory_usage": health.memory_usage,
        "active_transfers": health.active_transfers,
        "queue_size": health.queue_size,
        "buffer_size": health.buffer_size,
        "error_rate": health.error_rate,
        "timestamp": health.timestamp,
    })))
}

/// Get configuration
pub async fn config(state: web::Data<Arc<DashboardState>>) -> ActixResult<HttpResponse> {
    let config = state.get_config();
    Ok(HttpResponse::Ok().json(config))
}

/// Update configuration
#[derive(Deserialize)]
pub struct ConfigUpdate {
    pub compression_enabled: Option<bool>,
    pub compression_algorithm: Option<String>,
    pub integrity_enabled: Option<bool>,
    pub integrity_method: Option<String>,
    pub default_route: Option<String>,
    pub enable_multipath: Option<bool>,
    pub p0_weight: Option<u32>,
    pub p1_weight: Option<u32>,
    pub p2_weight: Option<u32>,
    pub p2_enabled: Option<bool>,
    pub fec_enabled: Option<bool>,
    pub fec_redundancy: Option<u32>,
    pub buffer_size: Option<usize>,
}

pub async fn config_update(
    state: web::Data<Arc<DashboardState>>,
    update: web::Json<ConfigUpdate>,
) -> ActixResult<HttpResponse> {
    let mut config = state.get_config();
    
    if let Some(v) = update.compression_enabled {
        config.compression_enabled = v;
    }
    if let Some(ref v) = update.compression_algorithm {
        config.compression_algorithm = v.clone();
    }
    if let Some(v) = update.integrity_enabled {
        config.integrity_enabled = v;
    }
    if let Some(ref v) = update.integrity_method {
        config.integrity_method = v.clone();
    }
    if let Some(ref v) = update.default_route {
        config.default_route = v.clone();
    }
    if let Some(v) = update.enable_multipath {
        config.enable_multipath = v;
    }
    if let Some(v) = update.p0_weight {
        config.p0_weight = v;
    }
    if let Some(v) = update.p1_weight {
        config.p1_weight = v;
    }
    if let Some(v) = update.p2_weight {
        config.p2_weight = v;
    }
    if let Some(v) = update.p2_enabled {
        config.p2_enabled = v;
    }
    if let Some(v) = update.fec_enabled {
        config.fec_enabled = v;
    }
    if let Some(v) = update.fec_redundancy {
        config.fec_redundancy = v;
    }
    if let Some(v) = update.buffer_size {
        config.buffer_size = v;
    }
    
    state.update_config(config.clone());
    
    // Update scheduler weights
    // Create a minimal AiDecision for weight updates
    // We only need the weight fields, but must provide all required fields
    use trackshift::NetworkQuality;
    let default_metrics = NetworkMetricsInput::default();
    let network_quality = NetworkQuality::assess(&default_metrics);
    
    let decision = AiDecision {
        route: RouteDecision::WiFi,
        severity: Severity::Low,
        p2_enable: config.p2_enabled,
        congestion_predicted: false,
        wfq_p0_weight: config.p0_weight,
        wfq_p1_weight: config.p1_weight,
        wfq_p2_weight: config.p2_weight,
        should_send: true,
        similarity_score: 0.0,
        optimization_hint: OptimizationHint::SendFull,
        network_quality,
        should_buffer: false,
        retry_strategy: RetryStrategy::Exponential,
        recommended_chunk_size: 64 * 1024,
        enable_parallel_transfer: false,
        parallel_streams: 1,
        integrity_method: IntegrityMethod::Blake3,
        verify_after_transfer: true,
        transfer_status: TransferStatus::Pending,
        estimated_completion_time: 0.0,
        transfer_action: TransferAction::Continue,
    };
    state.scheduler.update_weights(&decision);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "updated",
        "config": config,
    })))
}

/// Control operations
#[derive(Deserialize)]
pub struct ControlRequest {
    pub action: String, // "start_transfer", "stop_transfer", "pause_transfer", "resume_transfer"
    pub transfer_id: Option<String>,
    pub data: Option<Vec<u8>>,
    pub priority: Option<String>,
}

pub async fn control(
    state: web::Data<Arc<DashboardState>>,
    req: web::Json<ControlRequest>,
) -> ActixResult<HttpResponse> {
    match req.action.as_str() {
        "start_transfer" => {
            // Start a new transfer
            let data = req.data.as_ref().ok_or_else(|| {
                actix_web::error::ErrorBadRequest("Missing data")
            })?;
            
            let priority = req.priority.as_ref()
                .and_then(|p| match p.as_str() {
                    "Critical" => Some(ChunkPriority::Critical),
                    "High" => Some(ChunkPriority::High),
                    "Normal" => Some(ChunkPriority::Normal),
                    "Low" => Some(ChunkPriority::Low),
                    "Bulk" => Some(ChunkPriority::Bulk),
                    _ => None,
                })
                .unwrap_or(ChunkPriority::Normal);
            
            let config = state.get_config();
            let integrity_method = match config.integrity_method.as_str() {
                "Blake3" => IntegrityMethod::Blake3,
                "SHA256" => IntegrityMethod::SHA256,
                "CRC32" => IntegrityMethod::CRC32,
                "Checksum" => IntegrityMethod::Checksum,
                _ => IntegrityMethod::Blake3,
            };
            
            let transfer_id = req.transfer_id.clone()
                .unwrap_or_else(|| format!("transfer-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()));
            
            state.monitor.register_transfer(
                transfer_id.clone(),
                data.len() as u64,
                priority,
                integrity_method,
            );
            
            state.scheduler.schedule(data.clone(), priority, None)
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "started",
                "transfer_id": transfer_id,
            })))
        }
        "stop_transfer" => {
            // Stop a transfer (mark as failed)
            if let Some(ref id) = req.transfer_id {
                state.monitor.update_transfer(
                    id,
                    0,
                    TransferStatus::Failed,
                    None,
                ).map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
            }
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "stopped",
            })))
        }
        "pause_transfer" => {
            if let Some(ref id) = req.transfer_id {
                state.monitor.update_transfer(
                    id,
                    0,
                    TransferStatus::Paused,
                    None,
                ).map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
            }
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "paused",
            })))
        }
        "resume_transfer" => {
            if let Some(ref id) = req.transfer_id {
                state.monitor.update_transfer(
                    id,
                    0,
                    TransferStatus::InProgress,
                    None,
                ).map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
            }
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "resumed",
            })))
        }
        _ => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Unknown action",
            })))
        }
    }
}

/// Get available methods
pub async fn methods(_state: web::Data<Arc<DashboardState>>) -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "compression": {
            "algorithms": ["Lz4", "Zstd", "Auto"],
            "default": "Auto",
        },
        "integrity": {
            "methods": ["Blake3", "SHA256", "CRC32", "Checksum", "None"],
            "default": "Blake3",
        },
        "routing": {
            "options": ["WiFi", "Starlink", "Multipath"],
            "default": "WiFi",
        },
        "priority": {
            "levels": ["Critical", "High", "Normal", "Low", "Bulk"],
        },
    })))
}

/// Get statistics
pub async fn stats(state: web::Data<Arc<DashboardState>>) -> ActixResult<HttpResponse> {
    let scheduler_stats = state.scheduler.stats();
    let snapshot = state.monitor.get_status_snapshot();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "scheduler": {
            "critical_queue": scheduler_stats.critical_queue_size,
            "high_queue": scheduler_stats.high_queue_size,
            "normal_queue": scheduler_stats.normal_queue_size,
            "low_queue": scheduler_stats.low_queue_size,
            "bulk_queue": scheduler_stats.bulk_queue_size,
            "total_scheduled": scheduler_stats.total_scheduled,
            "total_sent": scheduler_stats.total_sent,
            "p0_weight": scheduler_stats.p0_weight,
            "p1_weight": scheduler_stats.p1_weight,
            "p2_weight": scheduler_stats.p2_weight,
        },
        "transfers": {
            "total": snapshot.transfers.len(),
            "active": snapshot.health.active_transfers,
            "completed": snapshot.transfers.iter().filter(|t| t.status == TransferStatus::Completed).count(),
            "failed": snapshot.transfers.iter().filter(|t| t.status == TransferStatus::Failed).count(),
        },
        "system": {
            "queue_size": snapshot.health.queue_size,
            "error_rate": snapshot.health.error_rate,
        },
    })))
}
