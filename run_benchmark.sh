#!/bin/bash
# Run comprehensive system benchmark

echo "🚀 Running Comprehensive System Benchmark"
echo "=========================================="
echo ""

cd "$(dirname "$0")"

# Run the benchmark
cargo run --example system_benchmark --package trackshift 2>&1

echo ""
echo "✅ Benchmark complete!"
echo ""
echo "📊 Results saved above"
echo "📚 See BENCHMARK_RESULTS.md for detailed analysis"

