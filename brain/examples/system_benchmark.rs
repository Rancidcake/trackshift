//! Comprehensive System Benchmark
//!
//! Run with: cargo run --example system_benchmark

use trackshift::telemetry_ai::{
    TelemetryAi, NetworkMetricsInput, RouteDecision,
};
use std::time::{Instant, Duration};
use anyhow::Result;

/// Test result for a single test case
#[derive(Debug, Clone)]
struct TestResult {
    format: String,
    network_condition: String,
    route: RouteDecision,
    latency_ms: f64,
    throughput_mbps: f64,
    loss_rate: f32,
    jitter_ms: f32,
    bytes_sent: usize,
    bytes_received: usize,
    compression_ratio: f32,
    success: bool,
    error_message: Option<String>,
}

/// Network condition presets
struct NetworkConditions;

impl NetworkConditions {
    fn excellent() -> NetworkMetricsInput {
        NetworkMetricsInput {
            rtt_ms: 10.0,
            jitter_ms: 1.0,
            loss_rate: 0.0001,
            throughput_mbps: 1000.0,
            retransmissions: 0.0,
            queue_p0: 0.0,
            queue_p1: 0.0,
            queue_p2: 0.0,
            p0_rate: 100.0,
            p1_rate: 0.0,
            p2_rate: 0.0,
            wifi_signal: -30.0,
            starlink_latency: 20.0,
            session_state: 0.0,
            retries: 0.0,
        }
    }

    fn good() -> NetworkMetricsInput {
        NetworkMetricsInput {
            rtt_ms: 30.0,
            jitter_ms: 5.0,
            loss_rate: 0.001,
            throughput_mbps: 100.0,
            retransmissions: 0.1,
            queue_p0: 10.0,
            queue_p1: 5.0,
            queue_p2: 2.0,
            p0_rate: 80.0,
            p1_rate: 15.0,
            p2_rate: 5.0,
            wifi_signal: -60.0,
            starlink_latency: 40.0,
            session_state: 0.0,
            retries: 0.0,
        }
    }

    fn patchy() -> NetworkMetricsInput {
        NetworkMetricsInput {
            rtt_ms: 200.0,
            jitter_ms: 50.0,
            loss_rate: 0.05,
            throughput_mbps: 10.0,
            retransmissions: 5.0,
            queue_p0: 50.0,
            queue_p1: 30.0,
            queue_p2: 20.0,
            p0_rate: 60.0,
            p1_rate: 30.0,
            p2_rate: 10.0,
            wifi_signal: -85.0,
            starlink_latency: 300.0,
            session_state: 0.5,
            retries: 2.0,
        }
    }

    fn bad() -> NetworkMetricsInput {
        NetworkMetricsInput {
            rtt_ms: 500.0,
            jitter_ms: 100.0,
            loss_rate: 0.15,
            throughput_mbps: 2.0,
            retransmissions: 20.0,
            queue_p0: 80.0,
            queue_p1: 60.0,
            queue_p2: 40.0,
            p0_rate: 40.0,
            p1_rate: 40.0,
            p2_rate: 20.0,
            wifi_signal: -95.0,
            starlink_latency: 800.0,
            session_state: 1.0,
            retries: 10.0,
        }
    }
}

/// Test data generators
struct TestData;

impl TestData {
    fn json_small() -> Vec<u8> {
        r#"{"sensor_id": "temp_01", "value": 23.5, "timestamp": 1234567890}"#.as_bytes().to_vec()
    }

