# 🧪 How to Run Tests

## Quick Start

### 1. Fix Windows SDK Issue (One-Time Setup)

**Option A: Install Windows SDK**
```powershell
# Download from: https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/
# Or use Visual Studio Installer to install "Windows 10/11 SDK"
```

**Option B: Use x86_64 Target (Easier)**
```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --target x86_64-pc-windows-msvc
cargo test --target x86_64-pc-windows-msvc
cargo run --target x86_64-pc-windows-msvc
```

**Option C: Use GNU Toolchain**
```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
cargo build
cargo test
cargo run
```

### 2. Verify Models Exist
```powershell
Test-Path models/slm.onnx
Test-Path models/embedder.onnx
# Should return: True True
```

If models don't exist:
```powershell
python scripts/create_onnx_models.py
```

### 3. Run Tests

**Unit Tests:**
```powershell
cargo test --lib
```

**All Tests:**
```powershell
cargo test
```

**With Output:**
```powershell
cargo test -- --nocapture
```

### 4. Run Examples

**Scheduler Integration:**
```powershell
cargo run --example scheduler_integration
```

**Patchy Network Example:**
```powershell
cargo run --example patchy_network_example
```

### 5. Run Main Program**
```powershell
cargo run
```

## Expected Test Output

### Unit Tests
```
running 5 tests
test telemetry_ai::tests::test_network_quality_assessment ... ok
test telemetry_ai::tests::test_redundancy_detection ... ok
test telemetry_ai::tests::test_vector_store ... ok
test telemetry_ai::tests::test_embedding_generation ... ok
test telemetry_ai::tests::test_ai_decision ... ok

test result: ok. 5 passed; 0 failed; 0 ignored
```

### Scheduler Integration Example
```
=== Iteration 1 ===
Collected network metrics: RTT=50ms, Throughput=100Mbps
AI Decision:
  Route: WiFi
  Severity: Low
  P2 Enabled: false
  Congestion Predicted: false
  WFQ Weights: P0=50, P1=30, P2=20
  Data Redundancy:
    Similarity Score: 45.23%
    Should Send: true
    Optimization: SendFull
  Network Quality: NetworkQuality { score: 0.85, ... }
  Should Buffer: false
  Retry Strategy: Immediate
Updated scheduler weights
Processing chunk...
```

### Patchy Network Example
```
🌐 Patchy Network Handling Example

=== Good Network ===
Network Quality:
  Score: 0.95 (95%)
  Patchy: false
  Connected: true
  Action: Conservative

Decision:
  Should Send: true
  Should Buffer: false
  Similarity: 45.23%
  Optimization: SendFull
  Retry Strategy: Immediate
  Route: WiFi

=== Patchy Network ===
Network Quality:
  Score: 0.45 (45%)
  Patchy: true
  Connected: true
  Action: Aggressive

Decision:
  Should Send: true
  Should Buffer: false
  Similarity: 45.23%
  Optimization: Compress
  Retry Strategy: Exponential
  Route: Multipath
```

## Test Scenarios

### 1. Redundancy Detection
- Send same data twice → Second should be skipped
- Send similar data → Should suggest delta/compress
- Send unique data → Should send full

### 2. Network Quality
- Good network → Conservative threshold (0.98)
- Patchy network → Aggressive threshold (0.90)
- Network down → Emergency threshold (0.85)

### 3. Buffering
- Network down → Should buffer
- Network recovers → Should flush buffer
- Critical data → Should prioritize

### 4. Retry Strategies
- Good network → Immediate retry
- Patchy network → Exponential backoff
- Critical data → Aggressive retry
- Network down → Buffer and wait

## Troubleshooting

### Issue: "cannot open input file 'kernel32.lib'"
**Solution**: Install Windows SDK or use x86_64 target (see Option B above)

### Issue: "base64ct requires Rust 1.85"
**Solution**: Already fixed! Run `cargo update -p base64ct --precise 1.7.3`

### Issue: "models not found"
**Solution**: Run `python scripts/create_onnx_models.py`

### Issue: "onnxruntime not found"
**Solution**: The `ort` crate handles this automatically. If issues persist, check your internet connection for downloading prebuilt binaries.

## Performance Testing

### Measure Latency
```rust
use std::time::Instant;

let start = Instant::now();
let decision = ai.process_chunk(chunk_data, metrics)?;
let elapsed = start.elapsed();
println!("Decision took: {:?}", elapsed);
// Expected: < 3ms
```

### Measure Throughput
```rust
let chunks_per_second = 1000.0 / elapsed.as_millis() as f32;
println!("Throughput: {:.1} chunks/sec", chunks_per_second);
// Expected: > 300 chunks/sec
```

## Continuous Testing

### Watch Mode (requires `cargo-watch`)
```powershell
cargo install cargo-watch
cargo watch -x test
```

### Benchmark Mode
```powershell
cargo bench
```

## ✅ Verification Checklist

Before deploying:
- [ ] All tests pass
- [ ] Examples run successfully
- [ ] Main program runs
- [ ] Models load correctly
- [ ] Performance meets targets (< 3ms)
- [ ] No memory leaks
- [ ] Error handling works
- [ ] Documentation is up to date

## 🎯 Success Criteria

✅ **Tests Pass**: All unit and integration tests pass
✅ **Examples Run**: Both examples execute successfully
✅ **Main Runs**: Interactive CLI works
✅ **Performance**: < 3ms per decision
✅ **No Errors**: Clean compilation and execution

## 📝 Test Results

See `FINAL_TEST_REPORT.md` for detailed test results.






