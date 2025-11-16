#!/bin/bash
# Complete GCP Deployment Script
# Creates two instances, deploys server and client, and sets up everything

set -e

# Configuration
PROJECT_ID=${1:-pitlinkpqc}
ZONE=${2:-us-central1-a}
REPO_URL=${3:-"https://github.com/Rancidcake/trackshift.git"}
INSTANCE_TYPE=${4:-e2-medium}

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🚀 Complete GCP Deployment for PitlinkPQC${NC}"
echo "=========================================="
echo ""
echo "Project: $PROJECT_ID"
echo "Zone: $ZONE"
echo "Repo: $REPO_URL"
echo "Instance Type: $INSTANCE_TYPE"
echo ""

# Set project
gcloud config set project $PROJECT_ID

# Check if instances exist
echo -e "${YELLOW}🔍 Checking for existing instances...${NC}"
SERVER_EXISTS=$(gcloud compute instances describe pitlink-server --zone=$ZONE --format="value(name)" 2>/dev/null || echo "")
CLIENT_EXISTS=$(gcloud compute instances describe pitlink-client --zone=$ZONE --format="value(name)" 2>/dev/null || echo "")

if [ -n "$SERVER_EXISTS" ] || [ -n "$CLIENT_EXISTS" ]; then
    echo -e "${YELLOW}⚠️  Instances already exist. Skipping creation...${NC}"
    if [ -n "$SERVER_EXISTS" ]; then
        echo "   ✓ pitlink-server exists"
    fi
    if [ -n "$CLIENT_EXISTS" ]; then
        echo "   ✓ pitlink-client exists"
    fi
else
    # Create instances
    echo -e "${YELLOW}📦 Creating VM instances...${NC}"
    gcloud compute instances create pitlink-server pitlink-client \
      --zone=$ZONE \
      --machine-type=$INSTANCE_TYPE \
      --image-family=ubuntu-2204-lts \
      --image-project=ubuntu-os-cloud \
      --boot-disk-size=20GB \
      --tags=pitlink-server,pitlink-client \
      --metadata=startup-script='#!/bin/bash
apt-get update
apt-get install -y curl build-essential git nodejs npm pkg-config libssl-dev openssl
curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
apt-get install -y nodejs
curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
rustup default stable'
fi

# Wait for instances
echo -e "${YELLOW}⏳ Waiting for instances to be ready...${NC}"
sleep 30

# Configure firewall
echo -e "${YELLOW}🔥 Configuring firewall...${NC}"
gcloud compute firewall-rules create allow-quic --allow udp:8443 --source-ranges 0.0.0.0/0 --description "QUIC" 2>/dev/null || true
gcloud compute firewall-rules create allow-dashboard-api --allow tcp:8080 --source-ranges 0.0.0.0/0 --description "Dashboard API" 2>/dev/null || true
gcloud compute firewall-rules create allow-nextjs --allow tcp:3000 --source-ranges 0.0.0.0/0 --description "Next.js" 2>/dev/null || true
gcloud compute firewall-rules create allow-ssh --allow tcp:22 --source-ranges 0.0.0.0/0 --description "SSH" 2>/dev/null || true

# Get IPs
SERVER_IP=$(gcloud compute instances describe pitlink-server --zone=$ZONE --format='get(networkInterfaces[0].accessConfigs[0].natIP)')
CLIENT_IP=$(gcloud compute instances describe pitlink-client --zone=$ZONE --format='get(networkInterfaces[0].accessConfigs[0].natIP)')

echo -e "${GREEN}✅ Instances created${NC}"
echo "   Server IP: $SERVER_IP"
echo "   Client IP: $CLIENT_IP"
echo ""

# Deploy server
echo -e "${YELLOW}🚀 Deploying server...${NC}"
gcloud compute ssh pitlink-server --zone=$ZONE --command="
cd ~ &&
if [ ! -d PitlinkPQC ]; then
  git clone --recurse-submodules $REPO_URL PitlinkPQC || git clone $REPO_URL PitlinkPQC
  cd PitlinkPQC
  git submodule update --init --recursive 2>/dev/null || true
  # brain is already in trackshift repo, no need to clone separately
else
  cd PitlinkPQC
  git pull
  git submodule update --init --recursive 2>/dev/null || true
  # Clone brain if it doesn't exist
  if [ ! -d brain/.git ]; then
    git clone https://github.com/Rancidcake/trackshift.git brain 2>/dev/null || echo 'Warning: Could not clone trackshift repository'
  fi