    fn json_large() -> Vec<u8> {
        let mut data = r#"{"sensors": ["#.to_string();
        for i in 0..1000 {
            data.push_str(&format!(r#"{{"id": "sensor_{}", "value": {:.2}, "timestamp": {}}}, "#, i, i as f32 * 0.1, 1234567890 + i));
        }
        data.push_str(r#"]}"#);
        data.into_bytes()
    }

    fn binary_random(size: usize) -> Vec<u8> {
        (0..size).map(|i| (i % 256) as u8).collect()
    }

    fn text_log() -> Vec<u8> {
        r#"2024-01-01 10:00:00 INFO: System started
2024-01-01 10:00:01 DEBUG: Initializing components
2024-01-01 10:00:02 WARN: Low memory detected
2024-01-01 10:00:03 ERROR: Connection failed
2024-01-01 10:00:04 INFO: Retrying connection
"#.as_bytes().to_vec()
    }

    fn image_png() -> Vec<u8> {
        let mut data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x0D]);
        data.extend_from_slice(b"IHDR");
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x64]);
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x64]);
        data.extend_from_slice(&[0x08, 0x02, 0x00, 0x00, 0x00]);
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        data.extend_from_slice(b"IEND");
        data.extend_from_slice(&[0xAE, 0x42, 0x60, 0x82]);
        data
    }

    fn csv_data() -> Vec<u8> {
        r#"timestamp,sensor_id,value,unit
1234567890,temp_01,23.5,C
1234567891,humidity_01,65.2,%
1234567892,pressure_01,1013.25,hPa
"#.as_bytes().to_vec()
    }
}

/// Simulate a test case (when models not available)
fn simulate_test_case(
    format_name: &str,
    data: &[u8],
    network_condition: &str,
    metrics: &NetworkMetricsInput,
) -> TestResult {
    let bytes_sent = data.len();
    
    // Simulate processing latency based on data size and network
    let base_latency = (data.len() as f64 / 1024.0) * 0.1; // 0.1ms per KB
    let network_latency = metrics.rtt_ms as f64;
    let processing_latency = base_latency + network_latency;
    
    // Simulate transmission time
    let transmission_time = (bytes_sent as f64 / (metrics.throughput_mbps as f64 * 1024.0 * 1024.0 / 8.0)) * 1000.0;
    let total_latency = processing_latency + transmission_time;
    
    // Simulate loss
    let bytes_lost = (bytes_sent as f32 * metrics.loss_rate) as usize;
    let bytes_received = bytes_sent - bytes_lost;
    
    // Determine route based on network conditions
    let route = if metrics.throughput_mbps > 500.0 && metrics.rtt_ms < 30.0 {
        RouteDecision::FiveG
    } else if metrics.throughput_mbps > 100.0 && metrics.rtt_ms < 50.0 {
        RouteDecision::Multipath
    } else if metrics.rtt_ms < 100.0 {
        RouteDecision::WiFi
    } else {
        RouteDecision::Starlink
    };
    
    // Estimate compression ratio
    let compression_ratio = match format_name {
        s if s.contains("JSON") => 0.7,
        s if s.contains("Text") => 0.6,
        s if s.contains("CSV") => 0.5,
        _ => 1.0,
    };
    
    TestResult {
        format: format_name.to_string(),
        network_condition: network_condition.to_string(),
        route,
        latency_ms: total_latency,
        throughput_mbps: metrics.throughput_mbps as f64,
        loss_rate: metrics.loss_rate,
        jitter_ms: metrics.jitter_ms,
        bytes_sent,
        bytes_received,
        compression_ratio,
        success: true,
        error_message: None,
    }
}

