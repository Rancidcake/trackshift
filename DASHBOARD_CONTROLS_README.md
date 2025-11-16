# User-Controlled Dashboard & Fallback System

Complete user control interface and fallback system for the PitlinkPQC file transfer system.

## Features

### 🎛️ User Controls

1. **System Configuration**
   - Max concurrent transfers
   - Default priority levels
   - Chunk size settings
   - AI routing enable/disable
   - Compression algorithm selection
   - Encryption toggle

2. **Network Settings**
   - Preferred network path (WiFi/5G/Starlink)
   - Handover strategy
   - Multipath enable/disable
   - RTT and loss thresholds
   - Bandwidth limits

3. **Fallback Control**
   - Fallback strategy selection
   - Manual fallback trigger
   - Recovery attempts
   - System state monitoring

4. **Transfer Management**
   - View active transfers
   - Pause/resume transfers
   - Cancel transfers
   - Change transfer priority

### 🔄 Fallback System

The fallback system provides graceful degradation when experimental features fail:

#### Fallback States

1. **FullExperimental** (100% health)
   - All features enabled
   - QUIC + FEC + Multipath + Handover
   - Best performance

2. **QuicWithFec** (80% health)
   - QUIC + FEC enabled
   - Multipath disabled
   - Handover disabled
   - Good reliability

3. **QuicBasic** (60% health)
   - Basic QUIC only
   - No FEC
   - No multipath
   - Moderate performance

4. **TcpFallback** (40% health)
   - TCP instead of QUIC
   - No advanced features
   - Basic reliability

5. **MinimalFallback** (20% health)
   - Minimal features only
   - Encryption only
   - Last resort

#### Fallback Strategies

- **None**: No fallback (fail fast)
- **Automatic**: Fallback on any failure
- **Conservative**: Fallback only on critical failures
- **Aggressive**: Quick fallback after 2 failures

#### Fallback Triggers

- Connection failures
- FEC failures
- Multipath failures
- Handover failures
- High error rates
- Timeouts
- Manual trigger

## API Endpoints

### Dashboard State

```bash
GET /api/dashboard/state
```

Returns complete dashboard state including:
- System state and configuration
- Network settings
- Performance metrics
- Active transfers
- Fallback history

### System Health

```bash
GET /api/dashboard/system-health
```

Returns system health status:
- Health score (0-100)
- Current system state
- Failure rate
- Network efficiency
- Last fallback time

### Configuration

```bash
GET /api/dashboard/config
POST /api/dashboard/config
```

Get or update system configuration:

```json
{
  "system": {
    "max_concurrent_transfers": 10,
    "default_priority": "Medium",
    "default_chunk_size": 65536,
    "enable_ai_routing": true,
    "enable_compression": true,
    "compression_algorithm": "Lz4",
    "enable_encryption": true
  },
  "network": {
    "preferred_path": "WiFi",
    "handover_strategy": "Smooth",
    "enable_multipath": true,
    "max_rtt_threshold": 200.0,
    "max_loss_threshold": 0.05,
    "bandwidth_limit": null
  },
  "fallback": {
    "strategy": "Automatic",
    "state": "FullExperimental",
    "config": {
      "enable_quic": true,
      "enable_fec": true,
      "enable_multipath": true,
      "enable_handover": true,
      "enable_compression": true,
      "enable_encryption": true,
      "use_tcp_fallback": false
    }
  }
}
```

### Network Settings

```bash
GET /api/dashboard/network
POST /api/dashboard/network
```

Get or update network settings.

### Fallback Control

```bash
POST /api/dashboard/fallback/strategy
```

Set fallback strategy:

```json
{
  "strategy": "automatic"  // none, automatic, conservative, aggressive
}
```

```bash
POST /api/dashboard/fallback/trigger
```

Trigger manual fallback:

```json
{
  "target_state": "quic_basic"  // Optional: full_experimental, quic_with_fec, quic_basic, tcp_fallback, minimal_fallback
}
```

```bash
POST /api/dashboard/fallback/recover
```

Try to recover (upgrade system state).

### Transfer Management

```bash
GET /api/dashboard/transfers
```

Get list of active transfers.

