# Quick Start - GCP Deployment

## 🚀 One-Command Deployment

```bash
# Make script executable
chmod +x deploy_gcp_complete.sh

# Run deployment (replace with your values)
./deploy_gcp_complete.sh YOUR_PROJECT_ID us-central1-a YOUR_REPO_URL

# Example:
./deploy_gcp_complete.sh pitlinkpqc us-central1-a https://github.com/yourusername/PitlinkPQC.git
```

**That's it!** The script will:
- ✅ Create two GCP instances
- ✅ Install all dependencies
- ✅ Deploy server and client
- ✅ Configure firewall
- ✅ Start all services
- ✅ Print access URLs

---

## 📋 Prerequisites

1. **Google Cloud Account** with billing enabled
2. **gcloud CLI** installed and authenticated:
   ```bash
   gcloud auth login
   gcloud config set project YOUR_PROJECT_ID
   ```
3. **Git repository** with your code

---

## 🔧 Manual Deployment (Step-by-Step)

### Step 1: Create Instances

```bash
# Set your project
export PROJECT_ID=pitlinkpqc
gcloud config set project $PROJECT_ID

# Create server instance
gcloud compute instances create pitlink-server \
  --zone=us-central1-a \
  --machine-type=e2-medium \
  --image-family=ubuntu-2204-lts \
  --image-project=ubuntu-os-cloud \
  --boot-disk-size=20GB

# Create client instance
gcloud compute instances create pitlink-client \
  --zone=us-central1-a \
  --machine-type=e2-medium \
  --image-family=ubuntu-2204-lts \
  --image-project=ubuntu-os-cloud \
  --boot-disk-size=20GB
```

### Step 2: Configure Firewall

```bash
# Allow QUIC (UDP 8443)
gcloud compute firewall-rules create allow-quic \
  --allow udp:8443 \
  --source-ranges 0.0.0.0/0

# Allow Dashboard API (TCP 8080)
gcloud compute firewall-rules create allow-dashboard-api \
  --allow tcp:8080 \
  --source-ranges 0.0.0.0/0

# Allow Next.js Dashboard (TCP 3000)
gcloud compute firewall-rules create allow-nextjs \
  --allow tcp:3000 \
  --source-ranges 0.0.0.0/0

# Allow SSH
gcloud compute firewall-rules create allow-ssh \
  --allow tcp:22 \
  --source-ranges 0.0.0.0/0
```

### Step 3: Deploy Server

```bash
# Get server IP
SERVER_IP=$(gcloud compute instances describe pitlink-server \
  --zone=us-central1-a \
  --format='get(networkInterfaces[0].accessConfigs[0].natIP)')

# Upload and run deployment script
gcloud compute scp deploy_server.sh pitlink-server:~/ --zone=us-central1-a
gcloud compute ssh pitlink-server --zone=us-central1-a --command="
  git clone YOUR_REPO_URL ~/PitlinkPQC &&
  cd ~/PitlinkPQC &&
  chmod +x deploy_server.sh &&
  ./deploy_server.sh
"
```

### Step 4: Deploy Client

```bash
# Get client IP
CLIENT_IP=$(gcloud compute instances describe pitlink-client \
  --zone=us-central1-a \
  --format='get(networkInterfaces[0].accessConfigs[0].natIP)')

# Upload and run deployment script
gcloud compute scp deploy_client.sh pitlink-client:~/ --zone=us-central1-a
gcloud compute ssh pitlink-client --zone=us-central1-a --command="
  git clone YOUR_REPO_URL ~/PitlinkPQC &&
  cd ~/PitlinkPQC &&
  chmod +x deploy_client.sh &&
  SERVER_IP=$SERVER_IP ./deploy_client.sh
"
```

---

## 🧪 Testing

### Quick Test

```bash
# Get IPs
SERVER_IP=$(gcloud compute instances describe pitlink-server --zone=us-central1-a --format='get(networkInterfaces[0].accessConfigs[0].natIP)')
CLIENT_IP=$(gcloud compute instances describe pitlink-client --zone=us-central1-a --format='get(networkInterfaces[0].accessConfigs[0].natIP)')

# Test server
curl http://$SERVER_IP:8080/api/health

# Test client
curl http://$CLIENT_IP:3000
```

### Full Test Suite

```bash
# Upload test script
gcloud compute scp test_demo.sh pitlink-client:~/ --zone=us-central1-a

# Run tests
gcloud compute ssh pitlink-client --zone=us-central1-a --command="
  cd ~/PitlinkPQC &&
  chmod +x test_demo.sh &&
  ./test_demo.sh $SERVER_IP 8080 $CLIENT_IP 3000
"
```

---

## 📊 Access Dashboards

After deployment, access:

- **Client Dashboard**: `http://CLIENT_IP:3000`
- **Server Monitor**: `http://CLIENT_IP:3000/server`
- **API Health**: `http://SERVER_IP:8080/api/health`

---

## 🛠️ Service Management

### Server Instance

```bash
# SSH to server
gcloud compute ssh pitlink-server --zone=us-central1-a

# View logs
sudo journalctl -u pitlink-quic -f
sudo journalctl -u pitlink-dashboard -f

# Restart services
sudo systemctl restart pitlink-quic pitlink-dashboard
```

### Client Instance

```bash
# SSH to client
gcloud compute ssh pitlink-client --zone=us-central1-a

# View logs
sudo journalctl -u pitlink-client -f

# Restart service
sudo systemctl restart pitlink-client
```

---

## 🧹 Cleanup

```bash
# Delete instances
gcloud compute instances delete pitlink-server pitlink-client --zone=us-central1-a

# Delete firewall rules
gcloud compute firewall-rules delete allow-quic allow-dashboard-api allow-nextjs allow-ssh
```

---

## 📝 Next Steps

1. **Test the system**: Run `./test_demo.sh`
2. **Upload files**: Use client dashboard
3. **Monitor**: Check server dashboard
4. **Review logs**: Check service logs for issues

---

**Ready to deploy!** 🚀