/// Run a single test case
async fn run_test_case(
    ai_system: &TelemetryAi,
    format_name: &str,
    data: &[u8],
    network_condition: &str,
    metrics: &NetworkMetricsInput,
) -> TestResult {
    let start = Instant::now();
    let bytes_sent = data.len();
    
    let result = ai_system.process_chunk(data, metrics.clone());
    
    let latency = start.elapsed();
    let latency_ms = latency.as_secs_f64() * 1000.0;
    
    match result {
        Ok(decision) => {
            let transmission_time = (bytes_sent as f64 / (metrics.throughput_mbps as f64 * 1024.0 * 1024.0 / 8.0)) * 1000.0;
            let total_latency = latency_ms + transmission_time;
            
            let bytes_lost = (bytes_sent as f32 * metrics.loss_rate) as usize;
            let bytes_received = bytes_sent - bytes_lost;
            
            let compression_ratio = match decision.optimization_hint {
                trackshift::telemetry_ai::OptimizationHint::Compress => 0.5,
                trackshift::telemetry_ai::OptimizationHint::SendDelta => 0.3,
                _ => 1.0,
            };
            
            TestResult {
                format: format_name.to_string(),
                network_condition: network_condition.to_string(),
                route: decision.route,
                latency_ms: total_latency,
                throughput_mbps: metrics.throughput_mbps as f64,
                loss_rate: metrics.loss_rate,
                jitter_ms: metrics.jitter_ms,
                bytes_sent,
                bytes_received,
                compression_ratio,
                success: true,
                error_message: None,
            }
        }
        Err(e) => {
            TestResult {
                format: format_name.to_string(),
                network_condition: network_condition.to_string(),
                route: RouteDecision::WiFi,
                latency_ms,
                throughput_mbps: 0.0,
                loss_rate: metrics.loss_rate,
                jitter_ms: metrics.jitter_ms,
                bytes_sent,
                bytes_received: 0,
                compression_ratio: 1.0,
                success: false,
                error_message: Some(e.to_string()),
            }
        }
    }
}

/// Print results matrix
fn print_results_matrix(results: &[TestResult]) {
    println!("\n╔════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                                    SYSTEM BENCHMARK RESULTS MATRIX                                              ║");
    println!("╠════════════════════════════════════════════════════════════════════════════════════════════════════════════════╣");
    println!("║ {:<12} │ {:<12} │ {:<10} │ {:<10} │ {:<10} │ {:<8} │ {:<8} │ {:<10} │ {:<10} │ {:<6} ║",
             "Format", "Network", "Route", "Latency(ms)", "Throughput", "Loss(%)", "Jitter", "Bytes Sent", "Bytes Recv", "Success");
    println!("╠════════════════════════════════════════════════════════════════════════════════════════════════════════════════╣");
    
    for result in results {
        let route_str = match result.route {
            RouteDecision::WiFi => "WiFi",
            RouteDecision::Starlink => "Starlink",
            RouteDecision::Multipath => "Multipath",
            RouteDecision::FiveG => "5G",
        };
        
        let success_str = if result.success { "✓" } else { "✗" };
        
        println!("║ {:<12} │ {:<12} │ {:<10} │ {:<10.2} │ {:<10.2} │ {:<8.2} │ {:<8.2} │ {:<10} │ {:<10} │ {:<6} ║",
                 result.format,
                 result.network_condition,
                 route_str,
                 result.latency_ms,
                 result.throughput_mbps,
                 result.loss_rate * 100.0,
                 result.jitter_ms,
                 result.bytes_sent,
                 result.bytes_received,
                 success_str);
    }
    
    println!("╚════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝");
}

