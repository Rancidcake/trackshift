# TrackShift Telemetry AI - Setup Guide

## Prerequisites

1. **Rust Toolchain**: Rust 1.84+ (or nightly for base64ct compatibility)
2. **Python 3.7+**: For generating ONNX models
3. **ONNX Runtime**: Automatically downloaded by `ort` crate

## Known Issue: base64ct Dependency

The `ort` crate depends on `base64ct v1.8.0` which requires Rust edition 2024 (not yet stable).

### Solution Options:

#### Option 1: Use Rust Nightly (Recommended for development)
```bash
rustup toolchain install nightly
rustup override set nightly
cargo build
```

#### Option 2: Manual Patch (Quick fix)
After first `cargo build` fails, run:
```powershell
# Windows PowerShell
.\scripts\fix_base64ct.ps1
cargo build
```

Or manually edit:
`%USERPROFILE%\.cargo\registry\src\index.crates.io-*\base64ct-1.8.0\Cargo.toml`

Change:
```toml
edition = "2024"
```
To:
```toml
edition = "2021"
```

#### Option 3: Wait for Stable Rust Update
Update Rust to the latest version when edition 2024 becomes stable.

## Setup Steps

1. **Generate ONNX Models**:
```bash
# Install Python dependencies
pip install numpy onnx

# Generate models
python scripts/create_onnx_models.py
```

This creates:
- `models/slm.onnx` - Decision model (270 inputs → 7 outputs)
- `models/embedder.onnx` - Embedding model (1024 inputs → 128 outputs)

2. **Build the Project**:
```bash
cargo build --release
```

3. **Run Tests**:
```bash
cargo test
```

4. **Run Example**:
```bash
cargo run --example scheduler_integration
```

## Project Structure

```
trackshift/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   └── telemetry_ai/
│       ├── mod.rs          # Main AI decision module
│       └── vector_store.rs # Simple vector store (replaces HNSW temporarily)
├── examples/
│   └── scheduler_integration.rs
├── scripts/
│   ├── create_onnx_models.py  # Generate ONNX models
│   └── fix_base64ct.ps1       # Fix dependency issue
└── models/
    ├── slm.onnx
    └── embedder.onnx
```

## Features Implemented

✅ **Telemetry AI Decision Module**
- ONNX model inference for routing/scheduling decisions
- Vector store for context retrieval (simple cosine similarity)
- Feature preprocessing (270 features: 14 numeric + 256 embeddings)
- Decision output parsing

✅ **Vector Store**
- Simple in-memory store using cosine similarity
- Can be swapped with HNSW when dependency issues resolved
- Thread-safe with `parking_lot::RwLock`

✅ **ONNX Models**
- Mock decision model (slm.onnx)
- Mock embedding model (embedder.onnx)
- Ready for replacement with trained models

## Usage Example

```rust
use trackshift::telemetry_ai::*;

// Initialize AI system
let ai_system = TelemetryAi::new("models/slm.onnx", "models/embedder.onnx")?;

// Prepare input
let ai_input = AiInput {
    rtt_ms: 15.0,
    jitter_ms: 2.5,
    // ... other fields
    embed_current: [0.5; 128],
    embed_context: [0.3; 128],
    // ...
};

// Make decision
let decision = ai_system.ai_decide(&ai_input)?;

// Use decision
println!("Route: {:?}", decision.route);
println!("WFQ Weights: P0={}, P1={}, P2={}", 
    decision.wfq_p0_weight, 
    decision.wfq_p1_weight, 
    decision.wfq_p2_weight);
```

## Performance Targets

- **Total Latency**: < 3ms
  - Embedding: ~1ms
  - Vector Search: ~0.1ms (simple implementation)
  - ONNX Inference: ~0.3ms
  - Scheduling Updates: immediate

## Next Steps

1. Replace mock ONNX models with trained models
2. Implement HNSW when dependency issues resolved
3. Add embedding model integration
4. Add performance benchmarks
5. Add integration tests with network simulation

## Troubleshooting

### Compilation Errors

**Error**: `feature 'edition2024' is required`
- **Solution**: See "Known Issue: base64ct Dependency" above

**Error**: Model file not found
- **Solution**: Run `python scripts/create_onnx_models.py`

### Runtime Errors

**Error**: Failed to load ONNX model
- **Solution**: Ensure model files exist in `models/` directory
- Check model file paths are correct

**Error**: Dimension mismatch
- **Solution**: Ensure input features match model expectations (270 features)

## License

[Your License Here]

