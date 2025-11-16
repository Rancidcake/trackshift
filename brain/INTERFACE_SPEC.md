# Interface Specification - Component Data Contracts

This document defines the data formats and interfaces between `telemetry_ai` and other system components (scheduler, router, alert system, etc.).

## 🔄 Data Flow Overview

```
┌─────────────────┐
│ Network Metrics │ ──> f32 values (struct)
└─────────────────┘
         │
         ▼
┌─────────────────┐
│  Telemetry AI   │ ──> AiDecision struct
└─────────────────┘
         │
         ├──> WFQ Scheduler ──> u32 weights, bool
         ├──> Router         ──> RouteDecision enum
         └──> Alert System  ──> Severity enum, bool
```

## 📤 Output: AiDecision Struct

The `telemetry_ai` module outputs a single `AiDecision` struct that gets split and sent to different components:

```rust
pub struct AiDecision {
    pub route: RouteDecision,           // → Router
    pub severity: Severity,             // → Alert System
    pub p2_enable: bool,                // → WFQ Scheduler
    pub congestion_predicted: bool,     // → Congestion Handler
    pub wfq_p0_weight: u32,            // → WFQ Scheduler
    pub wfq_p1_weight: u32,            // → WFQ Scheduler
    pub wfq_p2_weight: u32,            // → WFQ Scheduler
}
```

## 🎯 Component Interfaces

### 1. **WFQ Scheduler Interface**

**Expected Data Types:**

```rust
// Weighted Fair Queue Scheduler expects:
struct WfqSchedulerInput {
    p0_weight: u32,      // Priority 0 weight (0-100)
    p1_weight: u32,      // Priority 1 weight (0-100)
    p2_weight: u32,      // Priority 2 weight (0-100)
    p2_enabled: bool,    // Whether P2 (bulk) is enabled
}
```

**Function Signature:**
```rust
impl WfqScheduler {
    /// Update WFQ weights for priority queues
    /// 
    /// # Arguments
    /// * `p0_weight` - Weight for priority 0 (highest), range: 0-100
    /// * `p1_weight` - Weight for priority 1 (medium), range: 0-100
    /// * `p2_weight` - Weight for priority 2 (lowest/bulk), range: 0-100
    /// 
    /// # Returns
    /// Result indicating success/failure
    fn update_weights(&mut self, p0: u32, p1: u32, p2: u32) -> Result<()>;
    
    /// Enable or disable P2 (bulk transfer) queue
    /// 
    /// # Arguments
    /// * `enabled` - true to allow bulk transfers, false to block
    fn set_p2_enabled(&mut self, enabled: bool) -> Result<()>;
}
```

**Usage Example:**
```rust
// From AiDecision to WFQ Scheduler
let decision = ai_system.ai_decide(&input)?;

scheduler.update_weights(
    decision.wfq_p0_weight,  // u32: e.g., 50
    decision.wfq_p1_weight,  // u32: e.g., 30
    decision.wfq_p2_weight,  // u32: e.g., 20
)?;

scheduler.set_p2_enabled(decision.p2_enable)?;  // bool: true/false
```

**Data Format Details:**
- **Type**: `u32` (unsigned 32-bit integer)
- **Range**: 0-100 (represents percentage/weight)
- **Semantics**: Higher weight = more bandwidth allocated
- **Validation**: Weights should sum to ~100 (but not enforced)
- **Example**: `p0=50, p1=30, p2=20` means 50% to P0, 30% to P1, 20% to P2

### 2. **Router Interface**

**Expected Data Types:**

```rust
// Router expects:
enum RouteDecision {
    FiveG = 0,      // Use 5G network
    WiFi = 1,       // Use WiFi network
    Starlink = 2,   // Use Starlink network
    Multipath = 3,  // Use multiple paths simultaneously
}
```

**Function Signature:**
```rust
impl Router {
    /// Switch to a new network path
    /// 
    /// # Arguments
    /// * `route` - RouteDecision enum indicating which path to use
    /// 
    /// # Returns
    /// Result indicating success/failure of path switch
    fn switch_path(&mut self, route: RouteDecision) -> Result<()>;
    
    /// Get current active route
    fn current_route(&self) -> RouteDecision;
}
```

**Usage Example:**
```rust
// From AiDecision to Router
let decision = ai_system.ai_decide(&input)?;

router.switch_path(decision.route)?;  
// RouteDecision::WiFi, RouteDecision::FiveG, etc.
```

**Data Format Details:**
- **Type**: `RouteDecision` enum
- **Values**: 
  - `RouteDecision::FiveG` (0)
  - `RouteDecision::WiFi` (1)
  - `RouteDecision::Starlink` (2)
  - `RouteDecision::Multipath` (3)
- **Conversion**: Can convert from `u32` using `RouteDecision::from(value)`
- **Default**: Falls back to `FiveG` if invalid value

### 3. **Alert System Interface**

**Expected Data Types:**

```rust
// Alert System expects:
enum Severity {
    High = 0,  // Urgent action required
    Low = 1,   // Normal operation
}

struct AlertInput {
    severity: Severity,
    congestion_predicted: bool,
}
```

**Function Signature:**
```rust
impl AlertSystem {
    /// Trigger alert based on severity
    /// 
    /// # Arguments
    /// * `severity` - Severity enum (High/Low)
    fn trigger_alert(&self, severity: Severity) -> Result<()>;
    
    /// Check if congestion is predicted
    fn is_congestion_predicted(&self) -> bool;
}
```

