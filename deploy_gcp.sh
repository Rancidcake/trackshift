#!/bin/bash
set -e

PROJECT_ID=${1:-pitlinkpqc}
ZONE=${2:-us-central1-a}
REPO_URL=${3:-"https://github.com/yourusername/PitlinkPQC.git"}

echo "🚀 Deploying PitlinkPQC to GCP..."
echo "Project: $PROJECT_ID"
echo "Zone: $ZONE"
echo "Repo: $REPO_URL"
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

# Wait a bit for server to start
sleep 10

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
echo "   QUIC Server:    $SERVER_IP:8443"
echo "   Dashboard API:  http://$SERVER_IP:8080"
echo ""
echo "💻 Client:"
echo "   Dashboard UI:    http://$CLIENT_IP:3000"
echo ""
echo "🔗 Open Dashboard: http://$CLIENT_IP:3000"
echo ""
echo "📝 Check logs:"
echo "   Server: gcloud compute ssh pitlink-server --zone=$ZONE --command='tail -f quic.log dashboard.log'"
echo "   Client: gcloud compute ssh pitlink-client --zone=$ZONE --command='tail -f nextjs.log'"

