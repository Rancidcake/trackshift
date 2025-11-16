/// Dashboard state management

use trackshift::*;
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    // Compression settings
    pub compression_enabled: bool,
    pub compression_algorithm: String, // "Lz4", "Zstd", "Auto"
    
    // Integrity settings
    pub integrity_enabled: bool,
    pub integrity_method: String, // "Blake3", "SHA256", "CRC32", "Checksum"
    
    // Routing settings
    pub default_route: String, // "WiFi", "Starlink", "Multipath"
    pub enable_multipath: bool,
    
    // Priority settings
    pub p0_weight: u32,
    pub p1_weight: u32,
    pub p2_weight: u32,
    pub p2_enabled: bool,
    
    // Network settings
    pub fec_enabled: bool,
    pub fec_redundancy: u32, // Percentage
    pub buffer_size: usize,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            compression_enabled: true,
            compression_algorithm: "Auto".to_string(),
            integrity_enabled: true,
            integrity_method: "Blake3".to_string(),
            default_route: "WiFi".to_string(),
            enable_multipath: false,
            p0_weight: 50,
            p1_weight: 30,
            p2_weight: 20,
            p2_enabled: true,
            fec_enabled: true,
            fec_redundancy: 30,
            buffer_size: 1000,
        }
    }
}

pub struct DashboardState {
    // AI system
    pub ai_system: Arc<RwLock<Option<Arc<TelemetryAi>>>>,
    
    // Priority tagger
    pub tagger: Arc<PriorityTagger>,
    
    // Scheduler
    pub scheduler: Arc<PriorityScheduler>,
    
    // Status monitor
    pub monitor: Arc<RealtimeStatusMonitor>,
    
    // Configuration
    pub config: Arc<RwLock<DashboardConfig>>,
    
    // Active transfers
    pub active_transfers: Arc<RwLock<HashMap<String, TransferInfo>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransferInfo {
    pub id: String,
    pub status: String,
    pub progress: f32,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub speed_mbps: f32,
    pub priority: String,
    pub route: Option<String>,
    pub integrity_method: String,
    pub compression_algorithm: Option<String>,
}

impl DashboardState {
    pub fn new() -> Self {
        let scheduler = Arc::new(PriorityScheduler::new());
        let monitor = Arc::new(RealtimeStatusMonitor::with_scheduler(scheduler.clone()));
        
        Self {
            ai_system: Arc::new(RwLock::new(None)),
            tagger: Arc::new(PriorityTagger::new()),
            scheduler,
            monitor,
            config: Arc::new(RwLock::new(DashboardConfig::default())),
            active_transfers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn initialize_ai(&self, slm_path: &str, embedder_path: &str) -> anyhow::Result<()> {
        let ai = Arc::new(TelemetryAi::new(slm_path, embedder_path)?);
        *self.ai_system.write() = Some(ai);
        Ok(())
    }
    
    pub fn update_config(&self, new_config: DashboardConfig) {
        *self.config.write() = new_config;
    }
    
    pub fn get_config(&self) -> DashboardConfig {
        self.config.read().clone()
    }
}

