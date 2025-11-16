# Data Formats Reference

This document describes all data formats used in the telemetry AI system.

## 📥 Input Data Format

### 1. **AiInput Struct** (Rust)
The main input structure passed to the AI decision function:

```rust
pub struct AiInput {
    // Network Metrics (f32 = 32-bit floating point)
    pub rtt_ms: f32,                    // Round-trip time in milliseconds
    pub jitter_ms: f32,                 // Jitter in milliseconds
    pub loss_rate: f32,                 // Packet loss rate (0.0-1.0)
    pub throughput_mbps: f32,           // Throughput in Mbps
    pub retransmissions: f32,           // Number of retransmissions
    
    // Queue Depths (f32)
    pub queue_p0: f32,                  // Priority 0 queue depth
    pub queue_p1: f32,                  // Priority 1 queue depth
    pub queue_p2: f32,                  // Priority 2 queue depth
    
    // Data Rates (f32)
    pub p0_rate: f32,                   // Priority 0 data rate
    pub p1_rate: f32,                   // Priority 1 data rate
    pub p2_rate: f32,                   // Priority 2 data rate
    
    // Signal Strengths (f32, in dBm)
    pub wifi_signal: f32,               // WiFi signal strength (e.g., -45.0)
    pub fiveg_signal: f32,              // 5G signal strength (e.g., -55.0)
    pub starlink_latency: f32,          // Starlink latency in ms
    
    // Session State (f32)
    pub session_state: f32,             // 0.0 = ACTIVE, 1.0 = BREAK
    
    // Embeddings (128-dimensional arrays)
    pub embed_current: [f32; 128],       // Current chunk embedding
    pub embed_context: [f32; 128],       // Historical context embedding
    
    // Chunk Metadata (f32)
    pub chunk_size: f32,                // Size in bytes
    pub retries: f32,                   // Number of retry attempts
}
```

**Data Types:**
- `f32` = 32-bit floating point (single precision)
- `[f32; 128]` = Fixed-size array of 128 floats

### 2. **Telemetry Chunk** (Raw Input)
The raw telemetry data before processing:

```rust
struct TelemetryChunk {
    data: Vec<u8>,      // Raw bytes (any format: JSON, binary, text, etc.)
    size: usize,        // Size in bytes
}
```

**Format:** Raw bytes (`Vec<u8>`) - can be:
- JSON strings
- Binary protocol buffers
- Text logs
- Binary telemetry data
- Any format (gets converted to embedding)

### 3. **Network Metrics** (Collected Data)
Metrics collected from the network:

```rust
struct NetworkMetrics {
    rtt_ms: f32,
    jitter_ms: f32,
    loss_rate: f32,
    throughput_mbps: f32,
    retransmissions: f32,
    queue_p0: f32,
    queue_p1: f32,
    queue_p2: f32,
    p0_rate: f32,
    p1_rate: f32,
    p2_rate: f32,
    wifi_signal: f32,        // dBm (typically -30 to -90)
    fiveg_signal: f32,       // dBm (typically -30 to -90)
    starlink_latency: f32,   // milliseconds
}
```

## 🔄 Internal Data Formats

### 4. **Feature Vector** (Preprocessed)
After preprocessing, converted to a flat vector:

```rust
Vec<f32>  // Length: 270 elements
```

**Structure:**
```
[0-13]:   14 numeric features (rtt_ms, jitter_ms, ..., starlink_latency)
[14-141]: 128 embedding_current values
[142-269]: 128 embedding_context values
```

**Total:** 270 `f32` values (1,080 bytes)

### 5. **ONNX Model Input** (Tensor Format)
Format sent to ONNX model:

```rust
Array2<f32>  // Shape: (1, 270)
// Batch size: 1
// Features: 270
```

**ONNX Tensor Format:**
- Shape: `[1, 270]`
- Data Type: `FLOAT` (32-bit)
- Layout: Row-major (C-style)
- Size: 1,080 bytes

### 6. **Embedding Format**
128-dimensional vectors:

```rust
[f32; 128]  // Fixed-size array
```

**Values:** Typically normalized between -1.0 and 1.0 (from Tanh activation)

## 📤 Output Data Format

### 7. **AiDecision Struct** (Rust)
The decision output from the AI:

```rust
pub struct AiDecision {
    pub route: RouteDecision,           // Enum: FiveG, WiFi, Starlink, Multipath
    pub severity: Severity,             // Enum: High, Low
    pub p2_enable: bool,                // Boolean: true/false
    pub congestion_predicted: bool,      // Boolean: true/false
    pub wfq_p0_weight: u32,            // Unsigned 32-bit integer (0-100)
    pub wfq_p1_weight: u32,            // Unsigned 32-bit integer (0-100)
    pub wfq_p2_weight: u32,            // Unsigned 32-bit integer (0-100)
}
```

