# PitlinkPQC Dashboard

Modern, high-contrast dashboard UI for the Smart File Transfer System.

## Features

- **File Transfer Overview**: Real-time progress, speed, compression, PQC handshake status
- **Multipath Link Health Monitor**: Live metrics for each network path
- **FEC Recovery Panel**: Recovery statistics and heatmap visualization
- **Integrity Verification**: BLAKE3 hash and Merkle tree verification
- **Priority Channels**: Queue management with priority assignment
- **Real-Time Logs**: Terminal-style event log viewer
- **Analytics**: Charts for throughput, recovery rates, and compression

## Tech Stack

- **Next.js 14** - React framework
- **Tailwind CSS** - Styling
- **shadcn/ui** - UI components (Radix UI)
- **Recharts** - Chart library
- **Framer Motion** - Animations
- **TypeScript** - Type safety

## Getting Started

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build

# Start production server
npm start
```

## Design

- **Futuristic telemetry aesthetic**
- **Glassmorphism effects**
- **Neon highlights** (cyan, green, blue, purple, pink)
- **Responsive layout**
- **Animated transitions**

## Components

All components are in `/components`:
- `FileTransferOverview.tsx`
- `MultipathHealthMonitor.tsx`
- `FECRecoveryPanel.tsx`
- `IntegrityVerification.tsx`
- `PriorityChannels.tsx`
- `RealTimeLogs.tsx`
- `Analytics.tsx`

## API Integration

The dashboard is designed to connect to the backend API. Update the API endpoints in each component to connect to your server.

## License

MIT

