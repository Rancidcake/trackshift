/// Integrated workflow example showing how all components work together
/// 
/// This example demonstrates:
/// 1. AI-powered telemetry analysis
/// 2. Compression when recommended
/// 3. Encryption for sensitive data
/// 4. Network-aware decision making

use trackshift::integration::*;
use trackshift::telemetry_ai::*;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("🚀 Integrated Telemetry Pipeline Example");
    println!("========================================\n");

    // Initialize the integrated pipeline with Zstd compression
    // Note: In production, provide actual ONNX model paths
    let pipeline_zstd = IntegratedTelemetryPipeline::with_compression_algorithm(
        "models/slm.onnx",
        "models/embedder.onnx",
        true,  // Enable encryption
        true,  // Enable compression
        CompressionAlgorithm::Zstd,  // Use Zstd compression
    )?;

    // Also create one with LZ4 for comparison
    let pipeline_lz4 = IntegratedTelemetryPipeline::with_compression_algorithm(
        "models/slm.onnx",
        "models/embedder.onnx",
        true,
        true,
        CompressionAlgorithm::Lz4,
    )?;

    // And one with auto-selection
    let pipeline = IntegratedTelemetryPipeline::new(
        "models/slm.onnx",
        "models/embedder.onnx",
        true,
        true,
    )?;

    println!("✅ Pipeline initialized\n");

    // Example 1: Process telemetry chunk with good network
    println!("--- Example 1: Good Network, Normal Data ---");
    let chunk1 = b"Temperature: 25.5C, Humidity: 60%, Status: OK";
    let metrics1 = NetworkMetricsInput {
        rtt_ms: 15.0,
        jitter_ms: 2.0,
        loss_rate: 0.001,
        throughput_mbps: 150.0,
        wifi_signal: -45.0,
        ..Default::default()
    };

    let processed1 = pipeline.process_chunk_full(chunk1, metrics1)?;
    print_processed_chunk(&processed1, 1);

    // Example 2: Process chunk with patchy network
    println!("\n--- Example 2: Patchy Network, Critical Data ---");
    let chunk2 = b"Alert: Fire detected! Location: Building A, Floor 3";
    let metrics2 = NetworkMetricsInput {
        rtt_ms: 500.0,
        jitter_ms: 100.0,
        loss_rate: 0.1,
        throughput_mbps: 5.0,
        wifi_signal: -80.0,
        ..Default::default()
    };

    let processed2 = pipeline.process_chunk_full(chunk2, metrics2)?;
    print_processed_chunk(&processed2, 2);

    // Example 3: Process redundant chunk
    println!("\n--- Example 3: Redundant Data (Should Skip) ---");
    let chunk3 = b"Temperature: 25.5C, Humidity: 60%, Status: OK"; // Same as chunk1
    let metrics3 = NetworkMetricsInput {
        rtt_ms: 20.0,
        jitter_ms: 3.0,
        loss_rate: 0.002,
        throughput_mbps: 100.0,
        ..Default::default()
    };

    let processed3 = pipeline.process_chunk_full(chunk3, metrics3)?;
    print_processed_chunk(&processed3, 3);

    // Example 4: Network down scenario
    println!("\n--- Example 4: Network Down (Should Buffer) ---");
    let chunk4 = b"Regular telemetry update";
    let metrics4 = NetworkMetricsInput {
        rtt_ms: 5000.0,
        jitter_ms: 1000.0,
        loss_rate: 0.5,
        throughput_mbps: 0.1,
        wifi_signal: -100.0,
        ..Default::default()
    };

    let processed4 = pipeline.process_chunk_full(chunk4, metrics4)?;
    print_processed_chunk(&processed4, 4);

    println!("\n✅ All examples completed!");
    Ok(())
}

fn print_processed_chunk(processed: &ProcessedChunk, example_num: u32) {
    println!("Example {} Decision:", example_num);
    println!("  Route: {:?}", processed.decision.route);
    println!("  Severity: {:?}", processed.decision.severity);
    println!("  Network Quality: {:.2} (Patchy: {}, Connected: {})",
        processed.decision.network_quality.score,
        processed.decision.network_quality.is_patchy,
        processed.decision.network_quality.is_connected
    );
    println!("  Should Send: {}", processed.decision.should_send);
    println!("  Similarity Score: {:.3}", processed.decision.similarity_score);
    println!("  Optimization Hint: {:?}", processed.decision.optimization_hint);
    println!("  Action: {:?}", processed.action);
    
    if let Some(ref data) = processed.processed_data {
        println!("  Processed Data Size: {} bytes", data.len());
    } else {
        println!("  Processed Data: None (skipped or buffered)");
    }
    
    if let Some(algo) = processed.compression_algorithm {
        println!("  Compression Algorithm: {:?}", algo);
    }
    
    if processed.decision.should_buffer {
        println!("  ⚠️  Data should be buffered (network too poor)");
    }
    
    if processed.decision.congestion_predicted {
        println!("  ⚠️  Congestion predicted in next 200-500ms");
    }
}