**Usage Example:**
```rust
// From AiDecision to Alert System
let decision = ai_system.ai_decide(&input)?;

if decision.severity == Severity::High {
    alert_system.trigger_alert(decision.severity)?;
}

if decision.congestion_predicted {
    // Take preventive action
    congestion_handler.prepare_for_congestion()?;
}
```

**Data Format Details:**
- **Type**: `Severity` enum
- **Values**: 
  - `Severity::High` (0) - Urgent
  - `Severity::Low` (1) - Normal
- **Type**: `bool` for congestion prediction
- **Values**: `true` = congestion predicted, `false` = no congestion

### 4. **Congestion Handler Interface**

**Expected Data Types:**

```rust
// Congestion Handler expects:
struct CongestionInput {
    predicted: bool,  // true if congestion predicted in next 200-500ms
}
```

**Function Signature:**
```rust
impl CongestionHandler {
    /// Prepare for predicted congestion
    fn prepare_for_congestion(&mut self) -> Result<()>;
    
    /// Check congestion prediction status
    fn is_congestion_predicted(&self) -> bool;
}
```

## 📋 Complete Integration Example

```rust
use trackshift::telemetry_ai::*;

// 1. Get AI decision
let decision = ai_system.ai_decide(&ai_input)?;

// 2. Send to WFQ Scheduler
scheduler.update_weights(
    decision.wfq_p0_weight,  // u32: 0-100
    decision.wfq_p1_weight,  // u32: 0-100
    decision.wfq_p2_weight,  // u32: 0-100
)?;
scheduler.set_p2_enabled(decision.p2_enable)?;  // bool

// 3. Send to Router
router.switch_path(decision.route)?;  // RouteDecision enum

// 4. Send to Alert System
if decision.severity == Severity::High {
    alert_system.trigger_alert(decision.severity)?;
}

// 5. Send to Congestion Handler
if decision.congestion_predicted {
    congestion_handler.prepare_for_congestion()?;
}
```

## 🔢 Data Type Summary Table

| Component | Field | Type | Range/Values | Size |
|-----------|-------|------|--------------|------|
| **WFQ Scheduler** | `p0_weight` | `u32` | 0-100 | 4 bytes |
| | `p1_weight` | `u32` | 0-100 | 4 bytes |
| | `p2_weight` | `u32` | 0-100 | 4 bytes |
| | `p2_enabled` | `bool` | true/false | 1 byte |
| **Router** | `route` | `RouteDecision` | 0-3 (enum) | 1 byte |
| **Alert System** | `severity` | `Severity` | 0-1 (enum) | 1 byte |
| **Congestion Handler** | `congestion_predicted` | `bool` | true/false | 1 byte |

## 📦 Serialization Formats

### JSON Format (for API/Inter-process Communication)

```json
{
  "wfq_weights": {
    "p0": 50,
    "p1": 30,
    "p2": 20
  },
  "p2_enabled": true,
  "route": "WiFi",
  "severity": "Low",
  "congestion_predicted": false
}
```

### Binary Format (for High-Performance IPC)

```
[0-3]:   p0_weight (u32, little-endian)
[4-7]:   p1_weight (u32, little-endian)
[8-11]:  p2_weight (u32, little-endian)
[12]:    p2_enabled (u8: 0=false, 1=true)
[13]:    route (u8: 0-3)
[14]:    severity (u8: 0=High, 1=Low)
[15]:    congestion_predicted (u8: 0=false, 1=true)
```

**Total Size**: 16 bytes

### MessagePack Format (Compact Binary)

```rust
// Using rmp_serde or similar
let encoded = rmp_serde::to_vec(&decision)?;
// Compact binary representation
```

## 🔌 Protocol Buffers (Optional)

If you want to use protobuf for inter-component communication:

```protobuf
syntax = "proto3";

message AiDecision {
    RouteDecision route = 1;
    Severity severity = 2;
    bool p2_enable = 3;
    bool congestion_predicted = 4;
    uint32 wfq_p0_weight = 5;
    uint32 wfq_p1_weight = 6;
    uint32 wfq_p2_weight = 7;
}

enum RouteDecision {
    FIVE_G = 0;
    WIFI = 1;
    STARLINK = 2;
    MULTIPATH = 3;
}

enum Severity {
    HIGH = 0;
    LOW = 1;
}
```

## ⚡ Performance Considerations

### Zero-Copy Passing (Same Process)
- Pass `AiDecision` struct directly (32 bytes)
- No serialization overhead
- Fastest option

### Shared Memory (Inter-Process)
- Use fixed-size binary format (16 bytes)
- Map to shared memory region
- Atomic operations for thread safety

### Network IPC (Remote)
- Use JSON for debugging
- Use MessagePack for production
- Use Protobuf for type safety

## 🛡️ Validation & Error Handling

### WFQ Scheduler Validation:
```rust
fn validate_weights(p0: u32, p1: u32, p2: u32) -> Result<()> {
    if p0 > 100 || p1 > 100 || p2 > 100 {
        return Err("Weights must be <= 100".into());
    }
    // Optional: check if sum is reasonable
    Ok(())
}
```

### Router Validation:
```rust
fn validate_route(route: RouteDecision) -> Result<()> {
    // RouteDecision enum is always valid (type-safe)
    Ok(())
}
```

## 📝 Summary

**To WFQ Scheduler:**
- 3 × `u32` weights (0-100)
- 1 × `bool` for P2 enable

**To Router:**
- 1 × `RouteDecision` enum (0-3)

**To Alert System:**
- 1 × `Severity` enum (0-1)
- 1 × `bool` for congestion prediction

All data types are **Rust native types** - no conversion needed when components are in the same process!

