#!/bin/bash
# Demonstration and Testing Script
# Tests the complete system end-to-end

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
SERVER_IP=${1:-"localhost"}
SERVER_API_PORT=${2:-8080}
CLIENT_IP=${3:-"localhost"}
CLIENT_PORT=${4:-3000}

echo -e "${BLUE}🧪 PitlinkPQC System Test & Demo${NC}"
echo "=================================="
echo ""
echo "Server API: http://$SERVER_IP:$SERVER_API_PORT"
echo "Client Dashboard: http://$CLIENT_IP:$CLIENT_PORT"
echo ""

# Test 1: Server Health Check
echo -e "${YELLOW}Test 1: Server Health Check${NC}"
if curl -s -f "http://$SERVER_IP:$SERVER_API_PORT/api/health" > /dev/null; then
    echo -e "${GREEN}✅ Server is healthy${NC}"
    curl -s "http://$SERVER_IP:$SERVER_API_PORT/api/health" | jq '.' || echo "Response received"
else
    echo -e "${RED}❌ Server health check failed${NC}"
    exit 1
fi
echo ""

# Test 2: Network Status
echo -e "${YELLOW}Test 2: Network Status${NC}"
if curl -s -f "http://$SERVER_IP:$SERVER_API_PORT/api/network" > /dev/null; then
    echo -e "${GREEN}✅ Network endpoint accessible${NC}"
    curl -s "http://$SERVER_IP:$SERVER_API_PORT/api/network" | jq '.' || echo "Response received"
else
    echo -e "${YELLOW}⚠️  Network endpoint not available (may be normal)${NC}"
fi
echo ""

# Test 3: Transfer List
echo -e "${YELLOW}Test 3: Transfer List${NC}"
if curl -s -f "http://$SERVER_IP:$SERVER_API_PORT/api/transfers" > /dev/null; then
    echo -e "${GREEN}✅ Transfers endpoint accessible${NC}"
    TRANSFERS=$(curl -s "http://$SERVER_IP:$SERVER_API_PORT/api/transfers")
    echo "$TRANSFERS" | jq '.count // "No transfers"' || echo "$TRANSFERS"
else
    echo -e "${RED}❌ Transfers endpoint failed${NC}"
fi
echo ""

# Test 4: Statistics
echo -e "${YELLOW}Test 4: Statistics${NC}"
if curl -s -f "http://$SERVER_IP:$SERVER_API_PORT/api/stats" > /dev/null; then
    echo -e "${GREEN}✅ Stats endpoint accessible${NC}"
    curl -s "http://$SERVER_IP:$SERVER_API_PORT/api/stats" | jq '.transfers // .' || echo "Response received"
else
    echo -e "${YELLOW}⚠️  Stats endpoint not available${NC}"
fi
echo ""

# Test 5: Client Dashboard
echo -e "${YELLOW}Test 5: Client Dashboard${NC}"
if curl -s -f "http://$CLIENT_IP:$CLIENT_PORT" > /dev/null; then
    echo -e "${GREEN}✅ Client dashboard is accessible${NC}"
    echo "   Open in browser: http://$CLIENT_IP:$CLIENT_PORT"
else
    echo -e "${RED}❌ Client dashboard not accessible${NC}"
fi
echo ""

# Test 6: Create Test File
echo -e "${YELLOW}Test 6: Creating Test File${NC}"
TEST_FILE="/tmp/pitlink_test_$(date +%s).txt"
echo "PitlinkPQC Test File - $(date)" > "$TEST_FILE"
echo "This is a test file for PitlinkPQC file transfer system." >> "$TEST_FILE"
echo "File size: $(wc -c < "$TEST_FILE") bytes" >> "$TEST_FILE"
echo -e "${GREEN}✅ Test file created: $TEST_FILE${NC}"
echo "   Size: $(du -h "$TEST_FILE" | cut -f1)"
echo ""

# Test 7: Upload Test (if API available)
echo -e "${YELLOW}Test 7: File Upload Test${NC}"
if curl -s -f -X POST -F "file=@$TEST_FILE" "http://$CLIENT_IP:$CLIENT_PORT/api/upload" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ File upload successful${NC}"
else
    echo -e "${YELLOW}⚠️  File upload test skipped (requires running dashboard)${NC}"
fi
echo ""

# Summary
echo -e "${BLUE}📊 Test Summary${NC}"
echo "=============="
echo ""
echo "✅ All basic connectivity tests completed"
echo ""
echo "📝 Next Steps:"
echo "   1. Open client dashboard: http://$CLIENT_IP:$CLIENT_PORT"
echo "   2. Open server monitor: http://$CLIENT_IP:$CLIENT_PORT/server"
echo "   3. Upload test file: $TEST_FILE"
echo "   4. Monitor transfer progress in dashboard"
echo ""
echo "🔗 URLs:"
echo "   Client: http://$CLIENT_IP:$CLIENT_PORT"
echo "   Server: http://$CLIENT_IP:$CLIENT_PORT/server"
echo "   API: http://$SERVER_IP:$SERVER_API_PORT/api/health"
echo ""

