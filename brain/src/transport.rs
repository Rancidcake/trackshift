//! Unified transport layer integrating QUIC-FEC with telemetry AI
//! 
//! This module connects:
//! - Telemetry AI decisions
//! - Compression (LZ4/Zstd)
//! - Encryption (post-quantum)
//! - QUIC-FEC transport with handover

use crate::telemetry_ai::*;
use crate::integration::{IntegratedTelemetryPipeline, ProcessedChunk};
use quic_fec::{TelemetryQuicAdapter, ConnectionConfig, FecConfig, HandoverStrategy, NetworkPath};
use anyhow::{Result, Context};
use bytes::Bytes;
use std::sync::Arc;
use parking_lot::RwLock;
use std::net::SocketAddr;

/// Unified transport system that integrates all components
pub struct UnifiedTransport {
    pipeline: IntegratedTelemetryPipeline,
    quic_adapter: Arc<RwLock<Option<TelemetryQuicAdapter>>>,
    server_addr: SocketAddr,
    server_name: String,
    connection_config: ConnectionConfig,
}

impl UnifiedTransport {
    /// Create a new unified transport system
    /// 
    /// # Arguments
    /// * `slm_model_path` - Path to SLM ONNX model
    /// * `embedder_model_path` - Path to embedder ONNX model
    /// * `server_addr` - QUIC-FEC server address
    /// * `server_name` - Server name for TLS
    /// * `encryption_enabled` - Enable encryption
    /// * `compression_enabled` - Enable compression
    pub async fn new(
        slm_model_path: &str,
        embedder_model_path: &str,
        server_addr: SocketAddr,
        server_name: &str,
        encryption_enabled: bool,
        compression_enabled: bool,
    ) -> Result<Self> {
        // Create integrated pipeline
        let pipeline = IntegratedTelemetryPipeline::new(
            slm_model_path,
            embedder_model_path,
            encryption_enabled,
            compression_enabled,
        )?;

        // Create QUIC-FEC connection config
        let connection_config = ConnectionConfig {
            fec_config: FecConfig::for_telemetry(),
            handover_strategy: HandoverStrategy::Smooth,
            initial_path: NetworkPath::WiFi,
            enable_fec: true,
            max_retransmissions: 3,
        };

        Ok(Self {
            pipeline,
            quic_adapter: Arc::new(RwLock::new(None)),
            server_addr,
            server_name: server_name.to_string(),
            connection_config,
        })
    }

    /// Connect to the QUIC-FEC server
    pub async fn connect(&self) -> Result<()> {
        let adapter = TelemetryQuicAdapter::new(
            self.server_addr,
            &self.server_name,
            self.connection_config.clone(),
        ).await
        .context("Failed to connect QUIC-FEC")?;

        *self.quic_adapter.write() = Some(adapter);
        Ok(())
    }

    /// Process and send telemetry chunk through the complete pipeline
    /// 
    /// This method:
    /// 1. Analyzes chunk with AI
    /// 2. Applies compression if recommended
    /// 3. Applies encryption if needed
    /// 4. Sends via QUIC-FEC with automatic handover
    /// 5. Updates network metrics for handover decisions
    pub async fn process_and_send(
        &self,
        chunk_data: &[u8],
        network_metrics: NetworkMetricsInput,
    ) -> Result<AiDecision> {
        // Step 1: Process through AI pipeline
        let processed = self.pipeline.process_chunk_full(chunk_data, network_metrics.clone())?;
        
        // Step 2: Update QUIC-FEC with network metrics for handover
        self.update_network_metrics(&network_metrics).await?;

        // Step 3: Check and perform handover if needed
        self.check_handover().await?;

        // Step 4: Send data if we should
        if let Some(data) = processed.processed_data {
            match processed.action {
                crate::integration::ProcessAction::Skip => {
                    // Skip sending (redundant data)
                }
                crate::integration::ProcessAction::Buffer => {
                    // Buffer for later (network down)
                    // In a real implementation, you'd use TelemetryBuffer here
                }
                _ => {
                    // Send via QUIC-FEC
                    self.send_data(Bytes::from(data)).await?;
                }
            }
        }

        Ok(processed.decision)
    }

    /// Send data via QUIC-FEC
    async fn send_data(&self, data: Bytes) -> Result<()> {
        let adapter = self.quic_adapter.read();
        let adapter = adapter.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to QUIC-FEC server"))?;
        
        adapter.send_telemetry(data).await
            .context("Failed to send data via QUIC-FEC")
    }

    /// Update network metrics for handover decisions
    async fn update_network_metrics(&self, metrics: &NetworkMetricsInput) -> Result<()> {
        let adapter = self.quic_adapter.read();
        if let Some(adapter) = adapter.as_ref() {
            // Extract 5G signal if available (currently not in NetworkMetricsInput)
            // For now, use WiFi signal as placeholder
            adapter.update_network_metrics(
                metrics.wifi_signal,
                None, // 5G signal - would need to add to NetworkMetricsInput
                Some(metrics.starlink_latency), // Starlink latency
                metrics.rtt_ms,
                metrics.jitter_ms,
                metrics.loss_rate,
                metrics.throughput_mbps,
            );
        }
        Ok(())
    }

    /// Check and perform handover if needed
    async fn check_handover(&self) -> Result<bool> {
        let adapter = self.quic_adapter.read();
        if let Some(adapter) = adapter.as_ref() {
            adapter.check_and_handover().await
        } else {
            Ok(false)
        }
    }

    /// Receive data from QUIC-FEC
    pub async fn receive(&self) -> Result<Option<Bytes>> {
        let adapter = self.quic_adapter.read();
        if let Some(adapter) = adapter.as_ref() {
            adapter.recv_telemetry().await
        } else {
            Ok(None)
        }
    }

    /// Get current network path
    pub fn current_path(&self) -> Option<NetworkPath> {
        let adapter = self.quic_adapter.read();
        adapter.as_ref().map(|a| a.current_path())
    }

    /// Get reference to AI pipeline
    pub fn pipeline(&self) -> &IntegratedTelemetryPipeline {
        &self.pipeline
    }

    /// Update FEC configuration based on network quality
    pub fn update_fec_config(&mut self, network_quality_score: f32) {
        self.connection_config.fec_config = if network_quality_score < 0.3 {
            FecConfig::for_patchy_network() // High redundancy for poor networks
        } else if network_quality_score < 0.6 {
            FecConfig::for_telemetry() // Standard telemetry config
        } else {
            FecConfig::default() // Lower redundancy for good networks
        };
    }
}

/// Helper function to convert RouteDecision to NetworkPath
impl From<RouteDecision> for NetworkPath {
    fn from(route: RouteDecision) -> Self {
        match route {
            RouteDecision::WiFi => NetworkPath::WiFi,
            RouteDecision::Starlink => NetworkPath::Starlink,
            RouteDecision::Multipath => NetworkPath::Multipath,
            RouteDecision::FiveG => NetworkPath::FiveG,
        }
    }
}