fi &&
# Fix compilation errors in trackshift
if [ -d brain ] && [ -f brain/src/telemetry_ai/mod.rs ]; then
  echo '🔧 Fixing trackshift compilation errors...'
  # Use Python fix script if available (most reliable), otherwise bash script, then inline
  if command -v python3 &> /dev/null && [ -f fix_trackshift_python.py ]; then
    python3 fix_trackshift_python.py brain
  elif [ -f fix_trackshift_comprehensive.sh ]; then
    chmod +x fix_trackshift_comprehensive.sh
    ./fix_trackshift_comprehensive.sh brain
  else
    cd brain/src/telemetry_ai
    # Fix ORT imports
    sed -i 's/use ort::{Session, SessionBuilder, Value};/use ort::session::Session;\nuse ort::session::builder::SessionBuilder;\nuse ort::value::Value;/' mod.rs
    # Add imports at top
    if ! grep -q '^use std::sync::Arc;' mod.rs; then sed -i '1i use std::sync::Arc;' mod.rs; fi
    if ! grep -q 'use anyhow::Context' mod.rs; then sed -i '/^use anyhow::/a use anyhow::Context;' mod.rs || sed -i '1i use anyhow::Context;' mod.rs; fi
    if ! grep -q 'use anyhow::Result' mod.rs; then sed -i '/^use anyhow::/a use anyhow::Result;' mod.rs || sed -i '1i use anyhow::Result;' mod.rs; fi
    # Replace Result types
    sed -i 's/-> Result</-> anyhow::Result</g' mod.rs
    sed -i 's/) -> Result</) -> anyhow::Result</g' mod.rs
    # Fix SessionBuilder - commit_from_file returns Result, use ? directly instead of .context()?
    sed -i '/\.commit_from_file(.*)/{N;s/\n[[:space:]]*\.context(.*)?;//;}' mod.rs || true
    # Fix network_quality
    if [ -f network_quality.rs ]; then
      sed -i 's/let mut score = 1\.0;/let mut score: f32 = 1.0;/' network_quality.rs
      sed -i 's/score = score\.max(0\.0)\.min(1\.0);/score = score.max(0.0_f32).min(1.0_f32);/' network_quality.rs
    fi
    # Fix file_size
    sed -i 's/file_size > 100_000_000\.0/file_size > 100_000_000/g' mod.rs
    sed -i 's/file_size > 10_000_000\.0/file_size > 10_000_000/g' mod.rs
    cd ../../..
  fi
  echo '✅ Fixes applied'
fi &&
chmod +x deploy_server.sh &&
./deploy_server.sh
" --quiet

# Wait a bit for server to start
sleep 10

# Deploy client
echo -e "${YELLOW}💻 Deploying client...${NC}"
gcloud compute ssh pitlink-client --zone=$ZONE --command="
cd ~ &&
if [ ! -d PitlinkPQC ]; then
  git clone --recurse-submodules $REPO_URL PitlinkPQC || git clone $REPO_URL PitlinkPQC
  cd PitlinkPQC
  git submodule update --init --recursive 2>/dev/null || true
  # brain is already in trackshift repo, no need to clone separately
else
  cd PitlinkPQC
  git pull
  git submodule update --init --recursive 2>/dev/null || true
  # Clone brain if it doesn't exist
  if [ ! -d brain/.git ]; then
    git clone https://github.com/Rancidcake/trackshift.git brain 2>/dev/null || echo 'Warning: Could not clone trackshift repository'
  fi
fi &&
# Fix compilation errors in trackshift (if needed for client build)
if [ -d brain ] && [ -f brain/src/telemetry_ai/mod.rs ]; then
  echo '🔧 Fixing trackshift compilation errors...'
  if command -v python3 &> /dev/null && [ -f fix_trackshift_python.py ]; then
    python3 fix_trackshift_python.py brain
  elif [ -f fix_trackshift_comprehensive.sh ]; then
    chmod +x fix_trackshift_comprehensive.sh
    ./fix_trackshift_comprehensive.sh brain
  else
    cd brain/src/telemetry_ai
    sed -i 's/use ort::{Session, SessionBuilder, Value};/use ort::session::Session;\nuse ort::session::builder::SessionBuilder;\nuse ort::value::Value;/' mod.rs
    if ! grep -q '^use std::sync::Arc;' mod.rs; then sed -i '1i use std::sync::Arc;' mod.rs; fi
    if ! grep -q 'use anyhow::Context' mod.rs; then sed -i '/^use anyhow::/a use anyhow::Context;' mod.rs || sed -i '1i use anyhow::Context;' mod.rs; fi
    if ! grep -q 'use anyhow::Result' mod.rs; then sed -i '/^use anyhow::/a use anyhow::Result;' mod.rs || sed -i '1i use anyhow::Result;' mod.rs; fi
    sed -i 's/-> Result</-> anyhow::Result</g' mod.rs
    sed -i 's/) -> Result</) -> anyhow::Result</g' mod.rs
    sed -i '/\.commit_from_file(.*)/{N;s/\n[[:space:]]*\.context(.*)?;//;}' mod.rs || true
    if [ -f network_quality.rs ]; then
      sed -i 's/let mut score = 1\.0;/let mut score: f32 = 1.0;/' network_quality.rs
      sed -i 's/score = score\.max(0\.0)\.min(1\.0);/score = score.max(0.0_f32).min(1.0_f32);/' network_quality.rs
    fi
    sed -i 's/file_size > 100_000_000\.0/file_size > 100_000_000/g' mod.rs
    sed -i 's/file_size > 10_000_000\.0/file_size > 10_000_000/g' mod.rs
    cd ../../..
  fi
  echo '✅ Fixes applied'
fi &&
chmod +x deploy_client.sh &&
SERVER_IP=$SERVER_IP ./deploy_client.sh
" --quiet

echo ""
echo -e "${GREEN}✅ Deployment Complete!${NC}"
echo ""
echo "📡 Server:"
echo "   QUIC: $SERVER_IP:8443 (UDP)"
echo "   API:  http://$SERVER_IP:8080"
echo ""
echo "💻 Client:"
echo "   Dashboard: http://$CLIENT_IP:3000"
echo ""
echo "🔗 Access Dashboard: http://$CLIENT_IP:3000"
echo ""
echo "📝 Test Connection:"
echo "   curl http://$SERVER_IP:8080/api/health"

