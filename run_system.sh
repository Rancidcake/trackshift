#!/bin/bash
# Complete system test and run script

set -e

echo "🚀 PitlinkPQC System Test & Run"
echo "================================"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Step 1: Check dependencies
echo "📦 Step 1: Checking dependencies..."
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo not found. Please install Rust.${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Cargo found${NC}"
echo ""

# Step 2: Build all packages
echo "🔨 Step 2: Building all packages..."
cargo build --workspace 2>&1 | tail -20
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Build successful${NC}"
echo ""

# Step 3: Generate certificate for server
echo "🔐 Step 3: Generating certificate..."
cd quic_fec/examples
if [ ! -f "server.crt" ] || [ ! -f "server.key" ]; then
    chmod +x generate_cert.sh
    ./generate_cert.sh 2>&1
    if [ $? -ne 0 ]; then
        echo -e "${YELLOW}⚠️  Certificate generation failed, but continuing...${NC}"
    else
        echo -e "${GREEN}✅ Certificate generated${NC}"
    fi
else
    echo -e "${GREEN}✅ Certificate already exists${NC}"
fi
cd ../..
echo ""

# Step 4: Create test file
echo "📝 Step 4: Creating test file..."
TEST_FILE="test_transfer.txt"
cat > "$TEST_FILE" << EOF
This is a test file for PitlinkPQC file transfer system.
It contains multiple lines to test chunking and transfer.

Line 1: Testing file transfer functionality
Line 2: Testing chunk-based transfer
Line 3: Testing integrity verification
Line 4: Testing progress tracking
Line 5: Testing fallback system

The system should handle this file transfer smoothly.
EOF
echo -e "${GREEN}✅ Test file created: $TEST_FILE${NC}"
echo ""

# Step 5: Create storage directory
echo "📁 Step 5: Creating storage directories..."
mkdir -p server_storage
mkdir -p server_storage/temp
echo -e "${GREEN}✅ Storage directories created${NC}"
echo ""

# Step 6: Summary
echo "================================"
echo "✅ System Ready!"
echo ""
echo "📋 Next Steps:"
echo ""
echo "1. Start Dashboard (Terminal 1):"
echo "   cargo run --package dashboard"
echo ""
echo "2. Start Server (Terminal 2):"
echo "   cargo run --example server -- 127.0.0.1:8080"
echo ""
echo "3. Run Client (Terminal 3):"
echo "   cargo run --example client -- 127.0.0.1:8080 $TEST_FILE /uploads/$TEST_FILE"
echo ""
echo "4. Access Dashboard:"
echo "   http://localhost:8080"
echo ""
echo "================================"

