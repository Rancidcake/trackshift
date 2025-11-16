# Dashboard Guide

## 🎛️ Web Dashboard for PitlinkPQC

A comprehensive web-based dashboard for monitoring, controlling, and configuring the PitlinkPQC telemetry system.

## 🚀 Quick Start

### Start the Dashboard

```bash
cd /Users/mayankhete/trs/PitlinkPQC
cargo run --bin dashboard
```

Then open your browser to: **http://localhost:8080**

## 📊 Features

### 1. **Real-Time Monitoring**

#### Overview Tab
- **System Health**: Active transfers, queue size, error rate
- **Network Status**: RTT, throughput, quality score, patchy detection
- **Scheduler Stats**: Queue sizes, WFQ weights, total scheduled

#### Transfers Tab
- **Active Transfers**: List of all active transfers with:
  - Progress bars
  - Speed (Mbps)
  - Priority badges
  - Status indicators
  - Control buttons (Stop, Pause, Resume)

### 2. **Control Operations**

#### Start Transfer
- Enter data (text or base64)
- Select priority level
- Click "Start Transfer"

#### Control Transfers
- **Stop**: Cancel a transfer
- **Pause**: Pause a transfer
- **Resume**: Resume a paused transfer

### 3. **Configuration Management**

#### Compression Settings
- Enable/disable compression
- Choose algorithm: **LZ4**, **Zstd**, or **Auto**

#### Integrity Settings
- Enable/disable integrity checks
- Choose method: **Blake3** (recommended), **SHA256**, **CRC32**, **Checksum**, or **None**

#### Routing Settings
- Default route: **WiFi**, **Starlink**, or **Multipath**
- Enable/disable multipath

#### Priority Weights (WFQ)
- **P0 Weight**: Critical/High priority bandwidth %
- **P1 Weight**: Normal priority bandwidth %
- **P2 Weight**: Low/Bulk priority bandwidth %
- Enable/disable P2 (bulk transfers)

#### FEC Settings
- Enable/disable Forward Error Correction
- Redundancy percentage (0-100%)

### 4. **Method Selection**

View all available methods:
- Compression algorithms with descriptions
- Integrity methods with recommendations
- Routing options
- Priority levels

## 🔌 API Endpoints

### Status & Monitoring

```bash
# Get system status
GET /api/status

# Get all transfers
GET /api/transfers

# Get network status
GET /api/network

# Get system health
GET /api/health

# Get statistics
GET /api/stats
```

### Configuration

```bash
# Get configuration
GET /api/config

# Update configuration
POST /api/config
Content-Type: application/json

{
  "compression_enabled": true,
  "compression_algorithm": "Auto",
  "integrity_enabled": true,
  "integrity_method": "Blake3",
  "default_route": "WiFi",
  "p0_weight": 50,
  "p1_weight": 30,
  "p2_weight": 20
}
```

### Control

```bash
# Control transfers
POST /api/control
Content-Type: application/json

{
  "action": "start_transfer",
  "data": "base64_encoded_data",
  "priority": "High"
}

# Available actions:
# - "start_transfer"
# - "stop_transfer"
# - "pause_transfer"
# - "resume_transfer"
```

### Methods

```bash
# Get available methods
GET /api/methods
```

## 🎨 Dashboard Interface

### Tabs

1. **Overview**: System status, network, scheduler stats
2. **Transfers**: Active transfers and transfer control
3. **Configuration**: All system settings
4. **Methods**: Available methods and descriptions

### Real-Time Updates

- Status updates every 2 seconds
- Automatic refresh of active data
- Live progress bars
- Real-time network metrics

## 📱 Usage Examples

### Example 1: Monitor System

1. Open dashboard: http://localhost:8080
2. View **Overview** tab
3. See real-time:
   - Active transfers count
   - Network quality
   - Queue sizes

### Example 2: Start a Transfer

1. Go to **Transfers** tab
2. Enter data in "Start New Transfer" section
3. Select priority (e.g., "High")
4. Click "Start Transfer"
5. Watch progress in real-time

### Example 3: Change Compression Method

