# Quick Demo Guide

## 🚀 5-Minute Demo

### Step 1: Run the Priority Scheduler Example

```bash
cd /Users/mayankhete/trs/PitlinkPQC/brain
cargo run --example priority_scheduler
```

**What it shows:**
- ✅ Automatic priority tagging (Critical, High, Normal, Low)
- ✅ Priority-based scheduling
- ✅ Queue management
- ✅ WFQ weight allocation

### Step 2: Run Medical Data Demo

```bash
# Create a simple medical data test
cd /Users/mayankhete/trs/PitlinkPQC/brain

# Create test file
cat > /tmp/medical_test.txt << 'EOF'
MSH|^~\&|SendingApp|Facility|ReceivingApp|ReceivingFacility|20240101120000||ADT^A01|12345|P|2.5
Code Blue: Cardiac arrest in room 301
Vital Signs: BP 120/80, HR 72, Temp 98.6F
EOF

# Process it (modify example to read from file)
cargo run --example priority_scheduler
```

### Step 3: Show Real-Time Status

Create `examples/quick_status_demo.rs`:

```rust
use trackshift::*;

fn main() -> anyhow::Result<()> {
    let monitor = RealtimeStatusMonitor::new();
    
    // Register a transfer
    monitor.register_transfer(
        "demo-transfer".to_string(),
        1_000_000,
        ChunkPriority::High,
        IntegrityMethod::Blake3,
    );
    
    // Update progress
    monitor.update_transfer(
        "demo-transfer",
        500_000,
        TransferStatus::InProgress,
        Some(10.0),
    )?;
    
    // Show status
    let status = monitor.get_transfer_status("demo-transfer").unwrap();
    println!("Transfer: {}", status.transfer_id);
    println!("Progress: {:.1}%", status.progress * 100.0);
    println!("Speed: {:.2} Mbps", status.speed_mbps);
    println!("Status: {:?}", status.status);
    
    Ok(())
}
```

Run:
```bash
cargo run --example quick_status_demo
```

## 🎬 Presentation Demo Flow

### 1. Introduction (30 seconds)
- "This is PitlinkPQC - an intelligent telemetry system"
- "It automatically prioritizes data and optimizes transmission"

### 2. Priority Tagging Demo (1 minute)
```bash
cargo run --example priority_scheduler
```
**Highlight:**
- Different data types get different priorities automatically
- Critical alerts → Priority 0
- Status updates → Priority 192

### 3. Medical Data Demo (1 minute)
**Show:**
- HL7 message detection
- Code Blue → Critical priority
- Patient data → High priority

### 4. Real-Time Monitoring (1 minute)
**Show:**
- Transfer progress tracking
- Network quality monitoring
- System health metrics

### 5. Integration Demo (1 minute)
**Show:**
- AI decisions
- Compression
- Encryption
- Priority scheduling

## 📊 Demo Scripts

### All-in-One Demo

```bash
#!/bin/bash
# demo_all.sh

echo "🚀 PitlinkPQC Complete Demo"
echo "============================"
echo ""

echo "1. Priority Tagging & Scheduling"
cargo run --example priority_scheduler
echo ""

echo "2. Medical Data Processing"
# Show HL7 detection
echo ""

echo "3. Real-Time Status"
# Show monitoring
echo ""

echo "✅ Demo Complete!"
```

## 🎯 Key Points to Highlight

1. **Automatic Priority Tagging**
   - No manual configuration needed
   - Content-aware detection
   - Format-specific logic

2. **Intelligent Scheduling**
   - Priority-ordered queues
   - WFQ bandwidth allocation
   - Network-aware decisions

3. **Real-Time Monitoring**
   - Transfer progress
   - Network quality
   - System health

4. **Universal Format Support**
   - Medical (HL7, DICOM, FHIR)
   - Disaster response
   - Engineering data
   - Images, videos, audio

5. **Resilience**
   - FEC for packet loss recovery
   - Adaptive behaviors
   - Smart buffering

## 🎥 Video Demo Script

1. **Opening** (0:00-0:30)
   - Show system overview
   - Explain use cases

2. **Priority Tagging** (0:30-1:30)
   - Run priority_scheduler example
   - Show different priorities

3. **Medical Data** (1:30-2:30)
   - Show HL7 processing
   - Show Code Blue → Critical

4. **Real-Time Status** (2:30-3:30)
   - Show monitoring dashboard
   - Show transfer progress

5. **Integration** (3:30-4:30)
   - Show complete workflow
   - Show all components working together

6. **Closing** (4:30-5:00)
   - Summary of features
   - Next steps

## 📝 Quick Reference

### Run Examples
```bash
# Priority scheduler
cargo run --example priority_scheduler

# Integrated workflow
cargo run --example integrated_workflow

# Patchy network
cargo run --example patchy_network_example
```

### Check Status
```bash
# Build status
cargo build --release

# Test status
cargo test

# Documentation
cargo doc --open
```

### Common Commands
```bash
# Build
cargo build --release

# Run
cargo run --example priority_scheduler

# Test
cargo test

# Clean
cargo clean
```

Ready to demonstrate! 🎬

