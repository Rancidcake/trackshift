#!/bin/bash
# Comprehensive fix for trackshift compilation errors

set -e

BRAIN_DIR="${1:-brain}"

if [ ! -d "$BRAIN_DIR/src/telemetry_ai" ]; then
    echo "Error: brain/src/telemetry_ai directory not found"
    exit 1
fi

cd "$BRAIN_DIR/src/telemetry_ai"

echo "🔧 Applying comprehensive fixes..."

# 1. Fix ORT imports
if grep -q "use ort::{Session, SessionBuilder, Value};" mod.rs; then
    sed -i 's/use ort::{Session, SessionBuilder, Value};/use ort::session::Session;\nuse ort::session::builder::SessionBuilder;\nuse ort::value::Value;/' mod.rs
fi

# 2. Add missing imports at the very top (after any existing use statements)
# First, find where to insert
if ! grep -q "^use std::sync::Arc;" mod.rs; then
    # Insert after the first use statement or at line 1
    sed -i '1i use std::sync::Arc;' mod.rs
fi

# Add anyhow imports
if ! grep -q "use anyhow::Context" mod.rs; then
    # Find the first anyhow import or add at top
    if grep -q "^use anyhow::" mod.rs; then
        sed -i '/^use anyhow::/a use anyhow::Context;' mod.rs
    else
        sed -i '1i use anyhow::Context;' mod.rs
    fi
fi

if ! grep -q "use anyhow::Result" mod.rs; then
    if grep -q "^use anyhow::" mod.rs; then
        sed -i '/^use anyhow::/a use anyhow::Result;' mod.rs
    else
        sed -i '1i use anyhow::Result;' mod.rs
    fi
fi

# 3. Replace all Result< with anyhow::Result< (but avoid double replacement)
sed -i 's/-> Result</-> anyhow::Result</g' mod.rs
sed -i 's/) -> Result</) -> anyhow::Result</g' mod.rs
sed -i 's/(Result</(anyhow::Result</g' mod.rs

# 4. Fix SessionBuilder - commit_from_file might return Result<Session> or Session
# Remove .context() calls that are on Session (not Result)
# Pattern: .commit_from_file(...)\n                .context(...)?
sed -i '/\.commit_from_file(.*)/{N;s/\n[[:space:]]*\.context(.*)?;//;}' mod.rs || true
# Also handle if commit_from_file returns Result - wrap it properly
# If commit_from_file returns Result, we should use .map_err or just ?
# For now, let's try wrapping the whole chain
sed -i 's/\.commit_from_file(\([^)]*\))\n[[:space:]]*\.context(\([^)]*\))?/\.commit_from_file(\1).map_err(|e| anyhow::anyhow!(e).context(\2))?/' mod.rs || true

# 5. Fix network_quality.rs
if [ -f network_quality.rs ]; then
    sed -i 's/let mut score = 1\.0;/let mut score: f32 = 1.0;/' network_quality.rs
    sed -i 's/score = score\.max(0\.0)\.min(1\.0);/score = score.max(0.0_f32).min(1.0_f32);/' network_quality.rs
fi

# 6. Fix file_size comparisons
sed -i 's/file_size > 100_000_000\.0/file_size > 100_000_000/g' mod.rs
sed -i 's/file_size > 10_000_000\.0/file_size > 10_000_000/g' mod.rs

# 7. Remove unused SessionInputValue import
sed -i 's/SessionInputValue, //' mod.rs || sed -i 's/, SessionInputValue//' mod.rs || true

echo "✅ Comprehensive fixes applied!"

