# Quick Start Guide

## 🚀 Running the Complete System

### Prerequisites

- Rust (latest stable)
- OpenSSL (for certificate generation)
- Terminal with multiple tabs/windows

### Step 1: Build Everything

```bash
cargo build --workspace
```

### Step 2: Generate Certificate

```bash
cd quic_fec/examples
chmod +x generate_cert.sh
./generate_cert.sh
cd ../..
```

### Step 3: Run System Components

#### Terminal 1: Dashboard

```bash
cargo run --package dashboard
```

Dashboard will be available at: http://localhost:8080

#### Terminal 2: File Transfer Server

```bash
cargo run --example server -- 127.0.0.1:8080
```

Server will listen on: `127.0.0.1:8080`

#### Terminal 3: File Transfer Client

```bash
# Create a test file
echo "Hello from PitlinkPQC!" > test.txt

# Transfer the file
cargo run --example client -- 127.0.0.1:8080 ./test.txt /uploads/test.txt
```

### Step 4: Monitor in Dashboard

1. Open http://localhost:8080 in your browser
2. View real-time metrics
3. Control system settings
4. Monitor fallback system
5. View active transfers

## 🎛️ Dashboard Controls

### View System State

```bash
curl http://localhost:8080/api/dashboard/state
```

### Check System Health

```bash
curl http://localhost:8080/api/dashboard/system-health
```

### Set Fallback Strategy

```bash
curl -X POST http://localhost:8080/api/dashboard/fallback/strategy \
  -H "Content-Type: application/json" \
  -d '{"strategy": "automatic"}'
```

### Trigger Manual Fallback

```bash
curl -X POST http://localhost:8080/api/dashboard/fallback/trigger \
  -H "Content-Type: application/json" \
  -d '{"target_state": "quic_basic"}'
```

### Update Configuration

```bash
curl -X POST http://localhost:8080/api/dashboard/config \
  -H "Content-Type: application/json" \
  -d '{
    "max_concurrent_transfers": 20,
    "default_chunk_size": 131072,
    "enable_ai_routing": true
  }'
```

## 📊 System Architecture

```
┌─────────────┐
│  Dashboard  │  Port 8080 (HTTP)
│  (Web UI)   │
└──────┬──────┘
       │
       │ API Calls
       │
┌──────▼──────────────────────────┐
│     File Transfer System        │
│  ┌──────────┐    ┌──────────┐  │
│  │  Server  │◄──►│  Client  │  │
│  │ :8080    │    │          │  │
│  └──────────┘    └──────────┘  │
│       │                │        │
│       └────────┬────────┘        │
│                │                 │
│         ┌──────▼──────┐          │
│         │  Fallback   │          │
│         │   System    │          │
│         └─────────────┘          │
└──────────────────────────────────┘
```

## 🔄 Fallback System

The system automatically falls back through these states:

1. **FullExperimental** → All features enabled
2. **QuicWithFec** → QUIC + FEC, no multipath
3. **QuicBasic** → Basic QUIC only
4. **TcpFallback** → TCP fallback
5. **MinimalFallback** → Minimal features

## 🧪 Testing

### Test File Transfer

```bash
# Server
cargo run --example server -- 127.0.0.1:8080 &

# Client
cargo run --example client -- 127.0.0.1:8080 ./test.txt /uploads/test.txt
```

### Test Fallback System

```bash
# Trigger fallback
curl -X POST http://localhost:8080/api/dashboard/fallback/trigger \
  -H "Content-Type: application/json" \
  -d '{"target_state": "tcp_fallback"}'

# Check state
curl http://localhost:8080/api/dashboard/system-health

# Try recovery
curl -X POST http://localhost:8080/api/dashboard/fallback/recover
```

## 📝 Example Workflow

1. **Start Dashboard**: Monitor system in real-time
2. **Start Server**: Accept file transfers
3. **Transfer Files**: Use client to send files
4. **Monitor**: Watch metrics in dashboard
5. **Control**: Adjust settings via API
6. **Test Fallback**: Trigger fallback and observe recovery

## 🐛 Troubleshooting

### Certificate Issues

```bash
cd quic_fec/examples
./generate_cert.sh
```

### Port Already in Use

Change ports:
- Dashboard: `DASHBOARD_PORT=3000 cargo run --package dashboard`
- Server: `cargo run --example server -- 127.0.0.1:8081`

### Build Errors

```bash
cargo clean
cargo build --workspace
```

## 📚 Documentation

- `FILE_TRANSFER_README.md` - File transfer system
- `DASHBOARD_CONTROLS_README.md` - Dashboard controls
- `FILE_TRANSFER_ROADMAP.md` - Implementation roadmap

## ✅ Verification Checklist

- [ ] All packages build successfully
- [ ] Certificate generated
- [ ] Dashboard starts on port 8080
- [ ] Server starts on port 8080
- [ ] Client can connect to server
- [ ] File transfer works
- [ ] Dashboard shows metrics
- [ ] Fallback system responds
- [ ] API endpoints work

## 🎉 Success!

If all components start and communicate, your system is ready!

