pub mod telemetry_ai;
pub mod integration;
pub mod transport;

// Re-export main types for convenience
pub use telemetry_ai::{
    TelemetryAi, AiInput, AiDecision, NetworkMetricsInput,
    RouteDecision, Severity, OptimizationHint, RetryStrategy,
    PriorityTagger, ChunkPriority, DataFormat, DataScenario,
    PriorityScheduler, ScheduledChunk, SchedulerStats,
    RealtimeStatusMonitor, TransferStatusInfo, NetworkStatus, SystemHealth,
    StatusSnapshot, IntegrityCheckStatus, IntegrityMethod, TransferStatus,
    TransferAction, NetworkQuality,
};

// Re-export integration types
pub use integration::{
    IntegratedTelemetryPipeline, ProcessedChunk, ProcessAction, CompressionAlgorithm,
    compress_data, decompress_data,
};

// Re-export transport types
pub use transport::UnifiedTransport;

