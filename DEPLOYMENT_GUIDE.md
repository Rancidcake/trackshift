# Deployment & Demonstration Guide

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Python 3 (for model generation)
python3 --version  # Should be 3.7+

# Install ONNX Python package
pip3 install onnx numpy
```

### Build the System

```bash
# Clone/navigate to the project
cd /Users/mayankhete/trs/PitlinkPQC

# Generate ONNX models
python3 brain/scripts/create_onnx_models.py

# Copy models to brain directory
cp models/*.onnx brain/models/

# Build all components
cargo build --release

# Or build specific components
cargo build -p trackshift --release
cargo build -p quic_fec --release
cargo build -p rust_pqc --release
```

## 📖 Usage Examples

### 1. Basic Telemetry Processing

```rust
use trackshift::*;

fn main() -> anyhow::Result<()> {
    // Initialize AI system
    let ai_system = TelemetryAi::new(
        "brain/models/slm.onnx",
        "brain/models/embedder.onnx"
    )?;

    // Network metrics
    let metrics = NetworkMetricsInput {
        rtt_ms: 20.0,
        jitter_ms: 3.0,
        loss_rate: 0.001,
        throughput_mbps: 100.0,
        wifi_signal: -50.0,
        starlink_latency: 35.0,
        ..Default::default()
    };

    // Process telemetry chunk
    let chunk_data = b"Temperature: 25.5C, Humidity: 60%";
    let decision = ai_system.process_chunk(chunk_data, metrics)?;

    println!("Route: {:?}", decision.route);
    println!("Should send: {}", decision.should_send);
    println!("Network quality: {:.2}", decision.network_quality.score);

    Ok(())
}
```

### 2. Priority Tagging & Scheduling

```rust
use trackshift::*;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    // Initialize components
    let tagger = PriorityTagger::new();
    let scheduler = Arc::new(PriorityScheduler::new());

    // Process different data types
    let chunks = vec![
        (b"Alert: Fire detected".to_vec(), "Critical Alert"),
        (b"Warning: High CPU".to_vec(), "Warning"),
        (b"Status: OK".to_vec(), "Status"),
    ];

    for (data, description) in chunks {
        // Tag priority automatically
        let priority = tagger.tag_priority(&data, None);
        println!("{}: Priority {:?}", description, priority);

        // Schedule for transmission
        scheduler.schedule(data, priority, None)?;
    }

    // Retrieve in priority order
    while let Some(chunk) = scheduler.get_next() {
        println!("Sending chunk with priority {:?}", chunk.priority);
        // Send chunk...
    }

    Ok(())
}
```

### 3. Medical Data Processing

```rust
use trackshift::*;

fn main() -> anyhow::Result<()> {
    let tagger = PriorityTagger::new();
    let scheduler = Arc::new(PriorityScheduler::new());
    let monitor = RealtimeStatusMonitor::with_scheduler(scheduler.clone());

    // HL7 message
    let hl7_data = b"MSH|^~\\&|SendingApp|Facility|ReceivingApp|ReceivingFacility|20240101120000||ADT^A01|12345|P|2.5";
    
    // Tag priority (automatically detects medical format)
    let priority = tagger.tag_priority(hl7_data, None);
    let scenario = tagger.detect_scenario(hl7_data);
    
    println!("Format: Medical (HL7)");
    println!("Priority: {:?}", priority);
    println!("Scenario: {:?}", scenario);

    // Register transfer
    monitor.register_transfer(
        "hl7-msg-12345".to_string(),
        hl7_data.len() as u64,
        priority,
        IntegrityMethod::Blake3,
    );

    // Schedule
    scheduler.schedule(hl7_data.to_vec(), priority, None)?;

    // Monitor progress
    let status = monitor.get_transfer_status("hl7-msg-12345");
    println!("Transfer status: {:?}", status);

    Ok(())
}
```

### 4. Real-Time Status Monitoring

```rust
use trackshift::*;

fn main() -> anyhow::Result<()> {
    let monitor = RealtimeStatusMonitor::new();

    // Register multiple transfers
    monitor.register_transfer("transfer-1".to_string(), 1_000_000, ChunkPriority::High, IntegrityMethod::Blake3);
    monitor.register_transfer("transfer-2".to_string(), 5_000_000, ChunkPriority::Normal, IntegrityMethod::Blake3);

    // Update progress
    monitor.update_transfer("transfer-1", 500_000, TransferStatus::InProgress, Some(10.0))?;

    // Get comprehensive status
    let snapshot = monitor.get_status_snapshot();
    
    println!("Active Transfers: {}", snapshot.health.active_transfers);
    println!("Queue Size: {}", snapshot.health.queue_size);
    
    for transfer in &snapshot.transfers {
        println!("Transfer {}: {:.1}% - {:?}", 
            transfer.transfer_id,
            transfer.progress * 100.0,
            transfer.status
        );
    }

    Ok(())
}
```

### 5. Complete Integration Example

```rust
use trackshift::*;
use trackshift::integration::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize all components
    let pipeline = IntegratedTelemetryPipeline::new(
        "brain/models/slm.onnx",
        "brain/models/embedder.onnx",
        true,  // encryption enabled
        true,  // compression enabled
    )?;

    let tagger = PriorityTagger::new();
    let scheduler = Arc::new(PriorityScheduler::new());
    let monitor = RealtimeStatusMonitor::with_scheduler(scheduler.clone());

    // Network metrics
    let metrics = NetworkMetricsInput {
        rtt_ms: 25.0,
        jitter_ms: 5.0,
        loss_rate: 0.01,
        throughput_mbps: 50.0,
        wifi_signal: -70.0,
        starlink_latency: 40.0,
        ..Default::default()
    };

    // Process telemetry data
    let chunk_data = b"Alert: Critical system failure detected";
    
    // 1. Process through pipeline (AI + compression + encryption)
    let processed = pipeline.process_chunk_full(chunk_data, metrics.clone())?;
    
    // 2. Tag priority
    let priority = tagger.tag_priority(chunk_data, Some(processed.decision.severity));
    
    // 3. Schedule
    if let Some(data) = processed.processed_data {
        scheduler.schedule(data, priority, Some(processed.decision.route as u32))?;
    }

    // 4. Monitor
    monitor.update_network_status(metrics, processed.decision.network_quality.score, processed.decision.network_quality.is_patchy);
    
    println!("✅ Complete integration working!");

    Ok(())
}
```

## 🎯 Demonstration Scenarios

### Scenario 1: Medical Emergency

```bash
# Run the priority scheduler example
cd /Users/mayankhete/trs/PitlinkPQC/brain
cargo run --example priority_scheduler

# This demonstrates:
# - Automatic priority tagging
# - Priority-based scheduling
# - WFQ weight management
```

### Scenario 2: Disaster Response

Create a demo script:

```rust
// examples/disaster_response_demo.rs
use trackshift::*;

fn main() -> anyhow::Result<()> {
    let tagger = PriorityTagger::new();
    
    let alerts = vec![
        b"Evacuate: Immediate danger - structural collapse",
        b"Emergency: Chemical spill detected",
        b"Status: All personnel accounted for",
    ];

    for alert in alerts {
        let priority = tagger.tag_priority(alert, None);
        let scenario = tagger.detect_scenario(alert);
        let format = tagger.detect_format(alert);
        
        println!("Alert: {}", String::from_utf8_lossy(alert));
        println!("  Format: {:?}", format);
        println!("  Scenario: {:?}", scenario);
        println!("  Priority: {:?} ({})", priority, priority as u8);
        println!();
    }

    Ok(())
}
```

### Scenario 3: Real-Time Monitoring Dashboard

```bash
# If dashboard is available
cargo run --bin dashboard

# Or use the status monitor API
cargo run --example realtime_status_demo
```

## 🚢 Deployment Options

### Option 1: Standalone Binary

```bash
# Build release binary
cargo build --release -p trackshift

# Run
./target/release/trackshift

# Or install globally
cargo install --path brain --bin trackshift
trackshift
```

### Option 2: Docker Deployment

Create `Dockerfile`:

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/trackshift /usr/local/bin/
COPY --from=builder /app/brain/models/*.onnx /app/models/

WORKDIR /app
CMD ["trackshift"]
```

Build and run:

```bash
docker build -t pitlinkpqc .
docker run -p 8080:8080 pitlinkpqc
```

### Option 3: Systemd Service

Create `/etc/systemd/system/pitlinkpqc.service`:

```ini
[Unit]
Description=PitlinkPQC Telemetry System
After=network.target

[Service]
Type=simple
User=pitlinkpqc
WorkingDirectory=/opt/pitlinkpqc
ExecStart=/opt/pitlinkpqc/trackshift
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable pitlinkpqc
sudo systemctl start pitlinkpqc
sudo systemctl status pitlinkpqc
```

### Option 4: Kubernetes Deployment

Create `k8s/deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pitlinkpqc
spec:
  replicas: 3
  selector:
    matchLabels:
      app: pitlinkpqc
  template:
    metadata:
      labels:
        app: pitlinkpqc
    spec:
      containers:
      - name: pitlinkpqc
        image: pitlinkpqc:latest
        ports:
        - containerPort: 8080
        volumeMounts:
        - name: models
          mountPath: /app/models
      volumes:
      - name: models
        configMap:
          name: onnx-models
```

## 📊 Demonstration Scripts

### Demo 1: Priority Tagging Demo

```bash
# Create demo script
cat > demo_priority.sh << 'EOF'
#!/bin/bash
echo "🎯 Priority Tagging Demonstration"
echo "=================================="
echo ""

cd /Users/mayankhete/trs/PitlinkPQC/brain

# Run priority scheduler example
cargo run --example priority_scheduler 2>&1 | grep -A 5 "Tagging Priority"
EOF

chmod +x demo_priority.sh
./demo_priority.sh
```

### Demo 2: Medical Data Demo

```bash
# Create medical data demo
cat > demo_medical.sh << 'EOF'
#!/bin/bash
echo "🏥 Medical Data Processing Demo"
echo "==============================="

cd /Users/mayankhete/trs/PitlinkPQC/brain

# Create test HL7 message
echo "MSH|^~\\&|SendingApp|Facility|ReceivingApp|ReceivingFacility|20240101120000||ADT^A01|12345|P|2.5" > /tmp/hl7_test.txt

# Process with system
cargo run --example priority_scheduler 2>&1 | grep -i "medical\|priority"
EOF

chmod +x demo_medical.sh
./demo_medical.sh
```

### Demo 3: Real-Time Status Demo

Create `examples/realtime_demo.rs`:

```rust
use trackshift::*;
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let monitor = RealtimeStatusMonitor::new();

    // Simulate multiple transfers
    for i in 1..=5 {
        monitor.register_transfer(
            format!("transfer-{}", i),
            10_000_000 * i as u64,
            ChunkPriority::High,
            IntegrityMethod::Blake3,
        );
    }

    // Simulate progress updates
    for i in 1..=5 {
        thread::sleep(Duration::from_millis(500));
        
        let progress = i as u64 * 2_000_000;
        monitor.update_transfer(
            &format!("transfer-{}", i),
            progress,
            TransferStatus::InProgress,
            Some(10.0),
        )?;

        let snapshot = monitor.get_status_snapshot();
        println!("Active transfers: {}", snapshot.health.active_transfers);
        
        for transfer in &snapshot.transfers {
            if transfer.status == TransferStatus::InProgress {
                println!("  {}: {:.1}% - {:.2} Mbps", 
                    transfer.transfer_id,
                    transfer.progress * 100.0,
                    transfer.speed_mbps
                );
            }
        }
        println!();
    }

    Ok(())
}
```

Run:

```bash
cargo run --example realtime_demo
```

## 🔧 Configuration

### Environment Variables

```bash
# Set model paths
export SLM_MODEL_PATH=./brain/models/slm.onnx
export EMBEDDER_MODEL_PATH=./brain/models/embedder.onnx

# Set log level
export RUST_LOG=info

# Set network interface
export NETWORK_INTERFACE=eth0
```

### Configuration File

Create `config.toml`:

```toml
[models]
slm_path = "./brain/models/slm.onnx"
embedder_path = "./brain/models/embedder.onnx"

[network]
default_route = "WiFi"
enable_multipath = true

[scheduler]
max_queue_size = 1000
p0_weight = 50
p1_weight = 30
p2_weight = 20

[monitoring]
update_interval_ms = 1000
retention_seconds = 3600
```

## 📈 Performance Benchmarks

### Run Benchmarks

```bash
# Latency benchmark
cargo run --example latency_benchmark

# Full latency measurement
cargo run --example full_latency_measurement

# RL demo
cargo run --example rl_demo
```

## 🎬 Live Demonstration

### Step-by-Step Demo Flow

1. **Start the System**
   ```bash
   cd /Users/mayankhete/trs/PitlinkPQC/brain
   cargo run --example priority_scheduler
   ```

2. **Show Priority Tagging**
   - Different data types get different priorities
   - Medical alerts → Critical
   - Status updates → Normal/Low

3. **Show Scheduling**
   - Chunks queued by priority
   - Retrieved in priority order
   - WFQ weights applied

4. **Show Real-Time Status**
   - Transfer progress
   - Network quality
   - System health

5. **Show Integration**
   - AI decisions
   - Compression
   - Encryption
   - Priority tagging

## 📝 API Documentation

### Generate Docs

```bash
# Generate documentation
cargo doc --open -p trackshift

# Or view online
# Navigate to target/doc/trackshift/index.html
```

## 🐛 Troubleshooting

### Common Issues

1. **Models not found**
   ```bash
   # Generate models
   python3 brain/scripts/create_onnx_models.py
   cp models/*.onnx brain/models/
   ```

2. **ORT API errors**
   - Check ort version: `cargo tree | grep ort`
   - Should be `2.0.0-rc.10`

3. **Build errors**
   ```bash
   # Clean and rebuild
   cargo clean
   cargo build --release
   ```

## ✅ Quick Test

```bash
# Run all examples
cd /Users/mayankhete/trs/PitlinkPQC/brain

# Priority scheduler
cargo run --example priority_scheduler

# Integrated workflow
cargo run --example integrated_workflow

# Patchy network
cargo run --example patchy_network_example
```

## 🎯 Next Steps

1. **Customize for your use case**
   - Add custom keywords to priority tagger
   - Adjust WFQ weights
   - Configure network routes

2. **Integrate with your system**
   - Use the API in your application
   - Connect to your data sources
   - Set up monitoring

3. **Deploy to production**
   - Use Docker or Kubernetes
   - Set up monitoring
   - Configure backups

The system is ready to use! 🚀