1. Go to **Configuration** tab
2. Find "Compression Settings"
3. Select algorithm (e.g., "Zstd")
4. Click "Save Configuration"
5. New transfers will use Zstd

### Example 4: Adjust Priority Weights

1. Go to **Configuration** tab
2. Find "Priority Weights (WFQ)"
3. Adjust P0, P1, P2 weights (must sum to ~100)
4. Click "Save Configuration"
5. Scheduler will use new weights

## 🔧 Configuration Options

### Compression Algorithms

| Algorithm | Speed | Ratio | Best For |
|-----------|-------|-------|----------|
| **LZ4** | Fast | Lower | Small files, good networks |
| **Zstd** | Balanced | Better | Large files, poor networks |
| **Auto** | AI-driven | Optimal | All scenarios (recommended) |

### Integrity Methods

| Method | Speed | Security | Best For |
|--------|-------|----------|----------|
| **Blake3** | Fastest | High | All files (recommended) |
| **SHA256** | Slower | Highest | Critical files |
| **CRC32** | Fast | Medium | Large files |
| **Checksum** | Fastest | Low | Small files |

### Routing Options

| Route | Latency | Throughput | Best For |
|-------|---------|------------|----------|
| **WiFi** | Low | High | Good signal areas |
| **Starlink** | Medium | High | Remote areas |
| **Multipath** | Variable | Highest | Critical data |

## 🎯 Use Cases

### Medical Facility
- Monitor patient data transfers
- Set integrity to SHA256 for critical data
- Use High priority for vital signs

### Disaster Site
- Monitor emergency alerts
- Use Critical priority for evacuations
- Enable FEC for unreliable networks

### Remote Engineering
- Monitor sensor data
- Use Auto compression
- Adjust WFQ weights based on network

## 🚀 Deployment

### Development

```bash
cargo run --bin dashboard
```

### Production

```bash
# Build release
cargo build --release -p dashboard

# Run
./target/release/dashboard
```

### Docker

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p dashboard

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/dashboard /usr/local/bin/
COPY --from=builder /app/dashboard/static /app/static
WORKDIR /app
CMD ["dashboard"]
```

## 📊 Screenshots & Features

### Overview Dashboard
- Real-time system health metrics
- Network quality indicators
- Scheduler queue status
- Color-coded status indicators

### Transfer Management
- Progress bars for each transfer
- Priority badges (Critical, High, Normal, Low, Bulk)
- Speed indicators
- Control buttons

### Configuration Panel
- Organized sections
- Checkboxes for enable/disable
- Dropdowns for method selection
- Number inputs for weights

## 🔐 Security Notes

- Dashboard runs on localhost by default
- For production, add authentication
- Use HTTPS in production
- Restrict access to authorized users

## 🐛 Troubleshooting

### Dashboard won't start
```bash
# Check if port 8080 is available
lsof -i :8080

# Use different port (modify main.rs)
.bind("0.0.0.0:8081")?
```

### API errors
- Check that models are in `brain/models/`
- Verify AI system is initialized
- Check browser console for errors

### No data showing
- Ensure transfers are registered
- Check network status updates
- Verify scheduler is running

## ✅ Quick Reference

```bash
# Start dashboard
cargo run --bin dashboard

# Access dashboard
open http://localhost:8080

# API endpoints
curl http://localhost:8080/api/status
curl http://localhost:8080/api/transfers
curl http://localhost:8080/api/config
```

## 🎬 Demo Flow

1. **Start Dashboard**
   ```bash
   cargo run --bin dashboard
   ```

2. **Open Browser**
   - Navigate to http://localhost:8080
   - See real-time status

3. **Start Transfer**
   - Go to Transfers tab
   - Enter data
   - Select priority
   - Click Start

4. **Monitor Progress**
   - Watch progress bar
   - See speed
   - Check status

5. **Configure System**
   - Go to Configuration tab
   - Change settings
   - Save configuration

6. **View Methods**
   - Go to Methods tab
   - See all available options
   - Understand each method

The dashboard provides **complete control** over the PitlinkPQC system! 🎛️
