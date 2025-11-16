use ndarray::Array2;
use ort::session::{Session, SessionInputValue};
use ort::session::builder::SessionBuilder;
use ort::value::Value;
use std::sync::Arc;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};

mod vector_store;
mod network_quality;
mod buffer;
mod priority_tagger;
mod scheduler;
mod realtime_status;
mod reinforcement_learning;
mod rl_integration;

use vector_store::SimpleVectorStore;
pub use network_quality::{NetworkQuality, NetworkAction};
pub use buffer::{TelemetryBuffer, BufferedChunk, BufferStatus};
pub use priority_tagger::{PriorityTagger, ChunkPriority, DataFormat, DataScenario};
pub use scheduler::{PriorityScheduler, ScheduledChunk, SchedulerStats};
pub use realtime_status::{
    RealtimeStatusMonitor, TransferStatusInfo, NetworkStatus, SystemHealth,
    StatusSnapshot, IntegrityCheckStatus,
};
pub use reinforcement_learning::{
    RLManager, QLearningAgent, PolicyGradientAgent, NetworkState, RLAction, Reward, RLStats,
};
pub use rl_integration::{RLRecorder, TransferEpisode};

/// Network metrics input (simplified for high-level API)
#[derive(Debug, Clone)]
pub struct NetworkMetricsInput {
    pub rtt_ms: f32,
    pub jitter_ms: f32,
    pub loss_rate: f32,
    pub throughput_mbps: f32,
    pub retransmissions: f32,
    pub queue_p0: f32,
    pub queue_p1: f32,
    pub queue_p2: f32,
    pub p0_rate: f32,
    pub p1_rate: f32,
    pub p2_rate: f32,
    pub wifi_signal: f32,
    pub starlink_latency: f32,
    pub session_state: f32, // 0=ACTIVE, 1=BREAK
    pub retries: f32,
}

impl Default for NetworkMetricsInput {
    fn default() -> Self {
        Self {
            rtt_ms: 15.0,
            jitter_ms: 2.0,
            loss_rate: 0.001,
            throughput_mbps: 100.0,
            retransmissions: 0.0,
            queue_p0: 0.0,
            queue_p1: 0.0,
            queue_p2: 0.0,
            p0_rate: 0.0,
            p1_rate: 0.0,
            p2_rate: 0.0,
            wifi_signal: -50.0,
            starlink_latency: 40.0,
            session_state: 0.0,
            retries: 0.0,
        }
    }
}

/// File type classification for transfer optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Media = 0,      // Video, audio files (large, streaming-friendly)
    Document = 1,   // PDFs, office docs (medium, compressible)
    Data = 2,       // Database, logs (variable, may need integrity)
    Archive = 3,   // ZIP, TAR (already compressed)
    Binary = 4,     // Executables, firmware (critical integrity)
    Unknown = 5,    // Unknown type
}

impl From<u32> for FileType {
    fn from(value: u32) -> Self {
        match value {
            0 => FileType::Media,
            1 => FileType::Document,
            2 => FileType::Data,
            3 => FileType::Archive,
            4 => FileType::Binary,
            _ => FileType::Unknown,
        }
    }
}

/// File priority level for transfer scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilePriority {
    Critical = 0,   // Emergency, real-time critical
    High = 1,       // Important, time-sensitive
    Normal = 2,     // Standard priority
    Low = 3,        // Background, bulk transfers
}

impl From<u32> for FilePriority {
    fn from(value: u32) -> Self {
        match value {
            0 => FilePriority::Critical,
            1 => FilePriority::High,
            2 => FilePriority::Normal,
            _ => FilePriority::Low,
        }
    }
}

/// Input features for the AI decision model
#[derive(Debug, Clone)]
pub struct AiInput {
    // Network metrics (existing)
    pub rtt_ms: f32,
    pub jitter_ms: f32,
    pub loss_rate: f32,
    pub throughput_mbps: f32,
    pub retransmissions: f32,
    pub queue_p0: f32,
    pub queue_p1: f32,
    pub queue_p2: f32,
    pub p0_rate: f32,
    pub p1_rate: f32,
    pub p2_rate: f32,
    pub wifi_signal: f32,
    pub starlink_latency: f32,
    pub session_state: f32, // 0=ACTIVE, 1=BREAK
    
    // Embeddings (existing)
    pub embed_current: [f32; 128], // from ONNX embedding model
    pub embed_context: [f32; 128],  // from HNSW nearest neighbor
    
    // Chunk metadata (existing)
    pub chunk_size: f32,
    pub retries: f32,
    
    // File transfer metadata (NEW)
    /// Total file size in bytes (0.0 if not a file transfer)
    pub file_size: f32,
    /// File type (0=Media, 1=Document, 2=Data, 3=Archive, 4=Binary, 5=Unknown)
    pub file_type: f32,
    /// File priority (0=Critical, 1=High, 2=Normal, 3=Low)
    pub file_priority: f32,
    /// Transfer progress (0.0-1.0, 1.0 = complete)
    pub transfer_progress: f32,
    /// Bytes transferred so far
    pub bytes_transferred: f32,
    /// Whether integrity check is required (1.0=yes, 0.0=no)
    pub requires_integrity_check: f32,
    /// Number of failed integrity checks (0.0+)
    pub integrity_check_failures: f32,
    /// Whether this is a resume/retry of previous transfer (1.0=yes, 0.0=no)
    pub is_resume: f32,
}