**Data Types:**
- `RouteDecision`: Enum (0-3)
- `Severity`: Enum (0-1)
- `bool`: Boolean (true/false)
- `u32`: Unsigned 32-bit integer

### 8. **ONNX Model Output** (Tensor Format)
Format received from ONNX model:

```rust
ArrayView2<f32>  // Shape: (1, 7)
```

**Structure:**
```
[0]: route (f32, 0.0-1.0, mapped to 0-3)
[1]: severity (f32, 0.0-1.0, mapped to 0-1)
[2]: p2_enable (f32, 0.0-1.0, threshold > 0.5)
[3]: congestion_predicted (f32, 0.0-1.0, threshold > 0.5)
[4]: wfq_p0_weight (f32, 0.0-100.0)
[5]: wfq_p1_weight (f32, 0.0-100.0)
[6]: wfq_p2_weight (f32, 0.0-100.0)
```

**ONNX Tensor Format:**
- Shape: `[1, 7]`
- Data Type: `FLOAT` (32-bit)
- Size: 28 bytes

## 📊 Data Flow Example

### Example Input Values:

```rust
AiInput {
    rtt_ms: 15.5,                    // f32: 15.5 milliseconds
    jitter_ms: 2.3,                  // f32: 2.3 milliseconds
    loss_rate: 0.001,                // f32: 0.1% packet loss
    throughput_mbps: 150.0,          // f32: 150 Mbps
    retransmissions: 2.0,            // f32: 2 retransmissions
    queue_p0: 5.0,                  // f32: 5 packets
    queue_p1: 10.0,                 // f32: 10 packets
    queue_p2: 20.0,                 // f32: 20 packets
    p0_rate: 1000.0,                // f32: 1000 bytes/sec
    p1_rate: 2000.0,                // f32: 2000 bytes/sec
    p2_rate: 3000.0,                // f32: 3000 bytes/sec
    wifi_signal: -45.0,             // f32: -45 dBm (good signal)
    fiveg_signal: -55.0,            // f32: -55 dBm (moderate signal)
    starlink_latency: 35.0,         // f32: 35 ms
    session_state: 0.0,             // f32: ACTIVE
    embed_current: [0.5; 128],     // [f32; 128]: All 0.5
    embed_context: [0.3; 128],      // [f32; 128]: All 0.3
    chunk_size: 1024.0,             // f32: 1024 bytes
    retries: 0.0,                   // f32: No retries
}
```

### Example Output Values:

```rust
AiDecision {
    route: RouteDecision::WiFi,     // Enum: WiFi (value 1)
    severity: Severity::Low,        // Enum: Low (value 1)
    p2_enable: true,                // bool: Bulk transfers allowed
    congestion_predicted: false,    // bool: No congestion expected
    wfq_p0_weight: 50,             // u32: 50% bandwidth
    wfq_p1_weight: 30,             // u32: 30% bandwidth
    wfq_p2_weight: 20,             // u32: 20% bandwidth
}
```

## 🔢 Numeric Ranges

### Input Ranges (Typical Values):

| Field | Type | Typical Range | Unit |
|-------|------|---------------|------|
| `rtt_ms` | f32 | 5.0 - 200.0 | milliseconds |
| `jitter_ms` | f32 | 0.0 - 50.0 | milliseconds |
| `loss_rate` | f32 | 0.0 - 1.0 | ratio (0% - 100%) |
| `throughput_mbps` | f32 | 0.0 - 1000.0 | Mbps |
| `retransmissions` | f32 | 0.0 - 100.0 | count |
| `queue_p0/p1/p2` | f32 | 0.0 - 1000.0 | packets |
| `p0/p1/p2_rate` | f32 | 0.0 - 10000.0 | bytes/sec |
| `wifi_signal` | f32 | -90.0 - -30.0 | dBm |
| `fiveg_signal` | f32 | -90.0 - -30.0 | dBm |
| `starlink_latency` | f32 | 20.0 - 100.0 | milliseconds |
| `session_state` | f32 | 0.0 or 1.0 | ACTIVE/BREAK |
| `embed_current/context` | [f32; 128] | -1.0 - 1.0 | normalized |
| `chunk_size` | f32 | 0.0 - 1000000.0 | bytes |
| `retries` | f32 | 0.0 - 10.0 | count |

### Output Ranges:

| Field | Type | Range | Values |
|-------|------|-------|--------|
| `route` | RouteDecision | 0-3 | 0=FiveG, 1=WiFi, 2=Starlink, 3=Multipath |
| `severity` | Severity | 0-1 | 0=High, 1=Low |
| `p2_enable` | bool | true/false | Boolean |
| `congestion_predicted` | bool | true/false | Boolean |
| `wfq_p0_weight` | u32 | 0-100 | Percentage |
| `wfq_p1_weight` | u32 | 0-100 | Percentage |
| `wfq_p2_weight` | u32 | 0-100 | Percentage |

