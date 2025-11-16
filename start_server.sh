#!/bin/bash
# Start QUIC-FEC Server and Dashboard API

set -e

echo "🚀 Starting PitlinkPQC Server..."
echo "=================================="
echo ""

# Check if certificates exist
if [ ! -f "server.crt" ] || [ ! -f "server.key" ]; then
    echo "📜 Generating self-signed certificates..."
    openssl req -x509 -newkey rsa:4096 \
        -keyout server.key \
        -out server.crt \
        -days 365 \
        -nodes \
        -subj "/CN=localhost"
    echo "✅ Certificates generated"
    echo ""
fi

# Create storage directory
mkdir -p server_storage
echo "📁 Storage directory: ./server_storage"
echo ""

# Get server IP
SERVER_IP=$(hostname -I | awk '{print $1}')
echo "📡 Server IP: $SERVER_IP"
echo ""

# Start QUIC server in background
echo "🔐 Starting QUIC-FEC Server on 0.0.0.0:8443..."
cargo run --example server --package quic_fec -- 0.0.0.0:8443 ./server_storage > quic_server.log 2>&1 &
QUIC_PID=$!
echo "✅ QUIC Server started (PID: $QUIC_PID)"
echo ""

# Wait a moment for QUIC server to start
sleep 2

# Start Dashboard API
echo "📊 Starting Dashboard API on 0.0.0.0:8080..."
cargo run --bin dashboard --package dashboard > dashboard_api.log 2>&1 &
DASHBOARD_PID=$!
echo "✅ Dashboard API started (PID: $DASHBOARD_PID)"
echo ""

echo "✅ Server is running!"
echo ""
echo "📋 Services:"
echo "  • QUIC-FEC Server: 0.0.0.0:8443 (UDP)"
echo "  • Dashboard API:   0.0.0.0:8080 (HTTP)"
echo ""
echo "🌐 Client Configuration:"
echo "  Set NEXT_PUBLIC_BACKEND_URL=http://$SERVER_IP:8080"
echo ""
echo "📝 Logs:"
echo "  • QUIC Server: tail -f quic_server.log"
echo "  • Dashboard API: tail -f dashboard_api.log"
echo ""
echo "🛑 To stop:"
echo "  kill $QUIC_PID $DASHBOARD_PID"
echo ""

# Wait for user interrupt
trap "echo ''; echo '🛑 Stopping servers...'; kill $QUIC_PID $DASHBOARD_PID 2>/dev/null; exit" INT TERM

wait

