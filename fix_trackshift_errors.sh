#!/bin/bash
# Fix compilation errors in trackshift/brain code after cloning

set -e

BRAIN_DIR="${1:-brain}"

if [ ! -d "$BRAIN_DIR" ]; then
    echo "Error: brain directory not found: $BRAIN_DIR"
    exit 1
fi

echo "🔧 Fixing compilation errors in trackshift/brain..."

# Fix 1: ORT imports (if not already fixed)
if grep -q "use ort::{" "$BRAIN_DIR/src/telemetry_ai/mod.rs" 2>/dev/null; then
    echo "   Fixing ORT imports..."
    sed -i 's/use ort::{Session, SessionBuilder, Value};/use ort::session::{Session, SessionInputValue};\nuse ort::session::builder::SessionBuilder;\nuse ort::value::Value;/' "$BRAIN_DIR/src/telemetry_ai/mod.rs" 2>/dev/null || true
fi

# Fix 2: Type ambiguity in network_quality.rs
if grep -q "score = score.max(0.0)" "$BRAIN_DIR/src/telemetry_ai/network_quality.rs" 2>/dev/null; then
    echo "   Fixing type ambiguity in network_quality.rs..."
    sed -i 's/score = score\.max(0\.0)\.min(1\.0);/score = score.max(0.0_f32).min(1.0_f32);/' "$BRAIN_DIR/src/telemetry_ai/network_quality.rs" 2>/dev/null || true
fi

# Fix 3: Type mismatches - file_size comparisons with float literals
if grep -q "file_size > [0-9_]*\.0" "$BRAIN_DIR/src/telemetry_ai/mod.rs" 2>/dev/null; then
    echo "   Fixing file_size type mismatches..."
    # Replace float literals with integer literals for u64 comparisons
    sed -i 's/file_size > 100_000_000\.0/file_size > 100_000_000/g' "$BRAIN_DIR/src/telemetry_ai/mod.rs" 2>/dev/null || true
    sed -i 's/file_size > 10_000_000\.0/file_size > 10_000_000/g' "$BRAIN_DIR/src/telemetry_ai/mod.rs" 2>/dev/null || true
fi

# Fix 4: Ensure score variable is explicitly f32
if ! grep -q "let mut score: f32" "$BRAIN_DIR/src/telemetry_ai/network_quality.rs" 2>/dev/null; then
    echo "   Ensuring score is explicitly f32..."
    sed -i 's/let mut score = 1\.0;/let mut score: f32 = 1.0;/' "$BRAIN_DIR/src/telemetry_ai/network_quality.rs" 2>/dev/null || true
fi

echo "✅ Compilation fixes applied!"

