/// Example demonstrating priority tagger and scheduler integration
/// 
/// This example shows:
/// 1. Automatic priority tagging of telemetry chunks
/// 2. Priority-based scheduling
/// 3. WFQ weight management
/// 4. Integration with AI decisions

use trackshift::telemetry_ai::*;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    println!("🚀 Priority Tagger & Scheduler Example");
    println!("======================================\n");

    // Initialize components
    let ai_system = TelemetryAi::new("models/slm.onnx", "models/embedder.onnx")?;
    let priority_tagger = Arc::new(PriorityTagger::new());
    let scheduler = Arc::new(PriorityScheduler::new());

    println!("✅ Components initialized\n");

    // Example telemetry chunks with different content
    let chunks = vec![
        (b"Alert: Fire detected in building A, Floor 3".to_vec(), "Critical Alert"),
        (b"Warning: High CPU usage detected - 95%".to_vec(), "High Priority Warning"),
        (b"Temperature: 25.5C, Humidity: 60%, Status: OK".to_vec(), "Normal Telemetry"),
        (b"Status: OK, Normal operation, Info log entry".to_vec(), "Low Priority Status"),
        (b"Bulk metrics: [1000 data points...]".to_vec(), "Bulk Data"),
        (b"Critical: System failure detected".to_vec(), "Critical Error"),
    ];

    // Collect network metrics
    let network_metrics = NetworkMetricsInput {
        rtt_ms: 20.0,
        jitter_ms: 3.0,
        loss_rate: 0.001,
        throughput_mbps: 100.0,
        wifi_signal: -50.0,
        ..Default::default()
    };

    println!("--- Step 1: Tagging Priority ---\n");
    
    // Process each chunk: tag priority and schedule
    for (chunk_data, description) in &chunks {
        // Get AI decision
        let decision = ai_system.process_chunk(chunk_data, network_metrics.clone())?;
        
        // Tag priority automatically
        let priority = priority_tagger.tag_priority(chunk_data, Some(decision.severity));
        
        println!("Chunk: {}", description);
        println!("  AI Severity: {:?}", decision.severity);
        println!("  Tagged Priority: {:?} ({})", priority, priority as u8);
        println!("  Route: {:?}", decision.route);
        
        // Schedule chunk with priority
        scheduler.schedule(
            chunk_data.clone(),
            priority,
            Some(decision.route as u32),
        )?;
        
        // Update scheduler weights from AI decision
        scheduler.update_weights(&decision);
        
        println!();
    }

    println!("--- Step 2: Scheduler Statistics ---\n");
    let stats = scheduler.stats();
    println!("Queue Sizes:");
    println!("  Critical: {}", stats.critical_queue_size);
    println!("  High: {}", stats.high_queue_size);
    println!("  Normal: {}", stats.normal_queue_size);
    println!("  Low: {}", stats.low_queue_size);
    println!("  Bulk: {}", stats.bulk_queue_size);
    println!("\nWFQ Weights:");
    println!("  P0 (Critical/High): {}%", stats.p0_weight);
    println!("  P1 (Normal): {}%", stats.p1_weight);
    println!("  P2 (Low/Bulk): {}% (enabled: {})", stats.p2_weight, stats.p2_enabled);
    println!("\nTotal Scheduled: {}", stats.total_scheduled);
    println!();

    println!("--- Step 3: Retrieving Scheduled Chunks (Priority Order) ---\n");
    
    // Retrieve chunks in priority order
    let mut retrieved_count = 0;
    while let Some(chunk) = scheduler.get_next() {
        retrieved_count += 1;
        println!("Chunk {}: Priority {:?} ({})", 
            retrieved_count, 
            chunk.priority, 
            chunk.priority as u8
        );
        println!("  Data length: {} bytes", chunk.data.len());
        if let Some(route) = chunk.route {
            println!("  Route: {:?}", RouteDecision::from(route));
        }
        println!();
        
        if retrieved_count >= 10 {
            break; // Limit output
        }
    }

    println!("✅ Example completed!");
    println!("Total chunks retrieved: {}", retrieved_count);
    
    Ok(())
}

