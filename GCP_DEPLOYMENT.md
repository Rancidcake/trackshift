# Google Cloud Platform Deployment Guide

## Quick Setup (5 minutes)

### Prerequisites
- Google Cloud account
- `gcloud` CLI installed and authenticated
- Project created: `gcloud projects create pitlinkpqc --name="PitlinkPQC"`

---

## Step 1: Create Two VM Instances

```bash
# Set your project
export PROJECT_ID=pitlinkpqc
gcloud config set project $PROJECT_ID

# Enable billing APIs
gcloud services enable compute.googleapis.com

# Create Server Instance (Ubuntu 22.04)
gcloud compute instances create pitlink-server \
  --zone=us-central1-a \
  --machine-type=e2-medium \
  --image-family=ubuntu-2204-lts \
  --image-project=ubuntu-os-cloud \
  --boot-disk-size=20GB \
  --tags=pitlink-server \
  --metadata=startup-script='#!/bin/bash
apt-get update
apt-get install -y curl build-essential git
curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
rustup default stable'

# Create Client Instance (Ubuntu 22.04)
gcloud compute instances create pitlink-client \
  --zone=us-central1-a \
  --machine-type=e2-medium \
  --image-family=ubuntu-2204-lts \
  --image-project=ubuntu-os-cloud \
  --boot-disk-size=20GB \
  --tags=pitlink-client \
  --metadata=startup-script='#!/bin/bash
apt-get update
apt-get install -y curl build-essential git nodejs npm
curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
apt-get install -y nodejs
curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
rustup default stable'
```

**Get IP addresses:**
```bash
# Server IP
SERVER_IP=$(gcloud compute instances describe pitlink-server --zone=us-central1-a --format='get(networkInterfaces[0].accessConfigs[0].natIP)')
echo "Server IP: $SERVER_IP"

# Client IP
CLIENT_IP=$(gcloud compute instances describe pitlink-client --zone=us-central1-a --format='get(networkInterfaces[0].accessConfigs[0].natIP)')
echo "Client IP: $CLIENT_IP"
```

---

## Step 2: Configure Firewall Rules

```bash
# Allow QUIC (UDP 8443)
gcloud compute firewall-rules create allow-quic \
  --allow udp:8443 \
  --source-ranges 0.0.0.0/0 \
  --description "Allow QUIC-FEC traffic"

# Allow Dashboard API (TCP 8080)
gcloud compute firewall-rules create allow-dashboard-api \
  --allow tcp:8080 \
  --source-ranges 0.0.0.0/0 \
  --description "Allow Dashboard API"

# Allow Next.js Dashboard (TCP 3000)
gcloud compute firewall-rules create allow-nextjs \
  --allow tcp:3000 \
  --source-ranges 0.0.0.0/0 \
  --description "Allow Next.js Dashboard"

# Allow SSH
gcloud compute firewall-rules create allow-ssh \
  --allow tcp:22 \
  --source-ranges 0.0.0.0/0 \
  --description "Allow SSH"
```

---

## Step 3: Deploy to Server Instance

```bash
# SSH to server
gcloud compute ssh pitlink-server --zone=us-central1-a

# On server, run:
cd ~
git clone <YOUR_REPO_URL> PitlinkPQC || echo "Clone your repo manually"
cd PitlinkPQC

# Install dependencies
sudo apt-get update
sudo apt-get install -y pkg-config libssl-dev openssl

# Build and start server
chmod +x start_server.sh
./start_server.sh
```

**Or use a startup script:**

```bash
# Create server startup script
cat > server_setup.sh << 'EOF'
#!/bin/bash
cd /home/$USER
git clone <YOUR_REPO_URL> PitlinkPQC
cd PitlinkPQC
sudo apt-get update
sudo apt-get install -y pkg-config libssl-dev openssl
source $HOME/.cargo/env
cargo build --release
cd quic_fec/examples
openssl req -x509 -newkey rsa:4096 -keyout server.key -out server.crt -days 365 -nodes -subj "/CN=localhost"
cd ../../..
mkdir -p server_storage
nohup cargo run --example server --package quic_fec -- 0.0.0.0:8443 ./server_storage > quic.log 2>&1 &
cd dashboard
nohup cargo run --bin dashboard --package dashboard > dashboard.log 2>&1 &
EOF

# Upload and run
gcloud compute scp server_setup.sh pitlink-server:~/ --zone=us-central1-a
gcloud compute ssh pitlink-server --zone=us-central1-a --command="chmod +x server_setup.sh && ./server_setup.sh"
```

---

## Step 4: Deploy to Client Instance

```bash
# SSH to client
gcloud compute ssh pitlink-client --zone=us-central1-a

# On client, run:
cd ~
git clone <YOUR_REPO_URL> PitlinkPQC || echo "Clone your repo manually"
cd PitlinkPQC/dashboard-web

# Get server IP
SERVER_IP=$(gcloud compute instances describe pitlink-server --zone=us-central1-a --format='get(networkInterfaces[0].accessConfigs[0].natIP)')

# Configure environment
echo "NEXT_PUBLIC_BACKEND_URL=http://$SERVER_IP:8080" > .env.local
echo "BACKEND_URL=http://$SERVER_IP:8080" >> .env.local

# Install and start
npm install
npm run build
nohup npm start > nextjs.log 2>&1 &
```

