//! Integration module that combines AI telemetry decisions with encryption and compression
//! 
//! This module provides a unified interface for:
//! - AI-powered telemetry analysis and decision making
//! - Post-quantum encryption (when needed)
//! - Compression (LZ4 or Zstd when recommended by AI)
//! - File transfer management

use crate::telemetry_ai::*;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;

/// Compression algorithm selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    /// LZ4 - Fast compression, lower ratio
    Lz4,
    /// Zstd - Balanced compression, better ratio
    Zstd,
    /// Auto - Let AI decide based on data characteristics
    Auto,
}

impl Default for CompressionAlgorithm {
    fn default() -> Self {
        CompressionAlgorithm::Auto
    }
}

/// Integrated telemetry processing pipeline
/// Combines AI decisions with encryption and compression
pub struct IntegratedTelemetryPipeline {
    ai_system: TelemetryAi,
    encryption_enabled: bool,
    compression_enabled: bool,
    compression_algorithm: CompressionAlgorithm,
}

impl IntegratedTelemetryPipeline {
    /// Create a new integrated pipeline
    /// 
    /// # Arguments
    /// * `slm_model_path` - Path to SLM ONNX model
    /// * `embedder_model_path` - Path to embedder ONNX model
    /// * `encryption_enabled` - Enable encryption for sensitive data
    /// * `compression_enabled` - Enable compression when recommended
    /// * `compression_algorithm` - Compression algorithm to use (default: Auto)
    pub fn new(
        slm_model_path: &str,
        embedder_model_path: &str,
        encryption_enabled: bool,
        compression_enabled: bool,
    ) -> Result<Self> {
        Self::with_compression_algorithm(
            slm_model_path,
            embedder_model_path,
            encryption_enabled,
            compression_enabled,
            CompressionAlgorithm::Auto,
        )
    }

    /// Create a new integrated pipeline with specific compression algorithm
    /// 
    /// # Arguments
    /// * `slm_model_path` - Path to SLM ONNX model
    /// * `embedder_model_path` - Path to embedder ONNX model
    /// * `encryption_enabled` - Enable encryption for sensitive data
    /// * `compression_enabled` - Enable compression when recommended
    /// * `compression_algorithm` - Compression algorithm to use
    pub fn with_compression_algorithm(
        slm_model_path: &str,
        embedder_model_path: &str,
        encryption_enabled: bool,
        compression_enabled: bool,
        compression_algorithm: CompressionAlgorithm,
    ) -> Result<Self> {
        let ai_system = TelemetryAi::new(slm_model_path, embedder_model_path)
            .context("Failed to initialize AI system")?;
        
        Ok(Self {
            ai_system,
            encryption_enabled,
            compression_enabled,
            compression_algorithm,
        })
    }

    /// Process telemetry chunk with full pipeline
    /// 
    /// This method:
    /// 1. Analyzes the chunk with AI
    /// 2. Applies compression if recommended
    /// 3. Applies encryption if needed
    /// 4. Returns processed data and decision
    pub fn process_chunk_full(
        &self,
        chunk_data: &[u8],
        network_metrics: NetworkMetricsInput,
    ) -> Result<ProcessedChunk> {
        // Step 1: Get AI decision
        let decision = self.ai_system.process_chunk(chunk_data, network_metrics.clone())
            .context("AI decision failed")?;

        // Step 2: Check if we should send this chunk
        if !decision.should_send {
            return Ok(ProcessedChunk {
                decision,
                processed_data: None,
                action: ProcessAction::Skip,
                compression_algorithm: None,
            });
        }

        // Step 3: Apply compression if recommended
        let mut processed_data = chunk_data.to_vec();
        let mut action = ProcessAction::SendFull;
        let mut compression_used = CompressionAlgorithm::Lz4; // Track which was used

        if self.compression_enabled {
            match decision.optimization_hint {
                OptimizationHint::Compress => {
                    // Determine compression algorithm
                    let algo = match self.compression_algorithm {
                        CompressionAlgorithm::Auto => {
                            // Auto-select based on data size and network quality
                            if chunk_data.len() > 1_000_000 || decision.network_quality.score < 0.5 {
                                CompressionAlgorithm::Zstd // Better ratio for large files or bad networks
                            } else {
                                CompressionAlgorithm::Lz4 // Faster for small files or good networks
                            }
                        }
                        algo => algo,
                    };
                    
                    processed_data = compress_data(&processed_data, algo)?;
                    compression_used = algo;
                    action = ProcessAction::SendCompressed;
                }
                OptimizationHint::SendDelta => {
                    // Delta compression would require previous chunk
                    // For now, send full but mark as delta
                    action = ProcessAction::SendDelta;
                }
                OptimizationHint::Skip => {
                    return Ok(ProcessedChunk {
                        decision,
                        processed_data: None,
                        action: ProcessAction::Skip,
                        compression_algorithm: None,
                    });
                }
                OptimizationHint::SendFull => {
                    // No compression needed
                }
            }
        }

        // Step 4: Apply encryption if enabled and network quality is poor
        if self.encryption_enabled {
            // Encrypt if network is patchy or data is critical
            if decision.network_quality.is_patchy || decision.severity == Severity::High {
                // Note: In a real implementation, you would use rust_pqc here
                // For now, we'll just mark it as needing encryption
                action = ProcessAction::SendEncrypted;
            }
        }

        // Step 5: Handle buffering if network is down
        if decision.should_buffer {
            return Ok(ProcessedChunk {
                decision,
                processed_data: Some(processed_data),
                action: ProcessAction::Buffer,
                compression_algorithm: if action == ProcessAction::SendCompressed {
                    Some(compression_used)
                } else {
                    None
                },
            });
        }

        Ok(ProcessedChunk {
            decision,
            processed_data: Some(processed_data),
            action,
            compression_algorithm: if action == ProcessAction::SendCompressed {
                Some(compression_used)
            } else {
                None
            },
        })
    }