/// Route decision enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteDecision {
    WiFi = 0,
    Starlink = 1,
    Multipath = 2,
    FiveG = 3,  // Added 5G support
}

impl From<u32> for RouteDecision {
    fn from(value: u32) -> Self {
        match value {
            0 => RouteDecision::WiFi,
            1 => RouteDecision::Starlink,
            2 => RouteDecision::Multipath,
            3 => RouteDecision::FiveG,
            _ => RouteDecision::WiFi, // default fallback
        }
    }
}

/// Severity classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    High = 0,
    Low = 1,
}

impl From<u32> for Severity {
    fn from(value: u32) -> Self {
        match value {
            0 => Severity::High,
            _ => Severity::Low,
        }
    }
}

/// Transfer status for real-time monitoring
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending = 0,       // Waiting to start
    InProgress = 1,    // Actively transferring
    Paused = 2,        // Paused (network issues)
    Verifying = 3,     // Integrity check in progress
    Completed = 4,     // Successfully completed
    Failed = 5,        // Transfer failed
    Corrupted = 6,     // Integrity check failed
}

impl From<u32> for TransferStatus {
    fn from(value: u32) -> Self {
        match value {
            0 => TransferStatus::Pending,
            1 => TransferStatus::InProgress,
            2 => TransferStatus::Paused,
            3 => TransferStatus::Verifying,
            4 => TransferStatus::Completed,
            5 => TransferStatus::Failed,
            _ => TransferStatus::Corrupted,
        }
    }
}

/// Integrity check method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrityMethod {
    None = 0,          // No integrity check
    Checksum = 1,      // Simple checksum (fast)
    CRC32 = 2,         // CRC32 (balanced)
    SHA256 = 3,        // SHA256 (strong, slower)
    Blake3 = 4,        // Blake3 (fast, secure, recommended)
    Parallel = 5,      // Transfer + verify in parallel
}

impl From<u32> for IntegrityMethod {
    fn from(value: u32) -> Self {
        match value {
            0 => IntegrityMethod::None,
            1 => IntegrityMethod::Checksum,
            2 => IntegrityMethod::CRC32,
            3 => IntegrityMethod::SHA256,
            4 => IntegrityMethod::Blake3,
            _ => IntegrityMethod::Parallel,
        }
    }
}

/// AI decision output struct
#[derive(Debug, Clone)]
pub struct AiDecision {
    // Routing and scheduling (existing)
    pub route: RouteDecision,
    pub severity: Severity,
    pub p2_enable: bool,
    pub congestion_predicted: bool,
    pub wfq_p0_weight: u32,
    pub wfq_p1_weight: u32,
    pub wfq_p2_weight: u32,
    
    // Data optimization (existing)
    /// Whether this data should be sent (false if redundant)
    pub should_send: bool,
    /// Similarity score to previous data (0.0-1.0, 1.0 = identical)
    pub similarity_score: f32,
    /// Compression/optimization recommendation
    pub optimization_hint: OptimizationHint,
    
    // Network quality (existing)
    /// Network quality assessment
    pub network_quality: NetworkQuality,
    /// Should buffer data (network too bad to send)
    pub should_buffer: bool,
    /// Retry strategy recommendation
    pub retry_strategy: RetryStrategy,
    
    // File transfer features (NEW)
    /// Recommended chunk size for file transfer (bytes)
    pub recommended_chunk_size: u32,
    /// Whether to enable parallel transfers (multiple chunks simultaneously)
    pub enable_parallel_transfer: bool,
    /// Number of parallel streams to use (1-8)
    pub parallel_streams: u8,
    /// Integrity check method to use
    pub integrity_method: IntegrityMethod,
    /// Whether to verify integrity after transfer
    pub verify_after_transfer: bool,
    /// Current transfer status
    pub transfer_status: TransferStatus,
    /// Estimated time to completion (seconds, 0.0 if unknown)
    pub estimated_completion_time: f32,
    /// Recommended action for current transfer
    pub transfer_action: TransferAction,
}

/// Transfer action recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferAction {
    Continue = 0,      // Continue normal transfer
    Pause = 1,        // Pause and wait for better network
    Resume = 2,       // Resume paused transfer
    Retry = 3,        // Retry failed transfer
    SwitchRoute = 4,  // Switch to different network route
    Compress = 5,     // Compress before transfer
    Split = 6,        // Split into smaller chunks
}

/// Optimization hints based on data patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationHint {
    SendFull,      // Send full data (new/important)
    SendDelta,     // Send only changes (similar data)
    Skip,          // Skip sending (redundant)
    Compress,      // Compress before sending (repetitive pattern)
}