**Or use a startup script:**

```bash
# Create client startup script
cat > client_setup.sh << 'EOF'
#!/bin/bash
cd /home/$USER
git clone <YOUR_REPO_URL> PitlinkPQC
cd PitlinkPQC/dashboard-web
SERVER_IP=$(gcloud compute instances describe pitlink-server --zone=us-central1-a --format='get(networkInterfaces[0].accessConfigs[0].natIP)')
echo "NEXT_PUBLIC_BACKEND_URL=http://$SERVER_IP:8080" > .env.local
echo "BACKEND_URL=http://$SERVER_IP:8080" >> .env.local
npm install
npm run build
nohup npm start > nextjs.log 2>&1 &
EOF

# Upload and run
gcloud compute scp client_setup.sh pitlink-client:~/ --zone=us-central1-a
gcloud compute ssh pitlink-client --zone=us-central1-a --command="chmod +x client_setup.sh && ./client_setup.sh"
```

---

## Step 5: Quick One-Line Deployment Scripts

### Server Deployment (Run from your local machine)

```bash
SERVER_IP=$(gcloud compute instances describe pitlink-server --zone=us-central1-a --format='get(networkInterfaces[0].accessConfigs[0].natIP)')
gcloud compute ssh pitlink-server --zone=us-central1-a --command="
cd ~ && 
git clone <YOUR_REPO_URL> PitlinkPQC 2>/dev/null || (cd PitlinkPQC && git pull) &&
cd PitlinkPQC &&
sudo apt-get update -qq &&
sudo apt-get install -y -qq pkg-config libssl-dev openssl &&
source \$HOME/.cargo/env &&
cargo build --release &&
cd quic_fec/examples &&
[ ! -f server.crt ] && openssl req -x509 -newkey rsa:4096 -keyout server.key -out server.crt -days 365 -nodes -subj '/CN=localhost' || true &&
cd ../../.. &&
mkdir -p server_storage &&
nohup cargo run --example server --package quic_fec -- 0.0.0.0:8443 ./server_storage > quic.log 2>&1 &
cd dashboard &&
nohup cargo run --bin dashboard --package dashboard > dashboard.log 2>&1 &
echo 'Server started on http://$SERVER_IP:8080'
"
```

### Client Deployment (Run from your local machine)

```bash
SERVER_IP=$(gcloud compute instances describe pitlink-server --zone=us-central1-a --format='get(networkInterfaces[0].accessConfigs[0].natIP)')
CLIENT_IP=$(gcloud compute instances describe pitlink-client --zone=us-central1-a --format='get(networkInterfaces[0].accessConfigs[0].natIP)')
gcloud compute ssh pitlink-client --zone=us-central1-a --command="
cd ~ &&
git clone <YOUR_REPO_URL> PitlinkPQC 2>/dev/null || (cd PitlinkPQC && git pull) &&
cd PitlinkPQC/dashboard-web &&
echo 'NEXT_PUBLIC_BACKEND_URL=http://$SERVER_IP:8080' > .env.local &&
echo 'BACKEND_URL=http://$SERVER_IP:8080' >> .env.local &&
npm install --silent &&
npm run build &&
nohup npm start > nextjs.log 2>&1 &
echo 'Client started on http://$CLIENT_IP:3000'
"
```

---

## Step 6: Access Your Deployment

```bash
# Get IPs
SERVER_IP=$(gcloud compute instances describe pitlink-server --zone=us-central1-a --format='get(networkInterfaces[0].accessConfigs[0].natIP)')
CLIENT_IP=$(gcloud compute instances describe pitlink-client --zone=us-central1-a --format='get(networkInterfaces[0].accessConfigs[0].natIP)')

echo "✅ Deployment Complete!"
echo ""
echo "📡 Server:"
echo "   QUIC Server:    $SERVER_IP:8443"
echo "   Dashboard API:  http://$SERVER_IP:8080"
echo ""
echo "💻 Client:"
echo "   Dashboard UI:    http://$CLIENT_IP:3000"
echo ""
echo "🔗 Access Dashboard: http://$CLIENT_IP:3000"
```

---

## Automated Deployment Script

Save this as `deploy_gcp.sh`:

