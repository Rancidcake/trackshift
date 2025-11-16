# Real Deployment Guide - Client-Server Setup

## Overview

This guide sets up a **real working system** where:
- **Server Device**: Runs QUIC-FEC server + Dashboard API
- **Client Device**: Runs Next.js dashboard + File transfer client
- **Real Data**: All metrics come from actual system, no fake data
- **Real Transfers**: Actual file transfers between devices

## Architecture

```
┌─────────────────────────────────┐
│     Server Device               │
│  ┌───────────────────────────┐  │
│  │  QUIC-FEC Server          │  │
│  │  (Port 8443)              │  │
│  └───────────────────────────┘  │
│  ┌───────────────────────────┐  │
│  │  Dashboard API Server     │  │
│  │  (Port 8080)              │  │
│  └───────────────────────────┘  │
└─────────────────────────────────┘
              ↕ QUIC
┌─────────────────────────────────┐
│     Client Device               │
│  ┌───────────────────────────┐  │
│  │  Next.js Dashboard        │  │
│  │  (Port 3000)              │  │
│  └───────────────────────────┘  │
│  ┌───────────────────────────┐  │
│  │  File Transfer Client     │  │
│  └───────────────────────────┘  │
└─────────────────────────────────┘
```

## Server Device Setup

### 1. Start QUIC-FEC Server

```bash
cd /Users/mayankhete/trs/PitlinkPQC

# Generate certificates (first time only)
openssl req -x509 -newkey rsa:4096 -keyout server.key -out server.crt -days 365 -nodes

# Start QUIC server
cargo run --example server --package quic_fec -- 0.0.0.0:8443 ./server_storage
```

### 2. Start Dashboard API Server

```bash
# Start Rust dashboard API (provides metrics API)
cargo run --bin dashboard --package dashboard
```

The dashboard API will run on `http://0.0.0.0:8080`

### 3. Configure Firewall

```bash
# Allow QUIC port
sudo ufw allow 8443/udp

# Allow Dashboard API port
sudo ufw allow 8080/tcp
```

## Client Device Setup

### 1. Install Dependencies

```bash
cd /Users/mayankhete/trs/PitlinkPQC/dashboard-web
npm install
```

### 2. Configure Backend URL

Create `.env.local`:

```bash
# Replace with your server's IP address
NEXT_PUBLIC_BACKEND_URL=http://SERVER_IP:8080
BACKEND_URL=http://SERVER_IP:8080
```

### 3. Start Next.js Dashboard

```bash
npm run dev
```

Dashboard will run on `http://localhost:3000`

## Real Data Flow

### Metrics Collection

1. **Server Side**: Rust dashboard collects real metrics from:
   - QUIC connection stats
   - Network interface metrics (if available)
   - Transfer progress
   - FEC recovery stats

2. **Client Side**: Next.js dashboard fetches from:
   - `GET http://SERVER_IP:8080/api/metrics/current`
   - Updates every 1-2 seconds

### File Transfer

1. **Upload**: User uploads file in Next.js dashboard
2. **Client**: File saved locally, transfer request sent to backend
3. **Backend**: Starts QUIC transfer to server
4. **Server**: Receives file via QUIC-FEC
5. **Progress**: Real-time updates via API

## Network Health

### Real Network Detection

The system will:
- **If network interfaces available**: Show real RTT, throughput, loss
- **If not available**: Show "Network data unavailable"
- **No fake data**: All metrics are real or marked unavailable

### Network Interface Detection

The Rust backend can detect:
- WiFi interfaces
- Ethernet interfaces
- 5G modems (if available)
- Starlink (if configured)

If interfaces are not available, the dashboard shows:
- "Network data unavailable"
- "Backend connected" but no path metrics

## Testing Real Transfer

### From Client Device

1. Open dashboard: `http://localhost:3000`
2. Check backend status (should show "Backend Connected")
3. Upload a file
4. Monitor real-time progress
5. File transfers to server device

### Verify Transfer

On server device:
```bash
ls -lh server_storage/
# Should show received files
```

## Troubleshooting

### Backend Not Connected

**Symptoms**: Dashboard shows "Backend Disconnected"

**Solutions**:
1. Check server is running: `curl http://SERVER_IP:8080/api/health`
2. Check firewall: `sudo ufw status`
3. Check network connectivity: `ping SERVER_IP`
4. Verify `.env.local` has correct IP

### No Network Metrics

**Symptoms**: Network paths show as unavailable

**Solutions**:
1. This is normal if network interfaces aren't accessible
2. System will show "Network data unavailable"
3. File transfers still work without network metrics

### Transfer Fails

**Symptoms**: Transfer starts but fails

**Solutions**:
1. Check QUIC server is running on port 8443
2. Check certificates are valid
3. Check firewall allows UDP 8443
4. Check server storage directory exists and is writable

## Production Deployment

### Server Device

```bash
# Use systemd service
sudo systemctl enable pitlink-server
sudo systemctl start pitlink-server
```

### Client Device

```bash
# Build production dashboard
cd dashboard-web
npm run build
npm start
```

## Environment Variables

### Server

```bash
export QUIC_SERVER_PORT=8443
export DASHBOARD_API_PORT=8080
export STORAGE_PATH=/var/lib/pitlink/storage
```

### Client

```bash
# .env.local
NEXT_PUBLIC_BACKEND_URL=http://SERVER_IP:8080
BACKEND_URL=http://SERVER_IP:8080
```

## Real-World Status Display

The dashboard will show:
- ✅ **Available**: Real data from backend
- ❌ **Unavailable**: Backend not connected or data not available
- ⚠️ **Limited**: Some features available, others not

**No fake data** - everything is real or clearly marked as unavailable.