/// Retry strategy for patchy networks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryStrategy {
    Immediate,     // Retry immediately (good network)
    Exponential,   // Exponential backoff (patchy network)
    Aggressive,    // Aggressive retries (critical data)
    Buffer,        // Buffer and retry later (network down)
}

/// Vector store for context retrieval
/// Currently uses a simple cosine similarity implementation
/// Can be swapped with HNSW implementation when dependency issues are resolved
pub type HnswContextStore = SimpleVectorStore;

/// Telemetry AI decision engine
pub struct TelemetryAi {
    slm_session: Arc<Session>,
    embedder_session: Arc<Session>,
    context_store: Arc<parking_lot::RwLock<HnswContextStore>>,
    /// Reinforcement Learning manager (optional, enabled by default)
    rl_manager: Option<Arc<RLManager>>,
}

impl TelemetryAi {
    /// Initialize the telemetry AI system
    /// 
    /// # Arguments
    /// * `slm_model_path` - Path to the SLM ONNX model file (slm.onnx)
    /// * `embedder_model_path` - Path to the embedder ONNX model file (embedder.onnx)
    pub fn new(slm_model_path: &str, embedder_model_path: &str) -> Result<Self> {
        // Load SLM (decision model) - ort 2.0.0-rc.10 API
        let slm_session = Arc::new(
            SessionBuilder::new()?
                .commit_from_file(slm_model_path)
                .context("Failed to load SLM model")?
        );

        // Load embedder model - ort 2.0.0-rc.10 API
        let embedder_session = Arc::new(
            SessionBuilder::new()?
                .commit_from_file(embedder_model_path)
                .context("Failed to load embedder model")?
        );

        // Initialize vector store (128-dimensional embeddings)
        let context_store = Arc::new(parking_lot::RwLock::new(
            HnswContextStore::new(128)
        ));

        // Initialize RL manager (enabled by default)
        let rl_manager = Some(Arc::new(reinforcement_learning::RLManager::new()));

        Ok(Self {
            slm_session,
            embedder_session,
            context_store,
            rl_manager,
        })
    }

    /// Create TelemetryAi with RL disabled
    pub fn new_without_rl(slm_model_path: &str, embedder_model_path: &str) -> Result<Self> {
        let mut ai = Self::new(slm_model_path, embedder_model_path)?;
        ai.rl_manager = None;
        Ok(ai)
    }

    /// Enable or disable RL
    pub fn set_rl_enabled(&mut self, enabled: bool) {
        if enabled && self.rl_manager.is_none() {
            self.rl_manager = Some(Arc::new(reinforcement_learning::RLManager::new()));
        } else if !enabled {
            self.rl_manager = None;
        }
    }

    /// Get RL manager (if enabled)
    pub fn rl_manager(&self) -> Option<Arc<reinforcement_learning::RLManager>> {
        self.rl_manager.clone()
    }
    
    /// Generate embedding from telemetry chunk data
    /// 
    /// # Arguments
    /// * `chunk_data` - Raw telemetry data (will be padded/truncated to 1024 floats)
    /// 
    /// # Returns
    /// 128-dimensional embedding vector
    pub fn embed_chunk(&self, chunk_data: &[u8]) -> Result<[f32; 128]> {
        // Convert bytes to floats (normalize to 0-1 range)
        // Pad or truncate to 1024 elements as required by embedder model
        let mut input_vec = Vec::with_capacity(1024);
        
        // Convert bytes to floats (simple normalization)
        for byte in chunk_data.iter().take(1024) {
            input_vec.push(*byte as f32 / 255.0);
        }
        
        // Pad if needed
        while input_vec.len() < 1024 {
            input_vec.push(0.0);
        }
        
        // Create input tensor: shape (1, 1024)
        let input_array = Array2::from_shape_vec((1, 1024), input_vec)
            .context("Failed to create embedder input array")?;
        
        // Convert to ONNX Value - ort 2.0.0-rc.10 API
        // Convert to tuple format (shape, data) that ort expects
        let shape_vec: Vec<usize> = input_array.shape().iter().copied().collect();
        let data: Vec<f32> = input_array.iter().cloned().collect();
        let input_value = Value::from_array((&shape_vec[..], data))
            .context("Failed to convert to ONNX value")?;
        
        // Run embedder inference - ort 2.0.0-rc.10 API
        // Convert Value to SessionInputValue
        // Note: Session::run requires &mut, but Session doesn't implement Clone
        // Use unsafe to get mutable reference from Arc (safe because we have exclusive access)
        let input: SessionInputValue = input_value.into();
        let outputs = unsafe {
            let session_ptr = Arc::as_ptr(&self.embedder_session) as *mut Session;
            (&mut *session_ptr).run([input])
                .context("Failed to run embedder inference")?
        };
        
        // Extract output tensor - ort 2.0.0-rc.10 returns tuple (shape, data)
        let output_tensor = outputs[0]
            .try_extract_tensor::<f32>()
            .context("Failed to extract embedder output")?;
        
        // ort 2.0.0-rc.10 returns (Shape, &[f32])
        let (_, output_slice) = output_tensor;
        
        // Ensure we have 128 values
        if output_slice.len() < 128 {
            anyhow::bail!("Embedder output has {} values, expected 128", output_slice.len());
        }
        
        // Copy to fixed-size array
        let mut embedding = [0.0f32; 128];
        embedding.copy_from_slice(&output_slice[..128]);
        
        Ok(embedding)
    }
    