/// Print summary statistics
fn print_summary_statistics(results: &[TestResult]) {
    let successful: Vec<&TestResult> = results.iter().filter(|r| r.success).collect();
    let failed: Vec<&TestResult> = results.iter().filter(|r| !r.success).collect();
    
    if successful.is_empty() {
        println!("\n⚠️  No successful tests to analyze");
        return;
    }
    
    let avg_latency: f64 = successful.iter().map(|r| r.latency_ms).sum::<f64>() / successful.len() as f64;
    let min_latency = successful.iter().map(|r| r.latency_ms).fold(f64::INFINITY, f64::min);
    let max_latency = successful.iter().map(|r| r.latency_ms).fold(0.0, f64::max);
    
    let avg_throughput: f64 = successful.iter().map(|r| r.throughput_mbps).sum::<f64>() / successful.len() as f64;
    let avg_loss: f32 = successful.iter().map(|r| r.loss_rate).sum::<f32>() / successful.len() as f32;
    let avg_compression: f32 = successful.iter().map(|r| r.compression_ratio).sum::<f32>() / successful.len() as f32;
    
    println!("\n╔════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                                          SUMMARY STATISTICS                                                      ║");
    println!("╠════════════════════════════════════════════════════════════════════════════════════════════════════════════════╣");
    println!("║ Total Tests:        {:>6}                                                                                      ║", results.len());
    println!("║ Successful:         {:>6} ({:.1}%)                                                                              ║", 
             successful.len(), (successful.len() as f32 / results.len() as f32) * 100.0);
    println!("║ Failed:             {:>6} ({:.1}%)                                                                              ║",
             failed.len(), (failed.len() as f32 / results.len() as f32) * 100.0);
    println!("╠════════════════════════════════════════════════════════════════════════════════════════════════════════════════╣");
    println!("║ Latency (ms):                                                                                                   ║");
    println!("║   Average:           {:>10.2}                                                                                      ║", avg_latency);
    println!("║   Minimum:          {:>10.2}                                                                                      ║", min_latency);
    println!("║   Maximum:          {:>10.2}                                                                                      ║", max_latency);
    println!("╠════════════════════════════════════════════════════════════════════════════════════════════════════════════════╣");
    println!("║ Throughput (Mbps):                                                                                              ║");
    println!("║   Average:          {:>10.2}                                                                                      ║", avg_throughput);
    println!("╠════════════════════════════════════════════════════════════════════════════════════════════════════════════════╣");
    println!("║ Loss Rate:          {:>10.2}%                                                                                    ║", avg_loss * 100.0);
    println!("║ Compression Ratio:  {:>10.2}                                                                                      ║", avg_compression);
    println!("╚════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝");
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Comprehensive System Benchmark");
    println!("==========================================\n");
    
    // Try to initialize AI system
    // Note: If models don't exist, we'll simulate the benchmark with expected results
    let ai_system_result = TelemetryAi::new("models/slm.onnx", "models/embedder.onnx");
    
    let use_mock = ai_system_result.is_err();
    
    if use_mock {
        println!("⚠️  Warning: ONNX models not found.");
        println!("   Running benchmark with simulated results based on network conditions...");
        println!("   (Install models for actual AI-based routing decisions)\n");
    } else {
        println!("✅ AI system initialized with models\n");
    }
    
    let test_cases = vec![
        ("JSON Small", TestData::json_small()),
        ("JSON Large", TestData::json_large()),
        ("Binary 1KB", TestData::binary_random(1024)),
        ("Binary 10KB", TestData::binary_random(10 * 1024)),
        ("Binary 100KB", TestData::binary_random(100 * 1024)),
        ("Text Log", TestData::text_log()),
        ("Image PNG", TestData::image_png()),
        ("CSV Data", TestData::csv_data()),
    ];
    
    let network_conditions = vec![
        ("Excellent", NetworkConditions::excellent()),
        ("Good", NetworkConditions::good()),
        ("Patchy", NetworkConditions::patchy()),
        ("Bad", NetworkConditions::bad()),
    ];
    
    println!("📋 Test Configuration:");
    println!("   Formats: {}", test_cases.len());
    println!("   Network Conditions: {}", network_conditions.len());
    println!("   Total Test Cases: {}\n", test_cases.len() * network_conditions.len());
    
    let mut results = Vec::new();
    
    if use_mock {
        // Generate simulated results based on network conditions
        for (format_name, data) in &test_cases {
            for (network_name, network_metrics) in &network_conditions {
                print!("Simulating {} on {} network... ", format_name, network_name);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                
                let result = simulate_test_case(format_name, data, network_name, network_metrics);
                println!("✓");
                results.push(result);
                
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        }
    } else {
        let ai_system = ai_system_result.unwrap();
        for (format_name, data) in &test_cases {
            for (network_name, network_metrics) in &network_conditions {
                print!("Testing {} on {} network... ", format_name, network_name);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                
                let result = run_test_case(&ai_system, format_name, data, network_name, network_metrics).await;
                
                if result.success {
                    println!("✓");
                } else {
                    println!("✗");
                }
                
                results.push(result);
                
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
    
    print_results_matrix(&results);
    print_summary_statistics(&results);
    
    println!("\n✅ Benchmark Complete!");
    
    Ok(())
}

