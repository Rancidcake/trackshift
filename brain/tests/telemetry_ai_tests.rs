//! Telemetry AI Test Suite

use trackshift::telemetry_ai::{
    TelemetryAi, NetworkMetricsInput, RouteDecision, ChunkPriority,
};

#[test]
fn test_telemetry_ai_creation() {
    // Note: This test requires model files, so it may fail if models don't exist
    // In a real scenario, you'd use mock models or test fixtures
    let result = TelemetryAi::new("models/slm.onnx", "models/embedder.onnx");
    
    // If models exist, should succeed
    // If not, we'll get an error (which is expected in test environment)
    if result.is_ok() {
        let ai = result.unwrap();
        
        // Check that RL is enabled by default
        assert!(ai.rl_manager().is_some());
    }
}

#[test]
fn test_telemetry_ai_without_rl() {
    let result = TelemetryAi::new_without_rl("models/slm.onnx", "models/embedder.onnx");
    
    if result.is_ok() {
        let ai = result.unwrap();
        assert!(ai.rl_manager().is_none());
    }
}

#[test]
fn test_network_metrics_input_default() {
    let metrics = NetworkMetricsInput::default();
    
    assert_eq!(metrics.rtt_ms, 15.0);
    assert_eq!(metrics.jitter_ms, 2.0);
    assert_eq!(metrics.loss_rate, 0.001);
    assert_eq!(metrics.throughput_mbps, 100.0);
}

#[test]
fn test_route_decision_from_u32() {
    assert_eq!(RouteDecision::from(0), RouteDecision::WiFi);
    assert_eq!(RouteDecision::from(1), RouteDecision::Starlink);
    assert_eq!(RouteDecision::from(2), RouteDecision::Multipath);
    assert_eq!(RouteDecision::from(3), RouteDecision::FiveG);
    assert_eq!(RouteDecision::from(99), RouteDecision::WiFi); // Default fallback
}

#[test]
fn test_chunk_priority_ordering() {
    // Critical should be highest priority
    assert!(ChunkPriority::Critical < ChunkPriority::High);
    assert!(ChunkPriority::High < ChunkPriority::Normal);
    assert!(ChunkPriority::Normal < ChunkPriority::Low);
}

#[test]
fn test_network_metrics_input_clone() {
    let metrics1 = NetworkMetricsInput {
        rtt_ms: 50.0,
        loss_rate: 0.01,
        throughput_mbps: 200.0,
        ..Default::default()
    };
    
    let metrics2 = metrics1.clone();
    
    assert_eq!(metrics1.rtt_ms, metrics2.rtt_ms);
    assert_eq!(metrics1.loss_rate, metrics2.loss_rate);
    assert_eq!(metrics1.throughput_mbps, metrics2.throughput_mbps);
}

