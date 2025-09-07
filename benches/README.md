# Performance Benchmarks

This directory contains performance benchmarks for the high-frequency-snake game engine.

## Available Benchmarks

### 1. `game_bench.rs` - Timing Benchmarks
- **Purpose**: Measures execution time of the `tick()` function
- **Key benchmark**: `hot_path_bench` - isolates just the `tick()` timing
- **Usage**: `cargo bench --bench game_bench`

### 2. `perf_counters_bench.rs` - Hardware Performance Counters
- **Purpose**: Measures cache hit rate, branch prediction, and IPC for `tick()`
- **Features**: 
  - Cache warmup for consistent results
  - Global averaging across all benchmark runs
  - Hardware performance counters via `perf-event-open`
- **Usage**: `cargo bench --bench perf_counters_bench`

### 3. `perf_summary.py` - Comprehensive Performance Analysis
- **Purpose**: Runs multiple measurements across different snake counts and aggregates results
- **Features**:
  - Multiple runs per snake count for statistical reliability
  - Standard deviation calculations
  - JSON output for further analysis
  - Clean tabular output
- **Usage**: `python3 perf_summary.py`

## Quick Start

### Single Benchmark Run
```bash
# Time-based benchmark
cargo bench --bench game_bench

# Performance counters benchmark
cargo bench --bench perf_counters_bench

# Specific snake count
cargo bench --bench perf_counters_bench perf_counters/100_snakes
```

### Comprehensive Analysis
```bash
# Run full analysis across all snake counts
python3 perf_summary.py
```

### Using the Benchmark Script
```bash
# Run with performance counters
./run_bench.sh --perf-counters --snakes=100

# Run timing benchmark
./run_bench.sh --snakes=100
```

## Output Files

- `perf_summary.json`: Aggregated performance metrics across snake counts
- Benchmark reports in `target/criterion/`: Detailed timing analysis

## Key Metrics Measured

### Cache Performance
- **Cache Hit Rate**: Percentage of cache accesses that hit
- **Cache Hits/Misses**: Absolute counts for context
- **Cache Access Patterns**: How well the code utilizes CPU caches

### Branch Prediction
- **Branch Prediction Rate**: How often the CPU correctly predicts branches
- **Branch Misses**: When the CPU's branch predictor is wrong

### Instruction Efficiency
- **Instructions Per Cycle (IPC)**: How many instructions execute per CPU cycle
- **Pipeline Utilization**: How efficiently the CPU pipeline is used

## Technical Details

### Cache Warmup
The `perf_counters_bench` includes a cache warmup phase to ensure consistent measurements:
- Runs `tick()` 3 times before measurement
- Ensures CPU caches are populated with working set
- Eliminates cold start effects for reliable results

### Deterministic Inputs
All benchmarks use `DeterministicGenerator` to ensure:
- Consistent game states across runs
- Reproducible performance characteristics
- Fair comparison between different snake counts

### Hardware Performance Counters
Uses the `perf-event-open` crate to access Linux performance counters:
- `Hardware::CacheAccess`: Total cache accesses
- `Hardware::CacheMiss`: Cache misses
- `Hardware::BranchInstr`: Branch instructions
- `Hardware::BranchMiss`: Branch mispredictions
- `Hardware::CpuCycle`: CPU cycles
- `Hardware::Instructions`: Instructions executed
