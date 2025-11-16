# 🎨 Dashboard User Guide

## Overview

The PitlinkPQC Dashboard is a modern, real-time monitoring interface for the Smart File Transfer System. It provides comprehensive visibility into file transfers, network health, FEC recovery, and system analytics.

## Features

### 1. File Transfer Overview

**Location**: Top-left section

**Displays**:
- **File Metadata**: Name, size, transferred bytes
- **Progress Bar**: Visual progress indicator with percentage
- **Speed**: Current transfer speed in MB/s
- **Compression**: Compression ratio and savings
- **PQC Handshake**: Post-Quantum Cryptography handshake status (Kyber-768 + ECDHE)
- **Estimated Time**: Time remaining for transfer completion

**Status Indicators**:
- 🟢 **Active**: Transfer in progress
- 🔵 **Completed**: Transfer finished successfully
- 🟡 **Paused**: Transfer paused
- 🔴 **Error**: Transfer failed

### 2. Multipath Link Health Monitor

**Location**: Left column, below File Transfer Overview

**Displays**:
- **Network Paths**: WiFi, 5G, Starlink, Ethernet
- **Real-time Metrics**:
  - **RTT** (Round-Trip Time): Latency in milliseconds
  - **Loss**: Packet loss percentage
  - **Jitter**: Network jitter in milliseconds
  - **Throughput**: Bandwidth in Mbps

**Health Indicators**:
- 🟢 **Green**: Excellent (RTT < 50ms, Loss < 1%)
- 🟡 **Yellow**: Warning (RTT 50-200ms, Loss 1-5%)
- 🔴 **Red**: Critical (RTT > 200ms, Loss > 5%)

**Path Status**:
- **Active**: Currently in use
- **Backup**: Available but not primary
- **Down**: Unavailable

### 3. FEC Recovery Panel

**Location**: Left column, below Multipath Health Monitor

**Displays**:
- **FEC Algorithm**: Reed-Solomon or XOR
- **Block Configuration**: k (data shards) + r (parity shards)
- **Recovery Statistics**:
  - Recovered blocks count
  - Failed blocks count
  - Total blocks processed
- **Recovery Rate**: Percentage of successful recoveries
- **Heatmap**: Visual representation of repaired chunks (20x10 grid)

**Color Coding**:
- 🟢 **Green**: Successfully repaired chunk
- 🔴 **Red**: Failed chunk

### 4. Integrity Verification

**Location**: Right column, top

**Displays**:
- **Verification Status**: Verified, Failed, or Pending
- **Chunks Verified**: Progress of integrity checks
- **BLAKE3 Hash**: Full file hash for verification
- **Merkle Tree Root**: Root hash of the Merkle tree

**Status**:
- ✅ **Verified**: All integrity checks passed
- ❌ **Failed**: Hash mismatch detected
- ⏳ **Pending**: Verification in progress

### 5. Priority Channels

**Location**: Right column, below Integrity Verification

**Features**:
- **Priority Levels**:
  - 🔴 **Urgent**: Critical priority (red)
  - 🟡 **High**: High priority (yellow)
  - 🔵 **Normal**: Standard priority (cyan)
  - 🔵 **Bulk**: Low priority (blue)
- **Queue Count**: Number of items in each priority queue
- **Priority Assignment**: Buttons to assign priority to transfers

**Usage**:
1. Click on a priority queue to view details
2. Use assignment buttons to change transfer priority
3. Monitor queue sizes in real-time

### 6. Real-Time Logs

**Location**: Bottom section, full width

**Features**:
- **Terminal-style Interface**: Scrollable log viewer
- **Log Levels**:
  - ℹ️ **INFO**: General information (cyan)
  - ✅ **OK**: Success messages (green)
  - ⚠️ **WARN**: Warnings (yellow)
  - ❌ **ERR**: Errors (red)
- **Auto-scroll**: Automatically scrolls to latest entries
- **Expandable**: Click chevron to expand/collapse

**Events Logged**:
- QUIC connection events
- Handshake completions
- FEC operations
- Path handovers
- Compression activities
- Packet events

### 7. Analytics

**Location**: Right column, bottom

**Charts**:
1. **Throughput Chart**: Line chart showing Mbps over time
2. **FEC Recovery Rate**: Recovery percentage over time
3. **Failover Events**: Bar chart of path handover events
4. **Compression Ratio**: Compression efficiency over time

**Time Range**: Last 12 minutes (updates every 5 seconds)

## Color Scheme

### Neon Colors
- **Cyan** (#00ffff): Primary actions, active states
- **Green** (#00ff00): Success, healthy status
- **Blue** (#0080ff): Information, normal priority
- **Purple** (#8000ff): FEC operations, special features
- **Pink** (#ff00ff): Time estimates, highlights

### Status Colors
- **Red**: Errors, failures, critical issues
- **Yellow**: Warnings, high priority
- **Green**: Success, healthy, active
- **Cyan**: Information, normal operations

## Responsive Design

The dashboard is fully responsive:
- **Mobile**: Single column layout
- **Tablet**: 2-column layout
- **Desktop**: 3-column layout with full features

## Keyboard Shortcuts

- **Escape**: Close modals/dialogs
- **Arrow Keys**: Navigate charts
- **Space**: Pause/play animations

## API Integration

To connect to your backend:

1. Update API endpoints in each component
2. Replace mock data with API calls
3. Configure WebSocket for real-time updates

Example:
```typescript
// In components/FileTransferOverview.tsx
useEffect(() => {
  const fetchData = async () => {
    const response = await fetch('/api/transfer/status')
    const data = await response.json()
    setTransfer(data)
  }
  fetchData()
  const interval = setInterval(fetchData, 1000)
  return () => clearInterval(interval)
}, [])
```

## Troubleshooting

### Dashboard Not Loading
- Check Node.js version (18+ required)
- Run `npm install` to install dependencies
- Check browser console for errors

### Data Not Updating
- Verify API endpoints are correct
- Check network connectivity
- Ensure WebSocket connection is established

### Charts Not Rendering
- Verify Recharts is installed
- Check browser compatibility
- Clear browser cache

## Performance Tips

- **Reduce Update Frequency**: Increase interval times for less frequent updates
- **Limit Log History**: Keep only last 50-100 log entries
- **Optimize Charts**: Reduce data points for better performance
- **Use WebSockets**: For real-time updates instead of polling

## Customization

### Change Colors
Edit `tailwind.config.js`:
```javascript
colors: {
  neon: {
    cyan: '#your-color',
    // ...
  }
}
```

### Modify Layout
Edit `app/page.tsx` to rearrange components

### Add Components
Create new components in `/components` and import in `page.tsx`

## Support

For issues or questions:
- Check the README.md
- Review component source code
- Check API documentation

