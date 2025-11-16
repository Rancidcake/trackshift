# PitlinkPQC Demonstration & Testing Guide

## Quick Start

### 1. Deploy to Google Cloud (Automated)

```bash
# One command deployment
chmod +x deploy_gcp_complete.sh
./deploy_gcp_complete.sh PROJECT_ID ZONE REPO_URL

# Example:
./deploy_gcp_complete.sh my-project us-central1-a https://github.com/user/repo.git
```

This will:
- Create two GCP instances (server + client)
- Install all dependencies
- Deploy and start services
- Configure firewall rules
- Print access URLs

---

### 2. Manual Deployment

#### Server Instance

```bash
# SSH to server instance
gcloud compute ssh pitlink-server --zone=us-central1-a

# Clone repo and deploy
git clone <YOUR_REPO_URL> ~/PitlinkPQC
cd ~/PitlinkPQC
chmod +x deploy_server.sh
./deploy_server.sh
```

#### Client Instance

```bash
# SSH to client instance
gcloud compute ssh pitlink-client --zone=us-central1-a

# Clone repo and deploy
git clone <YOUR_REPO_URL> ~/PitlinkPQC
cd ~/PitlinkPQC
chmod +x deploy_client.sh
SERVER_IP=<SERVER_IP> ./deploy_client.sh
```

---

## Testing & Demonstration

### Run Automated Tests

```bash
# Run all tests
chmod +x test_demo.sh
./test_demo.sh SERVER_IP 8080 CLIENT_IP 3000

# Example:
./test_demo.sh 192.168.1.100 8080 192.168.1.101 3000
```

### Interactive Demo

```bash
# Interactive demonstration menu
chmod +x run_demo.sh
./run_demo.sh
```

### Create Test Files

```bash
# Generate test files
chmod +x create_test_files.sh
./create_test_files.sh
```

Test files created:
- `small_text.txt` - 1 KB text file
- `medium_text.txt` - 100 KB text file
- `large_binary.bin` - 1 MB binary file
- `test_data.csv` - CSV data file
- `test_data.json` - JSON data file

---

## Manual Testing

### 1. Test Server Health

```bash
curl http://SERVER_IP:8080/api/health
```

### 2. Test Network Status

```bash
curl http://SERVER_IP:8080/api/network
```

### 3. Test Transfer List

```bash
curl http://SERVER_IP:8080/api/transfers
```

### 4. Test Statistics

```bash
curl http://SERVER_IP:8080/api/stats
```

### 5. Upload File via API

```bash
curl -X POST -F "file=@test.txt" http://CLIENT_IP:3000/api/upload
```

---

## Access Dashboards

### Client Dashboard
- **URL**: `http://CLIENT_IP:3000`
- **Features**: File upload, transfer management

### Server Dashboard
- **URL**: `http://CLIENT_IP:3000/server`
- **Features**: System monitoring, health metrics

---

## Service Management

### Server Instance

```bash
# View QUIC server logs
sudo journalctl -u pitlink-quic -f

# View Dashboard API logs
sudo journalctl -u pitlink-dashboard -f

# Restart services
sudo systemctl restart pitlink-quic pitlink-dashboard

# Stop services
sudo systemctl stop pitlink-quic pitlink-dashboard
```

### Client Instance

```bash
# View dashboard logs
sudo journalctl -u pitlink-client -f

# Restart service
sudo systemctl restart pitlink-client

# Stop service
sudo systemctl stop pitlink-client
```

---

## Troubleshooting

### Server Not Responding

1. Check service status:
   ```bash
   sudo systemctl status pitlink-quic
   sudo systemctl status pitlink-dashboard
   ```

2. Check logs:
   ```bash
   sudo journalctl -u pitlink-quic -n 50
   sudo journalctl -u pitlink-dashboard -n 50
   ```

3. Check firewall:
   ```bash
   sudo ufw status
   # Allow ports if needed
   sudo ufw allow 8443/udp
   sudo ufw allow 8080/tcp
   ```

### Client Dashboard Not Loading

