#!/bin/bash
# Check GCP Prerequisites and Guide Setup

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🔍 Checking GCP Prerequisites...${NC}"
echo ""

# Check for gcloud
if command -v gcloud &> /dev/null; then
    echo -e "${GREEN}✅ gcloud is installed${NC}"
    gcloud --version | head -1
else
    echo -e "${RED}❌ gcloud is not installed${NC}"
    echo ""
    echo "To install:"
    echo "  1. Using Homebrew:"
    echo "     brew install --cask google-cloud-sdk"
    echo ""
    echo "  2. After installation, add to PATH:"
    echo "     export PATH=\"/opt/homebrew/share/google-cloud-sdk/bin:\$PATH\""
    echo "     (Add this to your ~/.zshrc for persistence)"
    echo ""
    echo "  3. Initialize:"
    echo "     gcloud init"
    exit 1
fi

echo ""

# Check authentication
if gcloud auth list --filter=status:ACTIVE --format="value(account)" | grep -q .; then
    echo -e "${GREEN}✅ Authenticated to GCP${NC}"
    gcloud auth list --filter=status:ACTIVE --format="table(account,status)"
else
    echo -e "${YELLOW}⚠️  Not authenticated${NC}"
    echo ""
    echo "To authenticate:"
    echo "  gcloud auth login"
    exit 1
fi

echo ""

# Check project
CURRENT_PROJECT=$(gcloud config get-value project 2>/dev/null || echo "")
if [ -n "$CURRENT_PROJECT" ]; then
    echo -e "${GREEN}✅ Current project: $CURRENT_PROJECT${NC}"
    if [ "$CURRENT_PROJECT" != "calm-cab-478400-r8" ]; then
        echo -e "${YELLOW}⚠️  Project mismatch. Setting to calm-cab-478400-r8...${NC}"
        gcloud config set project calm-cab-478400-r8
    fi
else
    echo -e "${YELLOW}⚠️  No project set${NC}"
    echo "Setting project to calm-cab-478400-r8..."
    gcloud config set project calm-cab-478400-r8
fi

echo ""
echo -e "${GREEN}✅ All prerequisites met!${NC}"
echo ""
echo "You can now run:"
echo "  ./deploy_gcp_complete.sh calm-cab-478400-r8 us-central1-a https://github.com/Rancidcake/PitlinkPQC1"

