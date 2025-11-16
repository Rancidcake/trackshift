#!/bin/bash
# Deploy Server Instance on Google Cloud
# This script sets up and runs the QUIC-FEC server and Dashboard API

set -e

echo "🚀 Deploying PitlinkPQC Server Instance..."
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="/home/$USER/PitlinkPQC"
STORAGE_DIR="$PROJECT_ROOT/server_storage"
LOG_DIR="$PROJECT_ROOT/logs"
CERTS_DIR="$PROJECT_ROOT/certs"

# Create directories
echo "📁 Creating directories..."
mkdir -p "$STORAGE_DIR"
mkdir -p "$LOG_DIR"
mkdir -p "$CERTS_DIR"

# Navigate to project root
cd "$PROJECT_ROOT" || {
    echo -e "${RED}❌ Project directory not found: $PROJECT_ROOT${NC}"
    echo "Please clone the repository first:"
    echo "  git clone <YOUR_REPO_URL> $PROJECT_ROOT"
    exit 1
}

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}⚠️  Rust not found. Installing Rust...${NC}"
    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Install system dependencies
echo "📦 Installing system dependencies..."
sudo apt-get update -qq
sudo apt-get install -y -qq \
    build-essential \
    pkg-config \
    libssl-dev \
    openssl \
    curl \
    git

# Generate certificates if they don't exist
if [ ! -f "$CERTS_DIR/server.crt" ] || [ ! -f "$CERTS_DIR/server.key" ]; then
    echo "🔐 Generating SSL certificates..."
    openssl req -x509 -newkey rsa:4096 \
        -keyout "$CERTS_DIR/server.key" \
        -out "$CERTS_DIR/server.crt" \
        -days 365 \
        -nodes \
        -subj "/CN=pitlink-server/O=PitlinkPQC"
    echo -e "${GREEN}✅ Certificates generated${NC}"
fi

# Build the project
echo "🔨 Building project..."
source "$HOME/.cargo/env"
cargo build --release

# Get server IP
SERVER_IP=$(hostname -I | awk '{print $1}')
QUIC_PORT=8443
API_PORT=8080

echo ""
echo "📡 Server Configuration:"
echo "   IP Address: $SERVER_IP"
echo "   QUIC Port: $QUIC_PORT (UDP)"
echo "   API Port: $API_PORT (TCP)"
echo ""

# Create systemd service for QUIC server
echo "📝 Creating QUIC server service..."
sudo tee /etc/systemd/system/pitlink-quic.service > /dev/null <<EOF
[Unit]
Description=PitlinkPQC QUIC-FEC Server
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$PROJECT_ROOT
Environment="RUST_LOG=info"
ExecStart=$HOME/.cargo/bin/cargo run --release --example server --package quic_fec -- 0.0.0.0:$QUIC_PORT $STORAGE_DIR
Restart=always
RestartSec=5
StandardOutput=append:$LOG_DIR/quic.log
StandardError=append:$LOG_DIR/quic_error.log

[Install]
WantedBy=multi-user.target
EOF

# Create systemd service for Dashboard API
echo "📝 Creating Dashboard API service..."
sudo tee /etc/systemd/system/pitlink-dashboard.service > /dev/null <<EOF
[Unit]
Description=PitlinkPQC Dashboard API Server
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$PROJECT_ROOT/dashboard
Environment="RUST_LOG=info"
ExecStart=$HOME/.cargo/bin/cargo run --release --bin dashboard --package dashboard
Restart=always
RestartSec=5
StandardOutput=append:$LOG_DIR/dashboard.log
StandardError=append:$LOG_DIR/dashboard_error.log

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd and start services
echo "🔄 Starting services..."
sudo systemctl daemon-reload
sudo systemctl enable pitlink-quic pitlink-dashboard
sudo systemctl restart pitlink-quic pitlink-dashboard

# Wait for services to start
sleep 3

# Check service status
echo ""
echo "📊 Service Status:"
if sudo systemctl is-active --quiet pitlink-quic; then
    echo -e "   QUIC Server: ${GREEN}✓ Running${NC}"
else
    echo -e "   QUIC Server: ${RED}✗ Failed${NC}"
    echo "   Check logs: sudo journalctl -u pitlink-quic -n 50"
fi

if sudo systemctl is-active --quiet pitlink-dashboard; then
    echo -e "   Dashboard API: ${GREEN}✓ Running${NC}"
else
    echo -e "   Dashboard API: ${RED}✗ Failed${NC}"
    echo "   Check logs: sudo journalctl -u pitlink-dashboard -n 50"
fi

echo ""
echo "✅ Server deployment complete!"
echo ""
echo "📋 Access Information:"
echo "   QUIC Server: $SERVER_IP:$QUIC_PORT (UDP)"
echo "   Dashboard API: http://$SERVER_IP:$API_PORT"
echo ""
echo "📝 Useful Commands:"
echo "   View QUIC logs: sudo journalctl -u pitlink-quic -f"
echo "   View Dashboard logs: sudo journalctl -u pitlink-dashboard -f"
echo "   Restart services: sudo systemctl restart pitlink-quic pitlink-dashboard"
echo "   Stop services: sudo systemctl stop pitlink-quic pitlink-dashboard"
echo ""
echo "🔗 Share this IP with client instance: $SERVER_IP"