    /// Check if data is redundant (too similar to previous data)
    /// 
    /// # Arguments
    /// * `embedding` - Current chunk embedding
    /// * `similarity_threshold` - Threshold above which data is considered redundant (default: 0.95)
    /// 
    /// # Returns
    /// (similarity_score, is_redundant)
    pub fn check_redundancy(
        &self,
        embedding: &[f32; 128],
        similarity_threshold: f32,
    ) -> Result<(f32, bool)> {
        let store = self.context_store.read();
        
        if store.is_empty() {
            // No previous data, not redundant
            return Ok((0.0, false));
        }
        
        // Find most similar past embedding
        let neighbors = store.query(embedding, 1)?;
        
        if neighbors.is_empty() {
            return Ok((0.0, false));
        }
        
        // Calculate similarity
        let similarity = vector_store::SimpleVectorStore::cosine_similarity(
            embedding,
            &neighbors[0]
        );
        
        let is_redundant = similarity >= similarity_threshold;
        
        Ok((similarity, is_redundant))
    }
    
    /// Process a telemetry chunk end-to-end and get decision
    /// This is the main high-level API - handles everything automatically
    /// 
    /// # Arguments
    /// * `chunk_data` - Raw telemetry chunk bytes
    /// * `network_metrics` - Network metrics (RTT, jitter, etc.)
    /// * `redundancy_threshold` - Similarity threshold for redundancy detection (default: 0.95)
    /// 
    /// # Returns
    /// AI decision with routing, scheduling, congestion predictions, and redundancy detection
    pub fn process_chunk(
        &self,
        chunk_data: &[u8],
        network_metrics: NetworkMetricsInput,
    ) -> Result<AiDecision> {
        self.process_chunk_with_threshold(chunk_data, network_metrics, 0.95)
    }
    
