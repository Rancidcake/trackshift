#!/bin/bash
# Interactive Demonstration Script
# Guides user through testing the system

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

clear
echo -e "${BLUE}"
echo "╔══════════════════════════════════════════════════════════╗"
echo "║         PitlinkPQC System Demonstration                 ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo ""

# Get configuration
echo -e "${CYAN}📋 Configuration${NC}"
echo "================"
read -p "Server IP [localhost]: " SERVER_IP
SERVER_IP=${SERVER_IP:-localhost}

read -p "Server API Port [8080]: " SERVER_PORT
SERVER_PORT=${SERVER_PORT:-8080}

read -p "Client IP [localhost]: " CLIENT_IP
CLIENT_IP=${CLIENT_IP:-localhost}

read -p "Client Port [3000]: " CLIENT_PORT
CLIENT_PORT=${CLIENT_PORT:-3000}

echo ""
echo -e "${GREEN}✅ Configuration saved${NC}"
echo "   Server: http://$SERVER_IP:$SERVER_PORT"
echo "   Client: http://$CLIENT_IP:$CLIENT_PORT"
echo ""

# Menu
while true; do
    echo ""
    echo -e "${CYAN}📋 Demonstration Menu${NC}"
    echo "================"
    echo "1. Test Server Health"
    echo "2. Test Network Status"
    echo "3. Test Transfer List"
    echo "4. Test Statistics"
    echo "5. Create Test Files"
    echo "6. Upload Test File"
    echo "7. Run All Tests"
    echo "8. Open Dashboards"
    echo "9. View Logs"
    echo "0. Exit"
    echo ""
    read -p "Select option: " choice
    
    case $choice in
        1)
            echo ""
            echo -e "${YELLOW}Testing Server Health...${NC}"
            if curl -s -f "http://$SERVER_IP:$SERVER_PORT/api/health" > /dev/null; then
                echo -e "${GREEN}✅ Server is healthy${NC}"
                curl -s "http://$SERVER_IP:$SERVER_PORT/api/health" | jq '.' 2>/dev/null || curl -s "http://$SERVER_IP:$SERVER_PORT/api/health"
            else
                echo -e "${RED}❌ Server not responding${NC}"
            fi
            ;;
        2)
            echo ""
            echo -e "${YELLOW}Testing Network Status...${NC}"
            curl -s "http://$SERVER_IP:$SERVER_PORT/api/network" | jq '.' 2>/dev/null || curl -s "http://$SERVER_IP:$SERVER_PORT/api/network"
            ;;
        3)
            echo ""
            echo -e "${YELLOW}Testing Transfer List...${NC}"
            curl -s "http://$SERVER_IP:$SERVER_PORT/api/transfers" | jq '.' 2>/dev/null || curl -s "http://$SERVER_IP:$SERVER_PORT/api/transfers"
            ;;
        4)
            echo ""
            echo -e "${YELLOW}Testing Statistics...${NC}"
            curl -s "http://$SERVER_IP:$SERVER_PORT/api/stats" | jq '.' 2>/dev/null || curl -s "http://$SERVER_IP:$SERVER_PORT/api/stats"
            ;;
        5)
            echo ""
            echo -e "${YELLOW}Creating test files...${NC}"
            ./create_test_files.sh
            ;;
        6)
            echo ""
            echo -e "${YELLOW}Uploading test file...${NC}"
            if [ -f "./test_files/small_text.txt" ]; then
                curl -X POST -F "file=@./test_files/small_text.txt" "http://$CLIENT_IP:$CLIENT_PORT/api/upload"
                echo ""
            else
                echo -e "${RED}❌ Test file not found. Run option 5 first.${NC}"
            fi
            ;;
        7)
            echo ""
            echo -e "${YELLOW}Running all tests...${NC}"
            ./test_demo.sh "$SERVER_IP" "$SERVER_PORT" "$CLIENT_IP" "$CLIENT_PORT"
            ;;
        8)
            echo ""
            echo -e "${GREEN}Opening dashboards...${NC}"
            echo "   Client: http://$CLIENT_IP:$CLIENT_PORT"
            echo "   Server: http://$CLIENT_IP:$CLIENT_PORT/server"
            if command -v xdg-open &> /dev/null; then
                xdg-open "http://$CLIENT_IP:$CLIENT_PORT" 2>/dev/null || true
            elif command -v open &> /dev/null; then
                open "http://$CLIENT_IP:$CLIENT_PORT" 2>/dev/null || true
            else
                echo "   Please open manually in your browser"
            fi
            ;;
        9)
            echo ""
            echo -e "${YELLOW}Viewing logs...${NC}"
            echo "Server logs:"
            echo "  sudo journalctl -u pitlink-quic -n 20"
            echo "  sudo journalctl -u pitlink-dashboard -n 20"
            echo ""
            echo "Client logs:"
            echo "  sudo journalctl -u pitlink-client -n 20"
            ;;
        0)
            echo ""
            echo -e "${GREEN}👋 Goodbye!${NC}"
            exit 0
            ;;
        *)
            echo -e "${RED}Invalid option${NC}"
            ;;
    esac
done

