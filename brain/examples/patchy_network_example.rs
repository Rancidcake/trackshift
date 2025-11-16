/// Example: Handling patchy networks with adaptive strategies
/// 
/// This demonstrates:
/// 1. Network quality assessment
/// 2. Adaptive redundancy detection
/// 3. Buffering during outages
/// 4. Smart retry strategies
/// 5. Compression on bad networks

use trackshift::telemetry_ai::*;
use trackshift::{TelemetryBuffer, NetworkMetricsInput};

fn main() -> anyhow::Result<()> {
    println!("🌐 Patchy Network Handling Example\n");

    // Initialize AI system
    let ai = TelemetryAi::new("models/slm.onnx", "models/embedder.onnx")?;
    
    // Initialize buffer for outages
    let buffer = TelemetryBuffer::new(1000, 3600);  // Max 1000 chunks, 1 hour max age

    // Simulate different network conditions
    let scenarios = vec![
        ("Good Network", create_good_network_metrics()),
        ("Patchy Network", create_patchy_network_metrics()),
        ("Network Down", create_network_down_metrics()),
        ("Emergency", create_emergency_network_metrics()),
    ];

    for (scenario_name, metrics) in scenarios {
        println!("=== {} ===", scenario_name);
        
        let chunk_data = b"Temperature: 25°C, Status: OK";
        let decision = ai.process_chunk(chunk_data, metrics)?;
        
        println!("Network Quality:");
        println!("  Score: {:.2} ({}%)", decision.network_quality.score, decision.network_quality.score * 100.0);
        println!("  Patchy: {}", decision.network_quality.is_patchy);
        println!("  Connected: {}", decision.network_quality.is_connected);
        println!("  Action: {:?}", decision.network_quality.recommended_action);
        
        println!("\nDecision:");
        println!("  Should Send: {}", decision.should_send);
        println!("  Should Buffer: {}", decision.should_buffer);
        println!("  Similarity: {:.2}%", decision.similarity_score * 100.0);
        println!("  Optimization: {:?}", decision.optimization_hint);
        println!("  Retry Strategy: {:?}", decision.retry_strategy);
        println!("  Route: {:?}", decision.route);
        
        // Handle based on network quality
        if decision.should_buffer {
            println!("\n📦 BUFFERING: Network too bad, buffering data...");
            let priority = if decision.severity == Severity::High { 0 } else { 128 };
            buffer.add(chunk_data.to_vec(), priority)?;
            println!("   Buffer status: {}/{} chunks", buffer.status().size, buffer.status().max_size);
        } else if decision.network_quality.is_patchy {
            println!("\n⚠️  PATCHY NETWORK: Using aggressive optimization...");
            match decision.optimization_hint {
                OptimizationHint::Compress => {
                    println!("   → Compressing data before sending");
                }
                OptimizationHint::SendDelta => {
                    println!("   → Sending only delta/changes");
                }
                OptimizationHint::Skip => {
                    println!("   → Skipping redundant data");
                }
                _ => {
                    println!("   → Sending full data");
                }
            }
        } else {
            println!("\n✅ GOOD NETWORK: Normal operation");
        }
        
        println!();
    }

    // Demonstrate buffer flushing when network recovers
    println!("=== Buffer Flush (Network Recovered) ===");
    let good_metrics = create_good_network_metrics();
    let decision = ai.process_chunk(b"test", good_metrics)?;
    
    if !decision.should_buffer && decision.network_quality.is_connected {
        println!("Network recovered! Flushing buffer...");
        let mut flushed = 0;
        while let Some(buffered) = buffer.pop() {
            println!("  Sending buffered chunk (priority: {}, age: {}s)", 
                     buffered.priority,
                     std::time::SystemTime::now()
                         .duration_since(std::time::UNIX_EPOCH)
                         .unwrap()
                         .as_secs() - buffered.timestamp);
            flushed += 1;
            if flushed >= 5 {
                println!("  ... (showing first 5)");
                break;
            }
        }
        println!("Flushed {} chunks", flushed);
    }

    Ok(())
}

fn create_good_network_metrics() -> NetworkMetricsInput {
    NetworkMetricsInput {
        rtt_ms: 15.0,
        jitter_ms: 2.0,
        loss_rate: 0.001,
        throughput_mbps: 100.0,
        retransmissions: 0.0,
        wifi_signal: -45.0,
        ..Default::default()
    }
}

fn create_patchy_network_metrics() -> NetworkMetricsInput {
    NetworkMetricsInput {
        rtt_ms: 250.0,        // High latency
        jitter_ms: 30.0,      // High jitter
        loss_rate: 0.08,      // 8% packet loss
        throughput_mbps: 2.0, // Low throughput
        retransmissions: 8.0, // Many retransmissions
        wifi_signal: -85.0,   // Weak signal
        session_state: 0.0,   // Still connected
        ..Default::default()
    }
}

fn create_network_down_metrics() -> NetworkMetricsInput {
    NetworkMetricsInput {
        rtt_ms: 1000.0,       // Very high (timeout)
        jitter_ms: 100.0,     // Extreme jitter
        loss_rate: 0.5,       // 50% loss
        throughput_mbps: 0.0, // No throughput
        retransmissions: 20.0, // Many retries
        wifi_signal: -100.0,  // No signal
        session_state: 1.0,   // Connection broken
        ..Default::default()
    }
}

fn create_emergency_network_metrics() -> NetworkMetricsInput {
    NetworkMetricsInput {
        rtt_ms: 800.0,
        jitter_ms: 80.0,
        loss_rate: 0.15,      // 15% loss
        throughput_mbps: 0.5, // Very slow
        retransmissions: 15.0,
        wifi_signal: -95.0,
        session_state: 0.5,   // Intermittent
        ..Default::default()
    }
}