    /// Process a file transfer chunk with file-specific metadata
    /// Optimized for file transfer scenarios with integrity checks and progress tracking
    /// 
    /// # Arguments
    /// * `chunk_data` - File chunk bytes
    /// * `network_metrics` - Network metrics
    /// * `file_size` - Total file size in bytes
    /// * `file_type` - File type classification
    /// * `file_priority` - File priority level
    /// * `bytes_transferred` - Bytes already transferred
    /// * `requires_integrity_check` - Whether integrity verification is required
    /// * `integrity_check_failures` - Number of previous integrity check failures
    /// * `is_resume` - Whether this is resuming a previous transfer
    /// 
    /// # Returns
    /// AI decision optimized for file transfer with integrity checks and status tracking
    pub fn process_file_transfer(
        &self,
        chunk_data: &[u8],
        network_metrics: NetworkMetricsInput,
        file_size: u64,
        file_type: FileType,
        file_priority: FilePriority,
        bytes_transferred: u64,
        requires_integrity_check: bool,
        integrity_check_failures: u32,
        is_resume: bool,
    ) -> Result<AiDecision> {
        // Assess network quality
        let network_quality = NetworkQuality::assess(&network_metrics);
        
        // Generate embedding
        let embedding = self.embed_chunk(chunk_data)?;
        
        // Get context embedding
        let context_embedding = {
            let store = self.context_store.read();
            store.get_context(&embedding)
                .context("Failed to retrieve context")?
        };
        
        // Calculate transfer progress
        let transfer_progress = if file_size > 0 {
            (bytes_transferred as f32 / file_size as f32).min(1.0)
        } else {
            0.0
        };
        
        // Build AI input with file transfer metadata
        let ai_input = AiInput {
            rtt_ms: network_metrics.rtt_ms,
            jitter_ms: network_metrics.jitter_ms,
            loss_rate: network_metrics.loss_rate,
            throughput_mbps: network_metrics.throughput_mbps,
            retransmissions: network_metrics.retransmissions,
            queue_p0: network_metrics.queue_p0,
            queue_p1: network_metrics.queue_p1,
            queue_p2: network_metrics.queue_p2,
            p0_rate: network_metrics.p0_rate,
            p1_rate: network_metrics.p1_rate,
            p2_rate: network_metrics.p2_rate,
            wifi_signal: network_metrics.wifi_signal,
            starlink_latency: network_metrics.starlink_latency,
            session_state: network_metrics.session_state,
            embed_current: embedding,
            embed_context: context_embedding,
            chunk_size: chunk_data.len() as f32,
            retries: network_metrics.retries,
            // File transfer metadata
            file_size: file_size as f32,
            file_type: file_type as u32 as f32,
            file_priority: file_priority as u32 as f32,
            transfer_progress,
            bytes_transferred: bytes_transferred as f32,
            requires_integrity_check: if requires_integrity_check { 1.0 } else { 0.0 },
            integrity_check_failures: integrity_check_failures as f32,
            is_resume: if is_resume { 1.0 } else { 0.0 },
        };
        
        // Make base decision
        let mut decision = self.ai_decide(&ai_input)?;
        
        // Add network quality
        decision.network_quality = network_quality;
        
        // File transfer specific logic
        decision.should_send = true; // Always send file chunks (no redundancy check for files)
        decision.similarity_score = 0.0; // Not applicable for file transfers
        
        // Determine integrity check method based on file type and priority
        // Blake3 is recommended for most cases (fast and secure)
        decision.integrity_method = if !requires_integrity_check {
            IntegrityMethod::None
        } else if file_priority == FilePriority::Critical || file_type == FileType::Binary {
            IntegrityMethod::Blake3  // Fast and secure for critical/binary files
        } else if file_size > 100_000_000 { // > 100MB
            IntegrityMethod::Blake3  // Blake3 is fast even for large files
        } else {
            IntegrityMethod::Blake3  // Blake3 recommended for all files
        };
        
        decision.verify_after_transfer = requires_integrity_check;
        
        // Determine recommended chunk size based on network and file size
        decision.recommended_chunk_size = if network_quality.is_patchy {
            // Smaller chunks on patchy networks for better resume capability
            if file_size > 10_000_000 {
                64 * 1024  // 64KB for large files
            } else {
                32 * 1024  // 32KB for smaller files
            }
        } else if file_size > 100_000_000 {
            1024 * 1024  // 1MB for large files on good networks
        } else {
            256 * 1024  // 256KB default
        };
        
        // Enable parallel transfer for large files on good networks
        decision.enable_parallel_transfer = file_size > 10_000_000 && network_quality.score > 0.7;
        decision.parallel_streams = if decision.enable_parallel_transfer {
            // More streams for better networks
            if network_quality.score > 0.9 {
                4
            } else {
                2
            }
        } else {
            1
        };
        
        // Determine transfer status
        decision.transfer_status = if integrity_check_failures > 2 {
            TransferStatus::Corrupted
        } else if !network_quality.is_connected || network_quality.score < 0.3 {
            TransferStatus::Paused
        } else if transfer_progress >= 1.0 {
            TransferStatus::Completed
        } else {
            TransferStatus::InProgress
        };
        
        // Estimate completion time
        decision.estimated_completion_time = if network_metrics.throughput_mbps > 0.0 && file_size > bytes_transferred {
            let remaining_bytes = (file_size - bytes_transferred) as f32;
            let remaining_mbits = remaining_bytes * 8.0 / 1_000_000.0;
            remaining_mbits / network_metrics.throughput_mbps
        } else {
            0.0
        };
        
        // Determine transfer action
        decision.transfer_action = if integrity_check_failures > 2 {
            TransferAction::Retry
        } else if !network_quality.is_connected {
            TransferAction::Pause
        } else if is_resume {
            TransferAction::Resume
        } else if network_quality.is_patchy && file_type == FileType::Document {
            TransferAction::Compress
        } else if network_quality.score < 0.5 && decision.route != RouteDecision::Multipath {
            TransferAction::SwitchRoute
        } else if file_size > 100_000_000 && network_quality.is_patchy {
            TransferAction::Split
        } else {
            TransferAction::Continue
        };
        
        // Adjust retry strategy for file transfers
        decision.retry_strategy = if integrity_check_failures > 0 {
            RetryStrategy::Aggressive  // Aggressive retry if integrity failed
        } else if !network_quality.is_connected {
            RetryStrategy::Buffer  // Buffer and resume later
        } else if file_priority == FilePriority::Critical {
            RetryStrategy::Aggressive
        } else {
            RetryStrategy::Exponential  // Exponential backoff for file transfers
        };
        
        decision.should_buffer = !network_quality.is_connected || network_quality.score < 0.2;
        
        // Adjust optimization hint for file types
        decision.optimization_hint = match file_type {
            FileType::Archive => OptimizationHint::SendFull,  // Already compressed
            FileType::Document if network_quality.is_patchy => OptimizationHint::Compress,
            FileType::Media => OptimizationHint::SendFull,  // Streaming-friendly
            _ => OptimizationHint::SendFull,
        };
        
        Ok(decision)
    }
    