1. Check service status:
   ```bash
   sudo systemctl status pitlink-client
   ```

2. Check environment:
   ```bash
   cat dashboard-web/.env.local
   ```

3. Check logs:
   ```bash
   sudo journalctl -u pitlink-client -n 50
   ```

### Connection Issues

1. Verify server IP is correct in client `.env.local`
2. Test connectivity:
   ```bash
   curl http://SERVER_IP:8080/api/health
   ```
3. Check firewall rules on GCP

---

## Test Scenarios

### Scenario 1: Small File Transfer
1. Create small test file: `./create_test_files.sh`
2. Upload via dashboard: `http://CLIENT_IP:3000`
3. Monitor progress in dashboard
4. Verify file received in `server_storage/`

### Scenario 2: Large File Transfer
1. Create large test file: `dd if=/dev/urandom of=large.bin bs=1M count=100`
2. Upload via dashboard
3. Monitor transfer speed and progress
4. Check for FEC recovery if packets lost

### Scenario 3: Multiple Concurrent Transfers
1. Upload multiple files simultaneously
2. Monitor queue and priority handling
3. Check transfer statistics

### Scenario 4: Network Failure Simulation
1. Start transfer
2. Simulate network issues (disable interface)
3. Observe handover behavior
4. Verify transfer completion after recovery

---

## Performance Testing

### Latency Test

```bash
# Measure API response time
time curl -s http://SERVER_IP:8080/api/health > /dev/null
```

### Throughput Test

```bash
# Upload large file and measure speed
time curl -X POST -F "file=@large.bin" http://CLIENT_IP:3000/api/upload
```

### Load Test

```bash
# Multiple concurrent requests
for i in {1..10}; do
    curl -s http://SERVER_IP:8080/api/health > /dev/null &
done
wait
```

---

## Monitoring

### Real-time Metrics

- **Client Dashboard**: `http://CLIENT_IP:3000`
  - Active transfers
  - Transfer history
  - Upload interface

- **Server Dashboard**: `http://CLIENT_IP:3000/server`
  - CPU/Memory usage
  - Network metrics
  - Transfer statistics
  - Error rates

### Log Monitoring

```bash
# Follow all logs
sudo journalctl -u pitlink-quic -u pitlink-dashboard -u pitlink-client -f
```

---

## Cleanup

### Stop All Services

```bash
# Server
sudo systemctl stop pitlink-quic pitlink-dashboard

# Client
sudo systemctl stop pitlink-client
```

### Remove Services

```bash
# Server
sudo systemctl disable pitlink-quic pitlink-dashboard
sudo rm /etc/systemd/system/pitlink-*.service
sudo systemctl daemon-reload

# Client
sudo systemctl disable pitlink-client
sudo rm /etc/systemd/system/pitlink-client.service
sudo systemctl daemon-reload
```

### Delete GCP Instances

```bash
gcloud compute instances delete pitlink-server pitlink-client --zone=us-central1-a
gcloud compute firewall-rules delete allow-quic allow-dashboard-api allow-nextjs allow-ssh
```

---

## Quick Reference

### Scripts

- `deploy_server.sh` - Deploy server instance
- `deploy_client.sh` - Deploy client instance
- `deploy_gcp_complete.sh` - Complete automated deployment
- `test_demo.sh` - Run all tests
- `run_demo.sh` - Interactive demo
- `create_test_files.sh` - Generate test files

### Ports

- **8443** (UDP) - QUIC server
- **8080** (TCP) - Dashboard API
- **3000** (TCP) - Next.js dashboard

### Directories

- `server_storage/` - Received files on server
- `dashboard-web/uploads/` - Uploaded files on client
- `logs/` - Service logs
- `certs/` - SSL certificates
- `test_files/` - Test files for demo

---

## Success Criteria

✅ **System is working if:**
- Server health endpoint responds
- Client dashboard loads
- File upload succeeds
- Transfer appears in history
- Server receives file in `server_storage/`
- Metrics update in real-time

---

**Happy Testing!** 🚀

