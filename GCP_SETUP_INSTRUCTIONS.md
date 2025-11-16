# Google Cloud Platform Setup Instructions

## Prerequisites

To deploy to GCP, you need:
1. Google Cloud SDK (`gcloud`) installed
2. A GCP project with billing enabled
3. Authentication configured

## Quick Installation (macOS)

### Option 1: Using Homebrew (Recommended)
```bash
brew install --cask google-cloud-sdk
```

### Option 2: Manual Installation
```bash
# Download and install
curl https://sdk.cloud.google.com | bash

# Restart your shell or run:
exec -l $SHELL

# Initialize gcloud
gcloud init
```

## Authentication

After installing, authenticate:
```bash
# Login to your Google account
gcloud auth login

# Set your project
gcloud config set project calm-cab-478400-r8

# Verify
gcloud config list
```

## Run Deployment

Once `gcloud` is installed and authenticated:
```bash
cd /Users/mayankhete/trs/PitlinkPQC
./deploy_gcp_complete.sh calm-cab-478400-r8 us-central1-a https://github.com/Rancidcake/PitlinkPQC1
```

## What the Script Does

1. **Creates two VM instances:**
   - `pitlink-server` - Runs QUIC server and Rust dashboard API
   - `pitlink-client` - Runs Next.js client dashboard

2. **Installs dependencies:**
   - Rust toolchain
   - Node.js 20.x
   - Build tools and libraries

3. **Configures firewall rules:**
   - UDP 8443 (QUIC)
   - TCP 8080 (Dashboard API)
   - TCP 3000 (Next.js)
   - TCP 22 (SSH)

4. **Deploys the application:**
   - Clones repository from GitHub
   - Builds and starts server components
   - Builds and starts client dashboard

## Expected Output

After successful deployment, you'll see:
- Server IP address (QUIC: `IP:8443`, API: `http://IP:8080`)
- Client IP address (Dashboard: `http://IP:3000`)
- Test connection command

## Troubleshooting

### If instances already exist:
The script will fail if instances with the same names exist. Delete them first:
```bash
gcloud compute instances delete pitlink-server pitlink-client --zone=us-central1-a
```

### If firewall rules already exist:
The script handles this gracefully (uses `|| true`), so it won't fail.

### Check instance status:
```bash
gcloud compute instances list --project=calm-cab-478400-r8
```

### SSH into instances:
```bash
gcloud compute ssh pitlink-server --zone=us-central1-a
gcloud compute ssh pitlink-client --zone=us-central1-a
```