    /// Process chunk with custom redundancy threshold
    pub fn process_chunk_with_threshold(
        &self,
        chunk_data: &[u8],
        network_metrics: NetworkMetricsInput,
        redundancy_threshold: f32,
    ) -> Result<AiDecision> {
        // 0. Assess network quality FIRST (for adaptive behavior)
        let network_quality = NetworkQuality::assess(&network_metrics);
        
        // Use adaptive threshold if network is patchy
        let effective_threshold = if network_quality.is_patchy {
            network_quality.adaptive_redundancy_threshold()
        } else {
            redundancy_threshold
        };
        
        // 1. Generate embedding from chunk
        let embedding = self.embed_chunk(chunk_data)?;
        
        // 2. Check for redundancy BEFORE storing (with adaptive threshold)
        let (similarity_score, is_redundant) = self.check_redundancy(&embedding, effective_threshold)?;
        
        // 3. Only store if not redundant (to avoid polluting context store)
        if !is_redundant {
            self.insert_embedding(&embedding)?;
        }
        
        // 4. Get context embedding (similar past situations)
        let context_embedding = {
            let store = self.context_store.read();
            store.get_context(&embedding)
                .context("Failed to retrieve context")?
        };
        
        // 5. Build AI input (for telemetry, file transfer fields default to 0)
        let ai_input = AiInput {
            rtt_ms: network_metrics.rtt_ms,
            jitter_ms: network_metrics.jitter_ms,
            loss_rate: network_metrics.loss_rate,
            throughput_mbps: network_metrics.throughput_mbps,
            retransmissions: network_metrics.retransmissions,
            queue_p0: network_metrics.queue_p0,
            queue_p1: network_metrics.queue_p1,
            queue_p2: network_metrics.queue_p2,
            p0_rate: network_metrics.p0_rate,
            p1_rate: network_metrics.p1_rate,
            p2_rate: network_metrics.p2_rate,
            wifi_signal: network_metrics.wifi_signal,
            starlink_latency: network_metrics.starlink_latency,
            session_state: network_metrics.session_state,
            embed_current: embedding,
            embed_context: context_embedding,
            chunk_size: chunk_data.len() as f32,
            retries: network_metrics.retries,
            // File transfer fields (default to 0 for telemetry)
            file_size: 0.0,
            file_type: 5.0, // Unknown
            file_priority: 2.0, // Normal
            transfer_progress: 0.0,
            bytes_transferred: 0.0,
            requires_integrity_check: 0.0,
            integrity_check_failures: 0.0,
            is_resume: 0.0,
        };
        
        // 6. Make decision
        let mut decision = self.ai_decide(&ai_input)?;
        
        // 7. Add redundancy detection results
        decision.similarity_score = similarity_score;
        decision.should_send = !is_redundant;
        
        // 8. Determine optimization hint based on similarity AND network quality
        decision.optimization_hint = if is_redundant {
            OptimizationHint::Skip
        } else if network_quality.should_compress() {
            // Always compress on patchy networks
            OptimizationHint::Compress
        } else if similarity_score > 0.85 {
            OptimizationHint::SendDelta  // Similar but not identical - send only changes
        } else if similarity_score > 0.70 {
            OptimizationHint::Compress   // Some repetition - compress
        } else {
            OptimizationHint::SendFull   // New/unique data - send full
        };
        
        // 9. Add network quality assessment
        decision.network_quality = network_quality;
        
        // 10. Determine if should buffer (network too bad)
        decision.should_buffer = !network_quality.is_connected || network_quality.score < 0.3;
        
        // 11. Determine retry strategy based on network quality
        decision.retry_strategy = if !network_quality.is_connected {
            RetryStrategy::Buffer  // Network down - buffer and retry later
        } else if network_quality.score < 0.4 {
            RetryStrategy::Exponential  // Very patchy - exponential backoff
        } else if decision.severity == Severity::High {
            RetryStrategy::Aggressive  // Critical data - retry aggressively
        } else {
            RetryStrategy::Immediate  // Good network - retry immediately
        };
        
        // 12. Adjust route for patchy networks (prefer multipath if available)
        if network_quality.is_patchy && network_quality.score < 0.5 {
            // Prefer multipath for redundancy on patchy networks
            if decision.route != RouteDecision::Multipath {
                // Only override if not already multipath
                // In real system, check if multipath is available
            }
        }
        
        // 13. Adjust weights for patchy networks (prioritize critical)
        if network_quality.prioritize_critical_only() {
            // Give more weight to priority 0 (critical) on bad networks
            // Rebalance to favor P0: 70% P0, 20% P1, 10% P2
            decision.wfq_p0_weight = 70;
            decision.wfq_p1_weight = 20;
            decision.wfq_p2_weight = 10;
        }
        
        Ok(decision)
    }

    /// Get mutable reference to context store for inserting embeddings
    pub fn context_store(&self) -> Arc<parking_lot::RwLock<HnswContextStore>> {
        self.context_store.clone()
    }
    
    /// Insert an embedding into the context store
    pub fn insert_embedding(&self, embedding: &[f32; 128]) -> Result<usize> {
        let mut store = self.context_store.write();
        store.insert(embedding)
    }