```bash
#!/bin/bash
set -e

PROJECT_ID=${1:-pitlinkpqc}
ZONE=${2:-us-central1-a}
REPO_URL=${3:-"https://github.com/yourusername/PitlinkPQC.git"}

echo "🚀 Deploying PitlinkPQC to GCP..."
echo "Project: $PROJECT_ID"
echo "Zone: $ZONE"
echo ""

# Set project
gcloud config set project $PROJECT_ID

# Create instances
echo "📦 Creating VM instances..."
gcloud compute instances create pitlink-server pitlink-client \
  --zone=$ZONE \
  --machine-type=e2-medium \
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

# Wait for instances
echo "⏳ Waiting for instances to be ready..."
sleep 30

# Configure firewall
echo "🔥 Configuring firewall..."
gcloud compute firewall-rules create allow-quic --allow udp:8443 --source-ranges 0.0.0.0/0 --description "QUIC" 2>/dev/null || true
gcloud compute firewall-rules create allow-dashboard-api --allow tcp:8080 --source-ranges 0.0.0.0/0 --description "Dashboard API" 2>/dev/null || true
gcloud compute firewall-rules create allow-nextjs --allow tcp:3000 --source-ranges 0.0.0.0/0 --description "Next.js" 2>/dev/null || true
gcloud compute firewall-rules create allow-ssh --allow tcp:22 --source-ranges 0.0.0.0/0 --description "SSH" 2>/dev/null || true

# Get IPs
SERVER_IP=$(gcloud compute instances describe pitlink-server --zone=$ZONE --format='get(networkInterfaces[0].accessConfigs[0].natIP)')
CLIENT_IP=$(gcloud compute instances describe pitlink-client --zone=$ZONE --format='get(networkInterfaces[0].accessConfigs[0].natIP)')

echo "📡 Server IP: $SERVER_IP"
echo "💻 Client IP: $CLIENT_IP"
echo ""

# Deploy server
echo "🚀 Deploying server..."
gcloud compute ssh pitlink-server --zone=$ZONE --command="
cd ~ &&
git clone $REPO_URL PitlinkPQC 2>/dev/null || (cd PitlinkPQC && git pull) &&
cd PitlinkPQC &&
source \$HOME/.cargo/env &&
cargo build --release &&
cd quic_fec/examples &&
[ ! -f server.crt ] && openssl req -x509 -newkey rsa:4096 -keyout server.key -out server.crt -days 365 -nodes -subj '/CN=localhost' || true &&
cd ../../.. &&
mkdir -p server_storage &&
nohup cargo run --example server --package quic_fec -- 0.0.0.0:8443 ./server_storage > quic.log 2>&1 &
cd dashboard &&
nohup cargo run --bin dashboard --package dashboard > dashboard.log 2>&1 &
" --quiet

# Deploy client
echo "💻 Deploying client..."
gcloud compute ssh pitlink-client --zone=$ZONE --command="
cd ~ &&
git clone $REPO_URL PitlinkPQC 2>/dev/null || (cd PitlinkPQC && git pull) &&
cd PitlinkPQC/dashboard-web &&
echo 'NEXT_PUBLIC_BACKEND_URL=http://$SERVER_IP:8080' > .env.local &&
echo 'BACKEND_URL=http://$SERVER_IP:8080' >> .env.local &&
npm install --silent &&
npm run build &&
nohup npm start > nextjs.log 2>&1 &
" --quiet

echo ""
echo "✅ Deployment Complete!"
echo ""
echo "📡 Server:"
echo "   Dashboard API:  http://$SERVER_IP:8080"
echo ""
echo "💻 Client:"
echo "   Dashboard UI:    http://$CLIENT_IP:3000"
echo ""
echo "🔗 Open: http://$CLIENT_IP:3000"
```

**Usage:**
```bash
chmod +x deploy_gcp.sh
./deploy_gcp.sh [PROJECT_ID] [ZONE] [REPO_URL]
```

---

## Troubleshooting

### Check Server Logs
```bash
gcloud compute ssh pitlink-server --zone=us-central1-a --command="tail -f quic.log dashboard.log"
```

### Check Client Logs
```bash
gcloud compute ssh pitlink-client --zone=us-central1-a --command="tail -f nextjs.log"
```

### Restart Services
```bash
# Server
gcloud compute ssh pitlink-server --zone=us-central1-a --command="pkill -f 'cargo run' && cd ~/PitlinkPQC && ./start_server.sh"

# Client
gcloud compute ssh pitlink-client --zone=us-central1-a --command="pkill -f 'npm start' && cd ~/PitlinkPQC/dashboard-web && nohup npm start > nextjs.log 2>&1 &"
```

### Check Status
```bash
# Server processes
gcloud compute ssh pitlink-server --zone=us-central1-a --command="ps aux | grep -E 'cargo|dashboard'"

# Client processes
gcloud compute ssh pitlink-client --zone=us-central1-a --command="ps aux | grep -E 'node|npm'"
```

---

## Cost Estimate

- **2x e2-medium instances**: ~$50-60/month
- **Network egress**: ~$0.12/GB
- **Storage**: ~$0.04/GB/month

**Total**: ~$60-80/month for basic usage

---

## Quick Commands Reference

```bash
# Get IPs
gcloud compute instances list

# SSH to server
gcloud compute ssh pitlink-server --zone=us-central1-a

# SSH to client
gcloud compute ssh pitlink-client --zone=us-central1-a

# Delete instances (when done)
gcloud compute instances delete pitlink-server pitlink-client --zone=us-central1-a

# Delete firewall rules
gcloud compute firewall-rules delete allow-quic allow-dashboard-api allow-nextjs allow-ssh
```
