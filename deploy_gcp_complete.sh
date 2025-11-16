#!/bin/bash
# Complete GCP Deployment Script
# Creates two instances, deploys server and client, and sets up everything

set -e

# Configuration
PROJECT_ID=${1:-pitlinkpqc}
ZONE=${2:-us-central1-a}
REPO_URL=${3:-"https://github.com/Rancidcake/PitlinkPQC1.git"}
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
  # Clone brain if it doesn't exist (it's a separate repo)
  if [ ! -d brain/.git ]; then
    git clone https://github.com/mardromus/brain.git brain 2>/dev/null || echo 'Warning: Could not clone brain repository'
  fi
else
  cd PitlinkPQC
  git pull
  git submodule update --init --recursive 2>/dev/null || true
  # Clone brain if it doesn't exist
  if [ ! -d brain/.git ]; then
    git clone https://github.com/mardromus/brain.git brain 2>/dev/null || echo 'Warning: Could not clone brain repository'
  fi
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
  # Clone brain if it doesn't exist (it's a separate repo)
  if [ ! -d brain/.git ]; then
    git clone https://github.com/mardromus/brain.git brain 2>/dev/null || echo 'Warning: Could not clone brain repository'
  fi
else
  cd PitlinkPQC
  git pull
  git submodule update --init --recursive 2>/dev/null || true
  # Clone brain if it doesn't exist
  if [ ! -d brain/.git ]; then
    git clone https://github.com/mardromus/brain.git brain 2>/dev/null || echo 'Warning: Could not clone brain repository'
  fi
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

