#!/bin/bash
# WARNING: This script is deprecated. Use bench/run_bench.sh instead.
# This measures cache metrics for setup as well!

# Cache Performance Measurement Script
# Usage: ./measure_cache.sh <snake_count>

if [ $# -eq 0 ]; then
    echo "Usage: ./measure_cache.sh <snake_count>"
    echo "Example: ./measure_cache.sh 100"
    exit 1
fi

SNAKE_COUNT=$1
echo "=== Measuring Cache Performance for $SNAKE_COUNT snakes ==="

# Run the benchmark with cache metrics
perf stat -e cache-misses,cache-references,cpu-cycles,instructions,branch-instructions,branch-misses \
    cargo bench --bench game_bench -- "game_tick_max_inputs/${SNAKE_COUNT}_snakes" 2>&1 | tee "cache_results_${SNAKE_COUNT}_snakes.txt"

echo "Results saved to cache_results_${SNAKE_COUNT}_snakes.txt"