    /// Main decision function - called for each telemetry chunk
    pub fn ai_decide(&self, input: &AiInput) -> Result<AiDecision> {
        // 1. Get context embedding from HNSW
        let context_embedding = {
            let store = self.context_store.read();
            store.get_context(&input.embed_current)
                .context("Failed to retrieve context from HNSW")?
        };

        // 2. Preprocess: Convert AiInput → feature vector (269 features)
        // Total: 13 numeric + 128 embed_current + 128 embed_context = 269 features
        let features = Self::preprocess_features(input, &context_embedding);

        // 3. Run ONNX inference
        // Note: Session::run requires &mut, but Session doesn't implement Clone
        // Use unsafe to get mutable reference from Arc
        let decision = unsafe {
            let session_ptr = Arc::as_ptr(&self.slm_session) as *mut Session;
            Self::run_inference(&mut *session_ptr, &features)
                .context("Failed to run ONNX inference")?
        };

        Ok(decision)
    }

    /// Preprocess input features into a flat vector
    /// Original: 13 numeric + 128 embed_current + 128 embed_context = 269 features (removed fiveg_signal)
    /// Extended: 13 base + 8 file transfer + 128 embed_current + 128 embed_context = 277 features
    /// Note: Model still expects 269 features, so file transfer features are used for post-processing decisions
    fn preprocess_features(input: &AiInput, context_embed: &[f32; 128]) -> Vec<f32> {
        let mut features = Vec::with_capacity(269);
        
        // 13 numeric features (removed fiveg_signal)
        features.push(input.rtt_ms);
        features.push(input.jitter_ms);
        features.push(input.loss_rate);
        features.push(input.throughput_mbps);
        features.push(input.retransmissions);
        features.push(input.queue_p0);
        features.push(input.queue_p1);
        features.push(input.queue_p2);
        features.push(input.p0_rate);
        features.push(input.p1_rate);
        features.push(input.p2_rate);
        features.push(input.wifi_signal);
        features.push(input.starlink_latency);
        // Note: session_state, chunk_size, retries are in struct but not counted in "13 numeric"
        // File transfer features are used in post-processing logic, not in model input
        
        // 128 embedding_current
        features.extend_from_slice(&input.embed_current);
        
        // 128 embedding_context
        features.extend_from_slice(context_embed);
        
        // Total: 269 features (13 numeric + 128 + 128)
        features
    }