```bash
POST /api/dashboard/transfers/:id/pause
POST /api/dashboard/transfers/:id/resume
POST /api/dashboard/transfers/:id/cancel
```

Control individual transfers.

## Usage Examples

### Set Fallback Strategy

```bash
curl -X POST http://localhost:8080/api/dashboard/fallback/strategy \
  -H "Content-Type: application/json" \
  -d '{"strategy": "automatic"}'
```

### Trigger Manual Fallback

```bash
curl -X POST http://localhost:8080/api/dashboard/fallback/trigger \
  -H "Content-Type: application/json" \
  -d '{"target_state": "quic_basic"}'
```

### Update System Configuration

```bash
curl -X POST http://localhost:8080/api/dashboard/config \
  -H "Content-Type: application/json" \
  -d '{
    "max_concurrent_transfers": 20,
    "default_chunk_size": 131072,
    "enable_ai_routing": true
  }'
```

### Get System Health

```bash
curl http://localhost:8080/api/dashboard/system-health
```

Response:
```json
{
  "status": "Healthy",
  "health_score": 100,
  "current_state": "FullExperimental",
  "active_transfers": 3,
  "failure_rate": 0.01,
  "network_efficiency": 0.95,
  "last_fallback": null
}
```

## Integration

### Using Fallback Manager

```rust
use quic_fec::FallbackManager;

// Create fallback manager
let fallback_manager = Arc::new(
    FallbackManager::new(FallbackStrategy::Automatic)
);

// Report failure (triggers automatic fallback if needed)
if let Ok(Some(new_state)) = fallback_manager.report_failure(
    FallbackReason::ConnectionFailure,
    Some("Connection timeout".to_string()),
) {
    println!("Fell back to: {:?}", new_state);
}

// Get current configuration
let config = fallback_manager.get_current_config();
if !config.enable_fec {
    // FEC is disabled, adjust behavior
}

// Try to recover
if let Ok(Some(upgraded)) = fallback_manager.try_recover() {
    println!("Recovered to: {:?}", upgraded);
}
```

### Using Dashboard Controller

```rust
use dashboard::DashboardController;

// Create controller (with fallback manager)
let controller = Arc::new(
    DashboardController::new(fallback_manager)
);

// Update system configuration
controller.update_system_config(SystemConfig {
    max_concurrent_transfers: 20,
    default_priority: PacketPriority::High,
    ..Default::default()
});

// Update network settings
controller.update_network_settings(NetworkSettings {
    preferred_path: NetworkPath::FiveG,
    enable_multipath: true,
    ..Default::default()
});

// Get dashboard state
let state = controller.get_dashboard_state();
println!("Current state: {:?}", state.system_state);

// Get system health
let health = controller.get_system_health();
println!("Health score: {}", health.health_score);
```

## Fallback Flow

1. **System starts** in `FullExperimental` state
2. **Failure occurs** (e.g., connection failure)
3. **Fallback manager** evaluates failure
4. **If fallback needed**, transitions to next state:
   - FullExperimental → QuicWithFec
   - QuicWithFec → QuicBasic
   - QuicBasic → TcpFallback
   - TcpFallback → MinimalFallback
5. **System continues** with reduced features
6. **Recovery attempt** after cooldown period
7. **Upgrade** to higher state if conditions allow

## Monitoring

The dashboard provides real-time monitoring of:
- Current system state
- Fallback history
- Failure counts per state
- Health score
- Active transfers
- Performance metrics
- Network efficiency

## Best Practices

1. **Start with Automatic** fallback strategy for production
2. **Monitor health score** - if below 60%, investigate issues
3. **Use Conservative** strategy if you want minimal disruption
4. **Manual fallback** for testing or maintenance
5. **Recovery attempts** should be automatic but can be manual
6. **Track fallback history** to identify patterns

## Troubleshooting

### System Stuck in Minimal Fallback

- Check network connectivity
- Verify server is running
- Review error logs
- Try manual recovery

### Frequent Fallbacks

- Review failure reasons
- Adjust thresholds
- Check network conditions
- Consider Conservative strategy

### Recovery Not Working

- Check cooldown period (60 seconds default)
- Verify network conditions improved
- Review failure history
- Try manual recovery

## License

See main project LICENSE file.

