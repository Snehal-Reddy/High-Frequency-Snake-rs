#!/bin/bash
# WARNING: This script is deprecated. Use bench/run_bench.sh instead.
# This measures pipeline metrics for setup as well!

# CPU Pipeline Performance Measurement Script
# Usage: ./measure_pipeline.sh <snake_count>

if [ $# -eq 0 ]; then
    echo "Usage: ./measure_pipeline.sh <snake_count>"
    echo "Example: ./measure_pipeline.sh 100"
    exit 1
fi

SNAKE_COUNT=$1
echo "=== Measuring CPU Pipeline Performance for $SNAKE_COUNT snakes ==="

# Run the benchmark with pipeline metrics
perf stat -e cpu-cycles,instructions,branch-instructions,branch-misses,cache-misses,cache-references \
    cargo bench --bench game_bench -- "game_tick_max_inputs/${SNAKE_COUNT}_snakes" 2>&1 | tee "pipeline_results_${SNAKE_COUNT}_snakes.txt"

echo "Results saved to pipeline_results_${SNAKE_COUNT}_snakes.txt"
