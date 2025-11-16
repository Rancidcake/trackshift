//! Reinforcement Learning Demo
//!
//! Demonstrates how the RL system learns optimal routing decisions
//! by observing transfer outcomes and network conditions.

use trackshift::telemetry_ai::{
    TelemetryAi, NetworkMetricsInput, RLManager, RLRecorder, NetworkState, RLAction, Reward,
    RouteDecision,
};
use anyhow::Result;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 Reinforcement Learning Demo");
    println!("==============================\n");

    // Initialize AI system (RL enabled by default)
    let ai_system = TelemetryAi::new("models/slm.onnx", "models/embedder.onnx")?;
    
    // Get RL manager
    let rl_manager = ai_system.rl_manager()
        .ok_or_else(|| anyhow::anyhow!("RL manager not available"))?;

    // Create RL recorder for tracking transfers
    let mut recorder = RLRecorder::new(rl_manager.clone());

    println!("📊 Simulating Transfer Episodes for RL Learning\n");

    // Simulate multiple transfer episodes with different network conditions
    let episodes = vec![
        // Episode 1: Good WiFi network
        (
            "transfer_1",
            NetworkMetricsInput {
                rtt_ms: 20.0,
                loss_rate: 0.001,
                throughput_mbps: 100.0,
                jitter_ms: 2.0,
                ..Default::default()
            },
            RouteDecision::WiFi,
            0, // Critical priority
            true, // Success
        ),
        // Episode 2: Patchy WiFi, should learn to switch
        (
            "transfer_2",
            NetworkMetricsInput {
                rtt_ms: 200.0,
                loss_rate: 0.05,
                throughput_mbps: 10.0,
                jitter_ms: 50.0,
                ..Default::default()
            },
            RouteDecision::WiFi,
            0,
            false, // Failed on WiFi
        ),
        // Episode 3: Good 5G network
        (
            "transfer_3",
            NetworkMetricsInput {
                rtt_ms: 30.0,
                loss_rate: 0.002,
                throughput_mbps: 200.0,
                jitter_ms: 3.0,
                ..Default::default()
            },
            RouteDecision::FiveG,
            1, // High priority
            true,
        ),
        // Episode 4: Multipath aggregation
        (
            "transfer_4",
            NetworkMetricsInput {
                rtt_ms: 25.0,
                loss_rate: 0.001,
                throughput_mbps: 300.0, // Combined bandwidth
                jitter_ms: 2.0,
                ..Default::default()
            },
            RouteDecision::Multipath,
            2, // Normal priority
            true,
        ),
    ];

    for (transfer_id, initial_metrics, path, priority, success) in episodes {
        println!("📡 Episode: {}", transfer_id);
        println!("   Initial: {} path, RTT: {:.1}ms, Loss: {:.2}%, Throughput: {:.1} Mbps",
                 match path {
                     RouteDecision::WiFi => "WiFi",
                     RouteDecision::Starlink => "Starlink",
                     RouteDecision::Multipath => "Multipath",
                     RouteDecision::FiveG => "5G",
                 },
                 initial_metrics.rtt_ms,
                 initial_metrics.loss_rate * 100.0,
                 initial_metrics.throughput_mbps);

        // Start episode
        recorder.start_transfer(transfer_id, &initial_metrics, path, priority);

        // Simulate some actions during transfer
        if !success {
            // Try handover on failure
            recorder.record_action(transfer_id, RLAction::HandoverToFiveG);
        }

        // Simulate final metrics (might be better after handover)
        let final_metrics = if !success {
            NetworkMetricsInput {
                rtt_ms: 40.0, // Better after handover
                loss_rate: 0.005,
                throughput_mbps: 150.0,
                jitter_ms: 5.0,
                ..initial_metrics
            }
        } else {
            initial_metrics.clone()
        };

        // End episode and learn
        recorder.end_transfer(transfer_id, final_metrics, success)?;

        println!("   Outcome: {}\n", if success { "✅ Success" } else { "❌ Failed (learned to handover)" });
    }

    // Show RL statistics
    let (q_stats, policy_stats) = rl_manager.get_stats();
    println!("📈 RL Learning Statistics:");
    println!("   Q-Learning:");
    println!("     Episodes: {}", q_stats.total_episodes);
    println!("     Average Reward: {:.2}", q_stats.average_reward);
    println!("     Best Reward: {:.2}", q_stats.best_reward);
    println!("     Exploration Rate: {:.3}", q_stats.exploration_rate);
    println!("     Q-Table Size: {}", q_stats.q_table_size);
    println!("     Success Rate: {:.1}%",
             if q_stats.total_episodes > 0 {
                 (q_stats.successful_decisions as f32 / q_stats.total_episodes as f32) * 100.0
             } else {
                 0.0
             });

    println!("\n   Policy Gradient:");
    println!("     Episodes: {}", policy_stats.total_episodes);
    println!("     Average Reward: {:.2}", policy_stats.average_reward);

    // Test RL recommendations
    println!("\n🎯 Testing RL Recommendations:\n");

    let test_scenarios = vec![
        ("Good WiFi", NetworkMetricsInput {
            rtt_ms: 15.0,
            loss_rate: 0.001,
            throughput_mbps: 120.0,
            jitter_ms: 1.0,
            ..Default::default()
        }),
        ("Patchy WiFi", NetworkMetricsInput {
            rtt_ms: 250.0,
            loss_rate: 0.08,
            throughput_mbps: 5.0,
            jitter_ms: 60.0,
            ..Default::default()
        }),
        ("Excellent 5G", NetworkMetricsInput {
            rtt_ms: 25.0,
            loss_rate: 0.001,
            throughput_mbps: 500.0,
            jitter_ms: 2.0,
            ..Default::default()
        }),
    ];

    for (scenario, metrics) in test_scenarios {
        let state = NetworkState::from_metrics(&metrics, RouteDecision::WiFi, 0);
        let recommended_action = rl_manager.recommend_action(&state);
        
        println!("   {}:", scenario);
        println!("     Recommended: {:?}", recommended_action);
        if let Some(route) = recommended_action.to_route_decision() {
            println!("     Route: {:?}", route);
        }
    }

    // Save learned Q-table
    println!("\n💾 Saving learned Q-table...");
    rl_manager.save("q_table.json")?;
    println!("   ✅ Saved to q_table.json");

    println!("\n✨ RL Demo Complete!");
    println!("\nThe RL system has learned from the transfer episodes and");
    println!("will now make better routing decisions based on network conditions.");

    Ok(())
}

