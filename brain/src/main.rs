/// TrackShift Telemetry AI - Main CLI
/// 
/// Simple command-line interface for testing the telemetry AI system

use trackshift::telemetry_ai::*;
use std::io::{self, Write, Read};

fn main() -> anyhow::Result<()> {
    println!("🚀 TrackShift Telemetry AI System");
    println!("===================================\n");

    // Initialize AI system
    println!("Loading ONNX models...");
    let ai_system = TelemetryAi::new("models/slm.onnx", "models/embedder.onnx")?;
    println!("✅ Models loaded successfully!\n");

    // Interactive loop
    loop {
        println!("Options:");
        println!("  1. Process telemetry chunk");
        println!("  2. Show system status");
        println!("  3. Exit");
        print!("\nChoice: ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();

        match choice {
            "1" => {
                process_chunk_interactive(&ai_system)?;
            }
            "2" => {
                show_status(&ai_system)?;
            }
            "3" => {
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("Invalid choice. Please enter 1, 2, or 3.\n");
            }
        }
    }

    Ok(())
}

fn process_chunk_interactive(ai_system: &TelemetryAi) -> anyhow::Result<()> {
    println!("\n--- Process Telemetry Chunk ---");
    
    // Get network metrics
    let mut metrics = NetworkMetricsInput::default();
    
    print!("RTT (ms) [default: 15.0]: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if !input.trim().is_empty() {
        metrics.rtt_ms = input.trim().parse().unwrap_or(15.0);
    }
    
    print!("Throughput (Mbps) [default: 100.0]: ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    if !input.trim().is_empty() {
        metrics.throughput_mbps = input.trim().parse().unwrap_or(100.0);
    }
    
    print!("WiFi Signal (dBm) [default: -50.0]: ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    if !input.trim().is_empty() {
        metrics.wifi_signal = input.trim().parse().unwrap_or(-50.0);
    }
    
    // Create sample chunk data
    let chunk_data = vec![0u8; 1024]; // In real system, this would be actual telemetry data
    
    println!("\nProcessing chunk...");
    let decision = ai_system.process_chunk(&chunk_data, metrics)?;
    
    println!("\n📊 AI Decision:");
    println!("  Route: {:?}", decision.route);
    println!("  Severity: {:?}", decision.severity);
    println!("  P2 Enabled: {}", decision.p2_enable);
    println!("  Congestion Predicted: {}", decision.congestion_predicted);
    println!("  WFQ Weights:");
    println!("    P0 (High Priority): {}%", decision.wfq_p0_weight);
    println!("    P1 (Medium Priority): {}%", decision.wfq_p1_weight);
    println!("    P2 (Low Priority): {}%", decision.wfq_p2_weight);
    
    println!("\n💡 Recommendations:");
    match decision.route {
        RouteDecision::WiFi => println!("  → Use WiFi network"),
        RouteDecision::Starlink => println!("  → Use Starlink network"),
        RouteDecision::Multipath => println!("  → Use multiple paths simultaneously"),
        RouteDecision::FiveG => println!("  → Use 5G network"),
    }
    
    if decision.severity == Severity::High {
        println!("  ⚠️  HIGH SEVERITY - Immediate action required!");
    }
    
    if decision.congestion_predicted {
        println!("  ⚠️  Congestion predicted in next 200-500ms");
    }
    
    println!();
    Ok(())
}

fn show_status(ai_system: &TelemetryAi) -> anyhow::Result<()> {
    println!("\n--- System Status ---");
    
    let store = ai_system.context_store();
    let context_store = store.read();
    
    println!("Context Store:");
    println!("  Stored Embeddings: {}", context_store.len());
    println!("  Empty: {}", context_store.is_empty());
    
    println!("\nModels:");
    println!("  SLM Model: ✅ Loaded");
    println!("  Embedder Model: ✅ Loaded");
    
    println!();
    Ok(())
}

