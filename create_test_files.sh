#!/bin/bash
# Create Test Files for Demonstration
# Generates various test files of different sizes and types

set -e

TEST_DIR="./test_files"
mkdir -p "$TEST_DIR"

echo "📁 Creating test files for demonstration..."
echo ""

# Small text file (1 KB)
echo "Creating small text file..."
cat > "$TEST_DIR/small_text.txt" <<EOF
PitlinkPQC Test File - Small Text
==================================

This is a small text file for testing the file transfer system.
File size: ~1 KB
Created: $(date)
EOF

# Medium text file (100 KB)
echo "Creating medium text file..."
{
    echo "PitlinkPQC Test File - Medium Text"
    echo "=================================="
    echo ""
    for i in {1..1000}; do
        echo "Line $i: This is test data for PitlinkPQC file transfer system. $(date +%s)"
    done
} > "$TEST_DIR/medium_text.txt"

# Large binary file (1 MB)
echo "Creating large binary file..."
dd if=/dev/urandom of="$TEST_DIR/large_binary.bin" bs=1024 count=1024 2>/dev/null

# CSV test file
echo "Creating CSV test file..."
cat > "$TEST_DIR/test_data.csv" <<EOF
id,name,value,timestamp
1,Test Item 1,100.50,$(date +%s)
2,Test Item 2,200.75,$(date +%s)
3,Test Item 3,300.25,$(date +%s)
4,Test Item 4,400.00,$(date +%s)
5,Test Item 5,500.50,$(date +%s)
EOF

# JSON test file
echo "Creating JSON test file..."
cat > "$TEST_DIR/test_data.json" <<EOF
{
  "system": "PitlinkPQC",
  "version": "1.0.0",
  "test_data": {
    "timestamp": $(date +%s),
    "files": [
      {"name": "small_text.txt", "size": 1024},
      {"name": "medium_text.txt", "size": 102400},
      {"name": "large_binary.bin", "size": 1048576}
    ],
    "metrics": {
      "rtt": 15.2,
      "throughput": 100.5,
      "loss_rate": 0.01
    }
  }
}
EOF

# Create file manifest
cat > "$TEST_DIR/manifest.txt" <<EOF
PitlinkPQC Test Files Manifest
==============================

Files created: $(date)
Total files: 5

1. small_text.txt - Small text file (~1 KB)
2. medium_text.txt - Medium text file (~100 KB)
3. large_binary.bin - Large binary file (1 MB)
4. test_data.csv - CSV data file
5. test_data.json - JSON data file

Usage:
  Upload any of these files through the client dashboard to test the transfer system.
EOF

echo ""
echo "✅ Test files created in $TEST_DIR/"
echo ""
ls -lh "$TEST_DIR/"
echo ""
echo "📝 Manifest: $TEST_DIR/manifest.txt"

