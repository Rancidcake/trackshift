# TrackShift - Telemetry AI Decision Model

A Rust-based real-time telemetry pipeline with AI-powered decision making for network routing, scheduling, and priority management.

## Overview

The `telemetry_ai` module acts as the brain of the telemetry system, making real-time decisions for:
- **Network path selection** (5G / WiFi / Starlink / Multipath)
- **Priority scheduling** (WFQ weight tuning)
- **Bulk transfer control** (P2 enable/disable)
- **Congestion prediction** (200-500ms ahead)
- **Severity classification** (High/Low)

## Architecture

```
Telemetry Chunk → Embedding Model → HNSW Context → AI Decision Model → Scheduler/Router
```

### Key Components

1. **TelemetryAi**: Main decision engine
2. **HnswContextStore**: Vector store for contextual embeddings
3. **AiInput**: Input features (270 dimensions)
4. **AiDecision**: Output decisions

## Features

- **ONNX Model Inference**: Uses `onnxruntime` for fast model execution
- **HNSW Vector Search**: Fast approximate nearest neighbor search for context retrieval
- **Low Latency**: Target < 3ms total inference time
- **Thread-Safe**: Uses `Arc` and `RwLock` for concurrent access

## Requirements

### Model Files

You need two ONNX model files:

1. **`slm.onnx`**: Decision model (takes 270 features, outputs 7 decisions)
2. **`embedder.onnx`**: Embedding model (converts telemetry chunks to 128-dim vectors)

Place these in the `models/` directory:

```
models/
├── slm.onnx
└── embedder.onnx
```

### Model Output Format

The `slm.onnx` model should output a tensor of shape `(1, 7)` with values:
- `[0]`: Route decision (0=5G, 1=WiFi, 2=Starlink, 3=Multipath)
- `[1]`: Severity (0=High, 1=Low)
- `[2]`: P2 enable (0.0=disabled, 1.0=enabled)
- `[3]`: Congestion predicted (0.0=no, 1.0=yes)
- `[4]`: WFQ P0 weight
- `[5]`: WFQ P1 weight
- `[6]`: WFQ P2 weight

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd trackshift

# Build the project
cargo build --release

# Run the example
cargo run --example scheduler_integration
```

## Usage

### Basic Usage

```rust
use trackshift::telemetry_ai::*;

// Initialize the AI system
let ai_system = TelemetryAi::new("models/slm.onnx", "models/embedder.onnx")?;

// Prepare input
let ai_input = AiInput {
    rtt_ms: 15.0,
    jitter_ms: 2.5,
    loss_rate: 0.001,
    throughput_mbps: 150.0,
    retransmissions: 2.0,
    queue_p0: 5.0,
    queue_p1: 10.0,
    queue_p2: 20.0,
    p0_rate: 1000.0,
    p1_rate: 2000.0,
    p2_rate: 3000.0,
    wifi_signal: -45.0,
    fiveg_signal: -55.0,
    starlink_latency: 35.0,
    session_state: 0.0, // 0=ACTIVE, 1=BREAK
    embed_current: [0.5; 128], // from embedder.onnx
    embed_context: [0.3; 128], // from HNSW (auto-retrieved)
    chunk_size: 1024.0,
    retries: 0.0,
};

// Make decision
let decision = ai_system.ai_decide(&ai_input)?;

// Use decision
scheduler.update_weights(
    decision.wfq_p0_weight,
    decision.wfq_p1_weight,
    decision.wfq_p2_weight,
);
scheduler.set_p2(decision.p2_enable);
router.switch_path(decision.route);
```

### Scheduler Loop Integration

See `examples/scheduler_integration.rs` for a complete example showing:
- Network metrics collection
- Embedding generation
- HNSW context retrieval
- AI decision making
- Scheduler/router updates

## Input Features (270 total)

### Numeric Features (17)
1. `rtt_ms`: Round-trip time in milliseconds
2. `jitter_ms`: Jitter in milliseconds
3. `loss_rate`: Packet loss rate (0.0-1.0)
4. `throughput_mbps`: Throughput in Mbps
5. `retransmissions`: Number of retransmissions
6. `queue_p0`: Queue depth for priority 0
7. `queue_p1`: Queue depth for priority 1
8. `queue_p2`: Queue depth for priority 2
9. `p0_rate`: Rate for priority 0
10. `p1_rate`: Rate for priority 1
11. `p2_rate`: Rate for priority 2
12. `wifi_signal`: WiFi signal strength (dBm)
13. `fiveg_signal`: 5G signal strength (dBm)
14. `starlink_latency`: Starlink latency in ms
15. `session_state`: 0=ACTIVE, 1=BREAK
16. `chunk_size`: Size of telemetry chunk in bytes
17. `retries`: Number of retry attempts

### Embedding Features (256)
- `embed_current`: 128-dim embedding from current chunk (from embedder.onnx)
- `embed_context`: 128-dim embedding from HNSW nearest neighbor

## Performance Targets

- **Embedding**: ~1ms
- **HNSW Query**: ~0.1ms
- **ONNX Inference**: ~0.3ms
- **Total**: < 3ms

## Dependencies

- `ort = "2.0.0-rc.10"`: ONNX Runtime for Rust (model inference)
- `ndarray = "0.15"`: Array operations
- `anyhow = "1.0"`: Error handling
- `parking_lot = "0.12"`: Fast RwLock implementation

**Note**: HNSW is temporarily replaced with a simple cosine similarity vector store due to dependency compatibility issues. See SETUP.md for details.

## Testing

```bash
# Run unit tests
cargo test

# Run with output
cargo test -- --nocapture
```

## Project Structure

```
trackshift/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   └── telemetry_ai/
│       └── mod.rs
├── examples/
│   └── scheduler_integration.rs
└── models/
    ├── slm.onnx          # (you need to provide)
    └── embedder.onnx     # (you need to provide)
```

## Setup

See [SETUP.md](SETUP.md) for detailed setup instructions, including:
- Prerequisites
- ONNX model generation
- Known dependency issues and solutions
- Troubleshooting guide

## Notes

- The vector store starts empty. You should populate it with historical embeddings during initialization or runtime using `insert_embedding()`.
- Model output parsing is configured for the mock models. Adjust based on your actual trained model's output format.
- The embedding model (`embedder.onnx`) is provided but not directly integrated - you'll need to call it separately to generate `embed_current` values.
- Currently uses a simple cosine similarity vector store. Can be swapped with HNSW when dependency issues are resolved.

## License

[Your License Here]

