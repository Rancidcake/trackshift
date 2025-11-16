#!/bin/bash
# Start Sender-Side Services

echo "📤 Starting PitlinkPQC Sender Side..."
echo ""

# Check if receiver address is set
if [ -z "$RECEIVER_ADDRESS" ]; then
    echo "⚠️  RECEIVER_ADDRESS not set!"
    echo "   Set it in dashboard-web/.env.local or export RECEIVER_ADDRESS=IP:PORT"
    echo ""
    read -p "Enter receiver IP:PORT (e.g., 192.168.1.100:8081): " RECEIVER_ADDRESS
    export RECEIVER_ADDRESS
fi

echo "📡 Receiver address: $RECEIVER_ADDRESS"
echo ""

# Start Rust backend
echo "🚀 Starting Rust backend (port 8080)..."
cd "$(dirname "$0")"
cargo run --release -p dashboard > /tmp/sender_backend.log 2>&1 &
BACKEND_PID=$!
echo "   Backend PID: $BACKEND_PID"
sleep 3

# Start Next.js frontend
echo "🌐 Starting Next.js frontend (port 3000)..."
cd dashboard-web
npm run dev > /tmp/sender_frontend.log 2>&1 &
FRONTEND_PID=$!
echo "   Frontend PID: $FRONTEND_PID"
sleep 5

echo ""
echo "✅ Sender services started!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🌐 File Upload Client:"
echo "   👉 http://localhost:3000/client"
echo ""
echo "📊 Dashboard API:"
echo "   👉 http://localhost:8080"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📝 Logs:"
echo "   Backend: tail -f /tmp/sender_backend.log"
echo "   Frontend: tail -f /tmp/sender_frontend.log"
echo ""
echo "🛑 Stop: kill $BACKEND_PID $FRONTEND_PID"
