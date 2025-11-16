#!/bin/bash
# Quick demonstration script

echo "🚀 PitlinkPQC Demonstration"
echo "=========================="
echo ""

cd brain

echo "📊 Running Priority Scheduler Example..."
echo ""
cargo run --example priority_scheduler 2>&1 | grep -A 100 "Priority Tagger"

echo ""
echo "✅ Demonstration complete!"
echo ""
echo "For more examples, see:"
echo "  • cargo run --example integrated_workflow"
echo "  • cargo run --example patchy_network_example"
echo "  • cargo run --example priority_scheduler"
