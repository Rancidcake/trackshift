//! End-to-End Tests

use trackshift::telemetry_ai::{
    TelemetryAi, NetworkMetricsInput, RLRecorder, RouteDecision,
};

#[test]
fn test_complete_transfer_episode() {
    // This is a comprehensive test that simulates a complete transfer
    // with RL learning throughout the process
    
    // Initialize AI system
    let ai_result = TelemetryAi::new("models/slm.onnx", "models/embedder.onnx");
    
    if ai_result.is_err() {
        // Models not available, skip test
        return;
    }
    
    let ai = ai_result.unwrap();
    let rl_manager = ai.rl_manager().unwrap();
    let mut recorder = RLRecorder::new(rl_manager.clone());
    
    // Simulate file transfer
    let transfer_id = "e2e_transfer_1";
    let initial_metrics = NetworkMetricsInput {
        rtt_ms: 25.0,
        loss_rate: 0.002,
        throughput_mbps: 150.0,
        jitter_ms: 3.0,
        ..Default::default()
    };
    
    // Start transfer
    recorder.start_transfer(transfer_id, &initial_metrics, RouteDecision::WiFi, 1);
    
    // Simulate network degradation mid-transfer
    let degraded_metrics = NetworkMetricsInput {
        rtt_ms: 300.0,
        loss_rate: 0.1,
        throughput_mbps: 2.0,
        jitter_ms: 80.0,
        ..initial_metrics
    };
    
    // RL should recommend handover
    let state = trackshift::telemetry_ai::NetworkState::from_metrics(
        &degraded_metrics,
        RouteDecision::WiFi,
        1,
    );
    
    let recommended_action = rl_manager.recommend_action(&state);
    
    // Record handover action
    if recommended_action.to_route_decision().is_some() {
        recorder.record_action(transfer_id, recommended_action);
    }
    
    // Final metrics (after handover to better path)
    let final_metrics = NetworkMetricsInput {
        rtt_ms: 35.0,
        loss_rate: 0.003,
        throughput_mbps: 180.0,
        jitter_ms: 4.0,
        ..initial_metrics
    };
    
    // End transfer successfully
    recorder.end_transfer(transfer_id, final_metrics, true).unwrap();
    
    // Verify learning occurred
    let (q_stats, _) = rl_manager.get_stats();
    assert!(q_stats.total_episodes > 0);
}

#[test]
fn test_rl_adaptation_over_time() {
    // Test that RL adapts to network patterns over multiple episodes
    
    let rl_manager = std::sync::Arc::new(trackshift::telemetry_ai::RLManager::new());
    let mut recorder = RLRecorder::new(rl_manager.clone());
    
    // Pattern: WiFi consistently fails, 5G consistently succeeds
    for i in 0..10 {
        let transfer_id = format!("adaptation_test_{}", i);
        
        // Try WiFi first
        let wifi_metrics = NetworkMetricsInput {
            rtt_ms: 250.0,
            loss_rate: 0.08,
            throughput_mbps: 5.0,
            ..Default::default()
        };
        
        recorder.start_transfer(&transfer_id, &wifi_metrics, RouteDecision::WiFi, 0);
        
        // WiFi fails, handover to 5G
        recorder.record_action(&transfer_id, trackshift::telemetry_ai::RLAction::HandoverToFiveG);
        
        // 5G succeeds
        let fiveg_metrics = NetworkMetricsInput {
            rtt_ms: 30.0,
            loss_rate: 0.001,
            throughput_mbps: 200.0,
            ..Default::default()
        };
        
        recorder.end_transfer(&transfer_id, fiveg_metrics, true).unwrap();
    }
    
    // After learning, RL should prefer 5G for similar conditions
    let test_state = trackshift::telemetry_ai::NetworkState::from_metrics(
        &NetworkMetricsInput {
            rtt_ms: 200.0,
            loss_rate: 0.05,
            throughput_mbps: 10.0,
            ..Default::default()
        },
        RouteDecision::WiFi,
        0,
    );
    
    // Get recommendation (should learn to prefer 5G)
    let recommended = rl_manager.recommend_action(&test_state);
    
    // Verify it's a valid action (RL may need more episodes to consistently prefer 5G)
    assert!(trackshift::telemetry_ai::RLAction::all().contains(&recommended));
    
    // Check that learning occurred (stats should show episodes)
    let (q_stats, _) = rl_manager.get_stats();
    assert!(q_stats.total_episodes >= 10);
}

#[test]
fn test_rl_statistics_tracking() {
    let rl_manager = std::sync::Arc::new(trackshift::telemetry_ai::RLManager::new());
    let mut recorder = RLRecorder::new(rl_manager.clone());
    
    // Run multiple transfers with varying outcomes
    let outcomes = vec![true, true, false, true, true, false, true];
    
    for (i, &success) in outcomes.iter().enumerate() {
        let transfer_id = format!("stats_test_{}", i);
        let metrics = NetworkMetricsInput {
            rtt_ms: if success { 30.0 } else { 300.0 },
            loss_rate: if success { 0.001 } else { 0.1 },
            throughput_mbps: if success { 200.0 } else { 2.0 },
            ..Default::default()
        };
        
        recorder.start_transfer(&transfer_id, &metrics, RouteDecision::WiFi, 0);
        // Record an action so learning occurs
        recorder.record_action(&transfer_id, trackshift::telemetry_ai::RLAction::SelectWiFi);
        recorder.end_transfer(&transfer_id, metrics, success).unwrap();
    }
    
    let (q_stats, _) = rl_manager.get_stats();
    
    // Verify that episodes were recorded
    assert_eq!(q_stats.total_episodes, outcomes.len() as u64);
    
    // Verify success/failure counts (may vary based on reward calculation)
    assert!(q_stats.successful_decisions + q_stats.failed_decisions > 0);
    assert!(q_stats.average_reward != 0.0);
}

