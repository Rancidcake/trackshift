#!/bin/bash
# Start Client Dashboard

set -e

echo "🚀 Starting PitlinkPQC Client Dashboard..."
echo "==========================================="
echo ""

# Check for .env.local
if [ ! -f "dashboard-web/.env.local" ]; then
    echo "⚠️  .env.local not found. Creating from example..."
    echo "NEXT_PUBLIC_BACKEND_URL=http://localhost:8080" > dashboard-web/.env.local
    echo "BACKEND_URL=http://localhost:8080" >> dashboard-web/.env.local
    echo ""
    echo "📝 Please edit dashboard-web/.env.local with your server IP"
    echo "   Example: NEXT_PUBLIC_BACKEND_URL=http://192.168.1.100:8080"
    echo ""
    read -p "Press Enter to continue or Ctrl+C to edit .env.local first..."
fi

cd dashboard-web

echo "📦 Installing dependencies (if needed)..."
npm install --silent

echo ""
echo "🌐 Starting Next.js dashboard..."
echo "   Dashboard: http://localhost:3000"
echo "   Backend:   $(grep NEXT_PUBLIC_BACKEND_URL .env.local | cut -d'=' -f2)"
echo ""

npm run dev

