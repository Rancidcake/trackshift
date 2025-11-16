#!/bin/bash
# Generate self-signed certificate for QUIC-FEC server

echo "🔐 Generating self-signed certificate for QUIC-FEC server..."

# Generate private key
openssl genrsa -out server.key 2048

# Generate certificate signing request
openssl req -new -key server.key -out server.csr -subj "/CN=localhost"

# Generate self-signed certificate (valid for 1 year)
openssl x509 -req -days 365 -in server.csr -signkey server.key -out server.crt

# Clean up CSR
rm server.csr

echo "✅ Certificate generated:"
echo "   - server.crt (certificate)"
echo "   - server.key (private key)"
echo ""
echo "⚠️  Note: This is a self-signed certificate for testing only."
echo "   For production, use proper certificates from a CA."

