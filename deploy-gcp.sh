#!/bin/bash
# GCP Deployment Script for PitlinkPQC

set -e

PROJECT_ID="${GCP_PROJECT_ID:-your-project-id}"
REGION="${GCP_REGION:-us-central1}"
ZONE="${GCP_ZONE:-us-central1-a}"
INSTANCE_NAME="pitlinkpqc-server"
MACHINE_TYPE="e2-standard-4"

echo "🚀 Deploying PitlinkPQC to Google Cloud Platform..."
echo "   Project: $PROJECT_ID"
echo "   Region: $REGION"
echo "   Zone: $ZONE"
echo ""

# Check if gcloud is installed
if ! command -v gcloud &> /dev/null; then
    echo "❌ gcloud CLI not found. Install from: https://cloud.google.com/sdk/docs/install"
    exit 1
fi

# Set project
echo "📋 Setting GCP project..."
gcloud config set project $PROJECT_ID

# Enable required APIs
echo "🔌 Enabling GCP APIs..."
gcloud services enable compute.googleapis.com
gcloud services enable cloudbuild.googleapis.com
gcloud services enable storage-api.googleapis.com

# Create Cloud Storage bucket
echo "📦 Creating Cloud Storage bucket..."
BUCKET_NAME="${PROJECT_ID}-pitlinkpqc"
gsutil mb -p $PROJECT_ID -l $REGION gs://$BUCKET_NAME 2>/dev/null || echo "Bucket already exists"

# Build application
echo "🔨 Building application..."
cd "$(dirname "$0")"
cargo build --release -p dashboard

# Generate models if needed
if [ ! -f "brain/models/slm.onnx" ]; then
    echo "🤖 Generating ONNX models..."
    python3 brain/scripts/create_onnx_models.py
    cp models/*.onnx brain/models/
fi

# Create deployment package
echo "📦 Creating deployment package..."
tar -czf /tmp/pitlinkpqc.tar.gz \
    target/release/dashboard \
    brain/models/*.onnx \
    dashboard/static/ 2>/dev/null || true

# Upload to Cloud Storage
echo "☁️ Uploading to Cloud Storage..."
gsutil cp /tmp/pitlinkpqc.tar.gz gs://$BUCKET_NAME/

# Create startup script
cat > /tmp/startup-script.sh << 'EOF'
#!/bin/bash
set -e

# Install dependencies
apt-get update
apt-get install -y curl build-essential ca-certificates

# Download application
gsutil cp gs://BUCKET_NAME/pitlinkpqc.tar.gz /tmp/
mkdir -p /opt/pitlinkpqc
cd /opt/pitlinkpqc
tar -xzf /tmp/pitlinkpqc.tar.gz

# Create systemd service
cat > /etc/systemd/system/pitlinkpqc.service << 'SERVICE'
[Unit]
Description=PitlinkPQC Server
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/pitlinkpqc
ExecStart=/opt/pitlinkpqc/dashboard
Restart=always
RestartSec=10
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
SERVICE

# Enable and start service
systemctl daemon-reload
systemctl enable pitlinkpqc
systemctl start pitlinkpqc

# Check status
sleep 5
systemctl status pitlinkpqc --no-pager
EOF

# Replace bucket name in startup script
sed -i "s/BUCKET_NAME/$BUCKET_NAME/g" /tmp/startup-script.sh

# Create VM instance
echo "🖥️ Creating VM instance..."
gcloud compute instances create $INSTANCE_NAME \
    --zone=$ZONE \
    --machine-type=$MACHINE_TYPE \
    --image-family=ubuntu-2204-lts \
    --image-project=ubuntu-os-cloud \
    --boot-disk-size=50GB \
    --boot-disk-type=pd-ssd \
    --tags=http-server \
    --metadata-from-file startup-script=/tmp/startup-script.sh \
    --scopes=storage-ro

# Configure firewall
echo "🔥 Configuring firewall..."
gcloud compute firewall-rules create allow-pitlinkpqc-http \
    --allow tcp:8080 \
    --source-ranges 0.0.0.0/0 \
    --description "Allow PitlinkPQC HTTP" 2>/dev/null || echo "Firewall rule already exists"

# Wait for instance to be ready
echo "⏳ Waiting for instance to be ready..."
sleep 30

# Get external IP
EXTERNAL_IP=$(gcloud compute instances describe $INSTANCE_NAME \
    --zone=$ZONE \
    --format='get(networkInterfaces[0].accessConfigs[0].natIP)')

echo ""
echo "✅ Deployment complete!"
echo ""
echo "📊 Dashboard: http://$EXTERNAL_IP:8080"
echo "🔌 API: http://$EXTERNAL_IP:8080/api"
echo ""
echo "📝 Next steps:"
echo "   1. Wait 1-2 minutes for service to start"
echo "   2. Access dashboard at http://$EXTERNAL_IP:8080"
echo "   3. Check logs: gcloud compute ssh $INSTANCE_NAME --zone=$ZONE --command='journalctl -u pitlinkpqc -f'"
echo ""

