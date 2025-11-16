//! Integration Tests for RL + Telemetry AI

use trackshift::telemetry_ai::{
    TelemetryAi, NetworkMetricsInput, RLRecorder, RouteDecision,
};

#[test]
fn test_rl_recorder_lifecycle() {
    // Create RL manager
    let rl_manager = trackshift::telemetry_ai::RLManager::new();
    let mut recorder = RLRecorder::new(std::sync::Arc::new(rl_manager));
    
    // Start transfer
    let initial_metrics = NetworkMetricsInput {
        rtt_ms: 20.0,
        loss_rate: 0.001,
        throughput_mbps: 100.0,
        ..Default::default()
    };
    
    recorder.start_transfer(
        "test_transfer_1",
        &initial_metrics,
        RouteDecision::WiFi,
        0, // Critical priority
    );
    
    // Record action
    recorder.record_action("test_transfer_1", trackshift::telemetry_ai::RLAction::HandoverToFiveG);
    
    // End transfer
    let final_metrics = NetworkMetricsInput {
        rtt_ms: 30.0,
        loss_rate: 0.002,
        throughput_mbps: 150.0,
        ..initial_metrics
    };
    
    let result = recorder.end_transfer("test_transfer_1", final_metrics, true);
    assert!(result.is_ok());
}

#[test]
fn test_rl_learning_from_multiple_transfers() {
    let rl_manager = std::sync::Arc::new(trackshift::telemetry_ai::RLManager::new());
    let mut recorder = RLRecorder::new(rl_manager.clone());
    
    // Simulate multiple transfers
    let transfers = vec![
        ("transfer_1", RouteDecision::WiFi, true),
        ("transfer_2", RouteDecision::FiveG, true),
        ("transfer_3", RouteDecision::WiFi, false), // Failed on WiFi
        ("transfer_4", RouteDecision::FiveG, true),
    ];
    
    for (transfer_id, path, success) in transfers {
        let metrics = NetworkMetricsInput {
            rtt_ms: if path == RouteDecision::WiFi { 200.0 } else { 30.0 },
            loss_rate: if path == RouteDecision::WiFi { 0.05 } else { 0.001 },
            throughput_mbps: if path == RouteDecision::WiFi { 10.0 } else { 200.0 },
            ..Default::default()
        };
        
        recorder.start_transfer(transfer_id, &metrics, path, 0);
        
        // Always record an action so learning occurs
        let action = match path {
            RouteDecision::WiFi => trackshift::telemetry_ai::RLAction::SelectWiFi,
            RouteDecision::FiveG => trackshift::telemetry_ai::RLAction::SelectFiveG,
            _ => trackshift::telemetry_ai::RLAction::SelectWiFi,
        };
        recorder.record_action(transfer_id, action);
        
        if !success {
            // Try handover on failure
            recorder.record_action(transfer_id, trackshift::telemetry_ai::RLAction::HandoverToFiveG);
        }
        
        let final_metrics = if !success {
            NetworkMetricsInput {
                rtt_ms: 40.0, // Better after handover
                loss_rate: 0.005,
                throughput_mbps: 150.0,
                ..metrics
            }
        } else {
            metrics.clone()
        };
        
        recorder.end_transfer(transfer_id, final_metrics, success).unwrap();
    }
    
    // Check that learning occurred
    let (q_stats, _) = rl_manager.get_stats();
    // Each transfer creates episodes based on actions taken
    // With handovers, we get multiple learning events per transfer
    assert!(q_stats.total_episodes >= 4);
    assert!(q_stats.successful_decisions + q_stats.failed_decisions > 0);
}

#[test]
fn test_rl_state_evolution() {
    let manager = trackshift::telemetry_ai::RLManager::new();
    
    // Start with good WiFi
    let state1 = trackshift::telemetry_ai::NetworkState::from_metrics(
        &NetworkMetricsInput {
            rtt_ms: 20.0,
            loss_rate: 0.001,
            throughput_mbps: 100.0,
            ..Default::default()
        },
        RouteDecision::WiFi,
        0,
    );
    
    manager.start_episode(state1.clone());
    manager.record_action(trackshift::telemetry_ai::RLAction::SelectWiFi);
    
    // Network degrades
    let state2 = trackshift::telemetry_ai::NetworkState::from_metrics(
        &NetworkMetricsInput {
            rtt_ms: 250.0,
            loss_rate: 0.08,
            throughput_mbps: 5.0,
            ..Default::default()
        },
        RouteDecision::WiFi,
        0,
    );
    
    // Handover to 5G
    manager.record_action(trackshift::telemetry_ai::RLAction::HandoverToFiveG);
    
    // Final state (better on 5G)
    let state3 = trackshift::telemetry_ai::NetworkState::from_metrics(
        &NetworkMetricsInput {
            rtt_ms: 40.0,
            loss_rate: 0.005,
            throughput_mbps: 150.0,
            ..Default::default()
        },
        RouteDecision::FiveG,
        0,
    );
    
    manager.end_episode(state3, 50.0, &NetworkMetricsInput::default());
    
    // Check stats
    let (q_stats, _) = manager.get_stats();
    assert_eq!(q_stats.total_episodes, 1);
}

#[test]
fn test_reward_calculation_variations() {
    use trackshift::telemetry_ai::Reward;
    
    // High throughput, low latency, low loss
    let reward1 = Reward::calculate(true, 500.0, 20.0, 0.001, 5.0);
    
    // Medium throughput, medium latency, medium loss
    let reward2 = Reward::calculate(true, 100.0, 100.0, 0.01, 10.0);
    
    // Low throughput, high latency, high loss
    let reward3 = Reward::calculate(true, 10.0, 300.0, 0.05, 30.0);
    
    // Failed transfer
    let reward4 = Reward::calculate(false, 5.0, 500.0, 0.1, 100.0);
    
    // Reward1 should be highest (best conditions)
    assert!(reward1.total > reward2.total);
    assert!(reward2.total > reward3.total);
    assert!(reward3.total > reward4.total);
    
    // All successful transfers should have positive base
    assert_eq!(reward1.base, 100.0);
    assert_eq!(reward2.base, 100.0);
    assert_eq!(reward3.base, 100.0);
    
    // Failed transfer should have negative base
    assert_eq!(reward4.base, -200.0);
}

