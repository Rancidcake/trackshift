#!/bin/bash
# Deploy Client Instance on Google Cloud
# This script sets up and runs the Next.js dashboard

set -e

echo "🚀 Deploying PitlinkPQC Client Instance..."
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="/home/$USER/PitlinkPQC"
DASHBOARD_DIR="$PROJECT_ROOT/dashboard-web"
LOG_DIR="$PROJECT_ROOT/logs"
UPLOADS_DIR="$DASHBOARD_DIR/uploads"

# Get server IP from user or environment
if [ -z "$SERVER_IP" ]; then
    echo -e "${YELLOW}⚠️  Server IP not set${NC}"
    read -p "Enter server IP address: " SERVER_IP
    if [ -z "$SERVER_IP" ]; then
        echo -e "${RED}❌ Server IP is required${NC}"
        exit 1
    fi
fi

# Create directories
echo "📁 Creating directories..."
mkdir -p "$UPLOADS_DIR"
mkdir -p "$LOG_DIR"

# Navigate to project root
cd "$PROJECT_ROOT" || {
    echo -e "${RED}❌ Project directory not found: $PROJECT_ROOT${NC}"
    echo "Please clone the repository first:"
    echo "  git clone <YOUR_REPO_URL> $PROJECT_ROOT"
    exit 1
}

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${YELLOW}⚠️  Node.js not found. Installing Node.js...${NC}"
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
    sudo apt-get install -y nodejs
fi

# Install system dependencies
echo "📦 Installing system dependencies..."
sudo apt-get update -qq
sudo apt-get install -y -qq curl git

# Navigate to dashboard directory
cd "$DASHBOARD_DIR" || {
    echo -e "${RED}❌ Dashboard directory not found: $DASHBOARD_DIR${NC}"
    exit 1
}

# Install npm dependencies
echo "📦 Installing npm dependencies..."
npm install --silent

# Configure environment
echo "⚙️  Configuring environment..."
cat > .env.local <<EOF
NEXT_PUBLIC_BACKEND_URL=http://$SERVER_IP:8080
BACKEND_URL=http://$SERVER_IP:8080
EOF

echo -e "${GREEN}✅ Environment configured${NC}"
echo "   Backend URL: http://$SERVER_IP:8080"

# Build the dashboard
echo "🔨 Building dashboard..."
npm run build

# Get client IP
CLIENT_IP=$(hostname -I | awk '{print $1}')
DASHBOARD_PORT=3000

# Create systemd service for Next.js dashboard
echo "📝 Creating dashboard service..."
sudo tee /etc/systemd/system/pitlink-client.service > /dev/null <<EOF
[Unit]
Description=PitlinkPQC Client Dashboard
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$DASHBOARD_DIR
Environment="NODE_ENV=production"
Environment="PORT=$DASHBOARD_PORT"
ExecStart=/usr/bin/npm start
Restart=always
RestartSec=5
StandardOutput=append:$LOG_DIR/client.log
StandardError=append:$LOG_DIR/client_error.log

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd and start service
echo "🔄 Starting dashboard service..."
sudo systemctl daemon-reload
sudo systemctl enable pitlink-client
sudo systemctl restart pitlink-client

# Wait for service to start
sleep 5

# Check service status
echo ""
echo "📊 Service Status:"
if sudo systemctl is-active --quiet pitlink-client; then
    echo -e "   Client Dashboard: ${GREEN}✓ Running${NC}"
else
    echo -e "   Client Dashboard: ${RED}✗ Failed${NC}"
    echo "   Check logs: sudo journalctl -u pitlink-client -n 50"
fi

echo ""
echo "✅ Client deployment complete!"
echo ""
echo "📋 Access Information:"
echo "   Dashboard URL: http://$CLIENT_IP:$DASHBOARD_PORT"
echo "   Backend Server: http://$SERVER_IP:8080"
echo ""
echo "📝 Useful Commands:"
echo "   View logs: sudo journalctl -u pitlink-client -f"
echo "   Restart service: sudo systemctl restart pitlink-client"
echo "   Stop service: sudo systemctl stop pitlink-client"
echo ""
echo "🌐 Open in browser: http://$CLIENT_IP:$DASHBOARD_PORT"