    /// Run ONNX model inference
    fn run_inference(session: &mut Session, features: &[f32]) -> Result<AiDecision> {
        // Create input tensor: shape (1, features.len())
        let input_array = Array2::from_shape_vec((1, features.len()), features.to_vec())
            .context("Failed to create input array")?;
        
        // Convert to ONNX Value - ort 2.0.0-rc.10 API
        // Convert to tuple format (shape, data) that ort expects
        let shape_vec: Vec<usize> = input_array.shape().iter().copied().collect();
        let data: Vec<f32> = input_array.iter().cloned().collect();
        let input_value = Value::from_array((&shape_vec[..], data))
            .context("Failed to convert array to ONNX value")?;
        
        // Run inference - ort 2.0.0-rc.10 API uses SessionInputValue
        let input: SessionInputValue = input_value.into();
        let outputs = session.run([input])
            .context("Failed to run ONNX session")?;
        
        // Parse outputs
        // Expected output shape: (1, 7)
        // Output format: [route, severity, p2_enable, congestion, wfq_p0, wfq_p1, wfq_p2]
        let output_tensor = outputs[0]
            .try_extract_tensor::<f32>()
            .context("Failed to extract output tensor")?;
        
        // ort 2.0.0-rc.10 returns (Shape, &[f32])
        let (_, output_slice) = output_tensor;
        
        // Ensure we have at least 7 values
        if output_slice.len() < 7 {
            anyhow::bail!("Model output has {} values, expected 7", output_slice.len());
        }
        
        // Parse decision outputs
        // Route: 0-2 (map sigmoid output to 0-2 range)
        let route_prob = output_slice[0];
        let route_idx = if route_prob < 0.33 {
            0 // WiFi
        } else if route_prob < 0.66 {
            1 // Starlink
        } else {
            2 // Multipath
        };
        
        // Severity: 0=High, 1=Low (threshold at 0.5)
        let severity_idx = if output_slice[1] > 0.5 { 1 } else { 0 };
        
        // Boolean outputs (threshold at 0.5)
        let p2_enable = output_slice[2] > 0.5;
        let congestion_predicted = output_slice[3] > 0.5;
        
        // Weight outputs (ensure non-negative and reasonable range)
        let wfq_p0_weight = (output_slice[4].max(0.0_f32).min(100.0_f32)) as u32;
        let wfq_p1_weight = (output_slice[5].max(0.0_f32).min(100.0_f32)) as u32;
        let wfq_p2_weight = (output_slice[6].max(0.0_f32).min(100.0_f32)) as u32;
        
        Ok(AiDecision {
            route: RouteDecision::from(route_idx),
            severity: Severity::from(severity_idx),
            p2_enable,
            congestion_predicted,
            wfq_p0_weight,
            wfq_p1_weight,
            wfq_p2_weight,
            should_send: true,  // Will be updated by process_chunk/process_file_transfer
            similarity_score: 0.0,  // Will be updated by process_chunk
            optimization_hint: OptimizationHint::SendFull,  // Will be updated by process_chunk/process_file_transfer
            network_quality: NetworkQuality {
                score: 1.0,
                is_patchy: false,
                is_connected: true,
                recommended_action: NetworkAction::Normal,
            },  // Will be updated by process_chunk/process_file_transfer
            should_buffer: false,  // Will be updated by process_chunk/process_file_transfer
            retry_strategy: RetryStrategy::Immediate,  // Will be updated by process_chunk/process_file_transfer
            // File transfer fields (defaults, will be updated by process_file_transfer)
            recommended_chunk_size: 256 * 1024,  // 256KB default
            enable_parallel_transfer: false,
            parallel_streams: 1,
            integrity_method: IntegrityMethod::None,
            verify_after_transfer: false,
            transfer_status: TransferStatus::Pending,
            estimated_completion_time: 0.0,
            transfer_action: TransferAction::Continue,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_features() {
        let input = AiInput {
            rtt_ms: 10.0,
            jitter_ms: 2.0,
            loss_rate: 0.01,
            throughput_mbps: 100.0,
            retransmissions: 5.0,
            queue_p0: 10.0,
            queue_p1: 20.0,
            queue_p2: 30.0,
            p0_rate: 1000.0,
            p1_rate: 2000.0,
            p2_rate: 3000.0,
            wifi_signal: -50.0,
            starlink_latency: 40.0,
            session_state: 0.0,
            embed_current: [0.5; 128],
            embed_context: [0.3; 128],
            chunk_size: 1024.0,
            retries: 0.0,
            // File transfer fields (defaults for telemetry)
            file_size: 0.0,
            file_type: 5.0,
            file_priority: 2.0,
            transfer_progress: 0.0,
            bytes_transferred: 0.0,
            requires_integrity_check: 0.0,
            integrity_check_failures: 0.0,
            is_resume: 0.0,
        };
        
        let context = [0.3; 128];
        let features = TelemetryAi::preprocess_features(&input, &context);
        
        assert_eq!(features.len(), 270); // 14 + 128 + 128
        assert_eq!(features[0], 10.0); // rtt_ms
    }
    
    #[test]
    fn test_file_type_from_u32() {
        assert_eq!(FileType::from(0), FileType::Media);
        assert_eq!(FileType::from(1), FileType::Document);
        assert_eq!(FileType::from(2), FileType::Data);
        assert_eq!(FileType::from(3), FileType::Archive);
        assert_eq!(FileType::from(4), FileType::Binary);
        assert_eq!(FileType::from(5), FileType::Unknown);
    }
    
    #[test]
    fn test_file_priority_from_u32() {
        assert_eq!(FilePriority::from(0), FilePriority::Critical);
        assert_eq!(FilePriority::from(1), FilePriority::High);
        assert_eq!(FilePriority::from(2), FilePriority::Normal);
        assert_eq!(FilePriority::from(3), FilePriority::Low);
    }
    
    #[test]
    fn test_transfer_status_from_u32() {
        assert_eq!(TransferStatus::from(0), TransferStatus::Pending);
        assert_eq!(TransferStatus::from(1), TransferStatus::InProgress);
        assert_eq!(TransferStatus::from(4), TransferStatus::Completed);
        assert_eq!(TransferStatus::from(5), TransferStatus::Failed);
    }
    
    #[test]
    fn test_integrity_method_from_u32() {
        assert_eq!(IntegrityMethod::from(0), IntegrityMethod::None);
        assert_eq!(IntegrityMethod::from(1), IntegrityMethod::Checksum);
        assert_eq!(IntegrityMethod::from(2), IntegrityMethod::CRC32);
        assert_eq!(IntegrityMethod::from(3), IntegrityMethod::SHA256);
    }

    #[test]
    fn test_route_decision_from_u32() {
        assert_eq!(RouteDecision::from(0), RouteDecision::WiFi);
        assert_eq!(RouteDecision::from(1), RouteDecision::Starlink);
        assert_eq!(RouteDecision::from(2), RouteDecision::Multipath);
    }

    #[test]
    fn test_severity_from_u32() {
        assert_eq!(Severity::from(0), Severity::High);
        assert_eq!(Severity::from(1), Severity::Low);
    }
    
    #[test]
    fn test_vector_store() {
        let mut store = HnswContextStore::new(128);
        
        // Insert some embeddings
        let emb1 = [0.1; 128];
        let emb2 = [0.2; 128];
        let emb3 = [0.3; 128];
        
        let id1 = store.insert(&emb1).unwrap();
        let id2 = store.insert(&emb2).unwrap();
        let id3 = store.insert(&emb3).unwrap();
        
        assert_eq!(store.len(), 3);
        assert!(!store.is_empty());
        
        // Query for nearest neighbor
        let query = [0.15; 128];
        let results = store.query(&query, 1).unwrap();
        assert_eq!(results.len(), 1);
        
        // Should find emb1 or emb2 (closest to 0.15)
        let context = store.get_context(&query).unwrap();
        assert_eq!(context.len(), 128);
    }
}