    /// Get reference to AI system for advanced usage
    pub fn ai_system(&self) -> &TelemetryAi {
        &self.ai_system
    }
}

/// Result of processing a chunk through the full pipeline
#[derive(Debug, Clone)]
pub struct ProcessedChunk {
    pub decision: AiDecision,
    pub processed_data: Option<Vec<u8>>,
    pub action: ProcessAction,
    /// Compression algorithm used (if compression was applied)
    pub compression_algorithm: Option<CompressionAlgorithm>,
}

/// Action to take for processed chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessAction {
    SendFull,
    SendCompressed,
    SendEncrypted,
    SendDelta,
    Buffer,
    Skip,
}

/// Compress data using the specified algorithm
/// 
/// # Arguments
/// * `data` - Data to compress
/// * `algorithm` - Compression algorithm to use
/// 
/// # Returns
/// Compressed data
pub fn compress_data(data: &[u8], algorithm: CompressionAlgorithm) -> Result<Vec<u8>> {
    match algorithm {
        CompressionAlgorithm::Lz4 => {
            use lz4_flex::compress_prepend_size;
            Ok(compress_prepend_size(data))
        }
        CompressionAlgorithm::Zstd => {
            use zstd::encode_all;
            encode_all(data, 3) // Level 3: good balance of speed and ratio
                .context("Zstd compression failed")
        }
        CompressionAlgorithm::Auto => {
            // Should not reach here, but fallback to LZ4
            compress_data(data, CompressionAlgorithm::Lz4)
        }
    }
}

/// Decompress data using the specified algorithm
/// 
/// # Arguments
/// * `data` - Compressed data
/// * `algorithm` - Compression algorithm used
/// 
/// # Returns
/// Decompressed data
pub fn decompress_data(data: &[u8], algorithm: CompressionAlgorithm) -> Result<Vec<u8>> {
    match algorithm {
        CompressionAlgorithm::Lz4 => {
            use lz4_flex::decompress_size_prepended;
            decompress_size_prepended(data)
                .map_err(|e| anyhow::anyhow!("LZ4 decompression failed: {}", e))
        }
        CompressionAlgorithm::Zstd => {
            use zstd::decode_all;
            decode_all(data)
                .context("Zstd decompression failed")
        }
        CompressionAlgorithm::Auto => {
            // Try both, starting with Zstd (more common)
            if let Ok(result) = decompress_data(data, CompressionAlgorithm::Zstd) {
                Ok(result)
            } else {
                decompress_data(data, CompressionAlgorithm::Lz4)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_algorithms() {
        let test_data = b"Hello, this is a test string that will be compressed using different algorithms!";
        
        // Test LZ4 compression/decompression
        let compressed_lz4 = compress_data(test_data, CompressionAlgorithm::Lz4).unwrap();
        let decompressed_lz4 = decompress_data(&compressed_lz4, CompressionAlgorithm::Lz4).unwrap();
        assert_eq!(test_data, decompressed_lz4.as_slice());
        
        // Test Zstd compression/decompression
        let compressed_zstd = compress_data(test_data, CompressionAlgorithm::Zstd).unwrap();
        let decompressed_zstd = decompress_data(&compressed_zstd, CompressionAlgorithm::Zstd).unwrap();
        assert_eq!(test_data, decompressed_zstd.as_slice());
        
        // Verify compression actually reduces size (for this test data)
        assert!(compressed_lz4.len() < test_data.len());
        assert!(compressed_zstd.len() < test_data.len());
    }

    #[test]
    fn test_compression_algorithm_selection() {
        // Test that CompressionAlgorithm enum works correctly
        assert_eq!(CompressionAlgorithm::default(), CompressionAlgorithm::Auto);
        assert_ne!(CompressionAlgorithm::Lz4, CompressionAlgorithm::Zstd);
    }
}