## 📝 JSON Representation (for API/Serialization)

### Output JSON Format (for External Systems)

This is the standard JSON format for AI decision output that other systems can consume:

```json
{
  "route": "WiFi",
  "severity": "Low",
  "p2_enable": true,
  "congestion_predicted": false,
  "wfq_p0_weight": 50,
  "wfq_p1_weight": 30,
  "wfq_p2_weight": 20
}
```

**Field Descriptions:**

| Field | Type | Values | Description |
|-------|------|--------|-------------|
| `route` | string | `"FiveG"`, `"WiFi"`, `"Starlink"`, `"Multipath"` | Selected network route |
| `severity` | string | `"High"`, `"Low"` | Network condition severity |
| `p2_enable` | boolean | `true`, `false` | Whether priority 2 (bulk) traffic is enabled |
| `congestion_predicted` | boolean | `true`, `false` | Whether congestion is predicted |
| `wfq_p0_weight` | integer | 0-100 | Weighted Fair Queue weight for priority 0 (percentage) |
| `wfq_p1_weight` | integer | 0-100 | Weighted Fair Queue weight for priority 1 (percentage) |
| `wfq_p2_weight` | integer | 0-100 | Weighted Fair Queue weight for priority 2 (percentage) |

**Note:** The three `wfq_*_weight` values should sum to 100 (or close to it) for proper bandwidth allocation.

### Complete JSON Example (Input + Output)

If you need to serialize both input and output:

```json
{
  "input": {
    "rtt_ms": 15.5,
    "jitter_ms": 2.3,
    "loss_rate": 0.001,
    "throughput_mbps": 150.0,
    "retransmissions": 2.0,
    "queue_p0": 5.0,
    "queue_p1": 10.0,
    "queue_p2": 20.0,
    "p0_rate": 1000.0,
    "p1_rate": 2000.0,
    "p2_rate": 3000.0,
    "wifi_signal": -45.0,
    "fiveg_signal": -55.0,
    "starlink_latency": 35.0,
    "session_state": 0.0,
    "embed_current": [0.5, 0.5, ...],  // 128 values
    "embed_context": [0.3, 0.3, ...],  // 128 values
    "chunk_size": 1024.0,
    "retries": 0.0
  },
  "output": {
    "route": "WiFi",
    "severity": "Low",
    "p2_enable": true,
    "congestion_predicted": false,
    "wfq_p0_weight": 50,
    "wfq_p1_weight": 30,
    "wfq_p2_weight": 20
  }
}
```

### JSON Schema (for Validation)

For systems that need schema validation, here's a JSON Schema:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["route", "severity", "p2_enable", "congestion_predicted", "wfq_p0_weight", "wfq_p1_weight", "wfq_p2_weight"],
  "properties": {
    "route": {
      "type": "string",
      "enum": ["FiveG", "WiFi", "Starlink", "Multipath"]
    },
    "severity": {
      "type": "string",
      "enum": ["High", "Low"]
    },
    "p2_enable": {
      "type": "boolean"
    },
    "congestion_predicted": {
      "type": "boolean"
    },
    "wfq_p0_weight": {
      "type": "integer",
      "minimum": 0,
      "maximum": 100
    },
    "wfq_p1_weight": {
      "type": "integer",
      "minimum": 0,
      "maximum": 100
    },
    "wfq_p2_weight": {
      "type": "integer",
      "minimum": 0,
      "maximum": 100
    }
  }
}
```

## 🔍 Binary Format (ONNX Tensor)

### Input Tensor (to model):
- **Format**: Row-major float32 array
- **Shape**: `[1, 270]`
- **Size**: 1,080 bytes (270 × 4 bytes)
- **Endianness**: Little-endian (x86/x64)

### Output Tensor (from model):
- **Format**: Row-major float32 array
- **Shape**: `[1, 7]`
- **Size**: 28 bytes (7 × 4 bytes)
- **Endianness**: Little-endian (x86/x64)

## 📦 Summary

| Stage | Format | Type | Size |
|-------|--------|------|------|
| Raw Chunk | `Vec<u8>` | Binary | Variable |
| Network Metrics | `struct` | Rust struct | ~200 bytes |
| Embeddings | `[f32; 128]` | Array | 512 bytes each |
| AiInput | `struct` | Rust struct | ~1,200 bytes |
| Feature Vector | `Vec<f32>` | Array | 1,080 bytes |
| ONNX Input | `Array2<f32>` | Tensor | 1,080 bytes |
| ONNX Output | `ArrayView2<f32>` | Tensor | 28 bytes |
| AiDecision | `struct` | Rust struct | ~32 bytes |

All floating-point values use **IEEE 754 single precision (32-bit)** format.

