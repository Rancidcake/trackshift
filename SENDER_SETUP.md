# Sender-Side Setup Guide

## 🚀 Quick Start - Sender Only

This setup runs **only the sender side** on this PC. The receiver will run on other devices.

## 📋 Prerequisites

- Node.js and npm installed
- Rust and Cargo installed
- Ports 3000 and 8080 available

## 🔧 Setup Steps

### 1. Configure Receiver Address

Edit `dashboard-web/.env.local` and set your receiver's IP address:

```bash
NEXT_PUBLIC_RECEIVER_ADDRESS=192.168.1.100:8081
NEXT_PUBLIC_RECEIVER_NAME=receiver
```

Replace `192.168.1.100:8081` with your actual receiver device's IP and port.

### 2. Start Sender Services

```bash
# Terminal 1: Start Rust backend (sender dashboard)
cd /Users/mayankhete/trs/PitlinkPQC
cargo run --release -p dashboard

# Terminal 2: Start Next.js frontend (file upload UI)
cd /Users/mayankhete/trs/PitlinkPQC/dashboard-web
npm run dev
```

### 3. Access File Upload Client

Open in browser:
```
http://localhost:3000/client
```

## 📤 How It Works

1. **Upload Files**: Use the web UI at `http://localhost:3000/client`
2. **Select Priority**: Choose Critical/High/Normal/Low/Bulk
3. **Start Transfer**: Files are sent to the configured receiver address
4. **Monitor Progress**: Real-time transfer status in the dashboard

## 🔌 Receiver Requirements

On the receiver device, you need to run:

```bash
# Receiver server (on remote device)
cargo run --example server -- --bind 0.0.0.0:8081
```

## ⚙️ Configuration

### Receiver Address

Set via environment variable or edit `dashboard-web/.env.local`:

```bash
export NEXT_PUBLIC_RECEIVER_ADDRESS="192.168.1.100:8081"
```

### Backend API

The sender dashboard API runs on:
```
http://localhost:8080
```

## 📊 Ports Used

- **3000**: Next.js file upload UI (sender client)
- **8080**: Rust backend API (sender dashboard)
- **8081**: Receiver server (on remote device)

## 🎯 Testing

1. Start receiver on remote device: `cargo run --example server -- --bind 0.0.0.0:8081`
2. Start sender on this PC (follow steps above)
3. Upload a file via `http://localhost:3000/client`
4. Check receiver device for received files

## 🔍 Troubleshooting

### Cannot connect to receiver

- Check receiver IP address in `.env.local`
- Ensure receiver is running on remote device
- Check firewall settings
- Verify network connectivity

### Files not uploading

- Check Rust backend is running on port 8080
- Check Next.js is running on port 3000
- Verify receiver address is correct
- Check browser console for errors

## 📝 Notes

- Sender side only handles file uploads and transfer initiation
- Actual file transfer uses QUIC-FEC protocol
- Receiver must be running before starting transfers
- Multiple files can be queued and sent sequentially

