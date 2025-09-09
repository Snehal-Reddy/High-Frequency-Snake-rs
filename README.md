# High-Frequency Snake: Low Level Optimization Playground

A high-frequency game server optimization project using a massive multiplayer snake battle royale as the test scenario.


## Current Status: Active Development

This is an active project focused on learning low-level Rust optimizations. The core infrastructure is in place with comprehensive benchmarking, profiling capabilities, and performance optimizations including cache-aware data structures and branch prediction improvements.

## Project Goal

Learn and experiment with low-level optimization techniques in Rust applying them to a game server that pushes tick rates to extreme limits.

**Target Performance**: 200,000+ ticks per second (5 microseconds per tick) with 1000+ concurrent snakes.

## Game Concept

**Massive Multiplayer Snake Battle Royale**
- 1000+ snakes moving simultaneously on a shared 2D grid
- Each snake moves 1 unit in any direction per tick
- Snake collisions, food consumption, and territory conflicts
- Real-time performance constraints with sub-millisecond tick processing

## Architecture

The application runs as a single process with two dedicated threads pinned to separate CPU cores with a custom minimal Lock Free Queue for IPC:

```
+--------------------------------------------------------------------------+
|                      Game Server Process (main.rs)                       |
|                                                                          |
|  +-----------------------+        +----------------------------------+   |
|  | Input Generator       |------->| Game Logic                       |   |
|  | (Thread 1, Core A)    | SPSC   | (Thread 2, Core B)               |   |
|  |                       | Queue  |                                  |   |
|  | - Generates random    |        | - Polls queue for inputs         |   |
|  |   (id, dir) inputs    |        | - Updates snake directions       |   |
|  | - Pushes to queue     |        | - Executes game tick()           |   |
|  +-----------------------+        +----------------------------------+   |
|                                                                          |
+--------------------------------------------------------------------------+
```

## What's Implemented

### Core Game Logic
- ✅ **Game Engine**: Complete snake movement, collision detection, and state management
- ✅ **Grid System**: 10000×10000 cell grid with efficient spatial queries
- ✅ **Snake Logic**: Movement, growth, collision detection, and lifecycle management
- ✅ **Apple System**: Food spawning and consumption mechanics

### No BS Infrastructure
- ✅ **SPSC Queue**: Lock-free single-producer, single-consumer queue for inter-thread communication
- ✅ **CPU Pinning**: Thread affinity to specific CPU cores for cache locality

### Performance Measurement
- ✅ **Benchmark Suite**: Comprehensive performance testing with Criterion.rs
  - SPSC queue throughput and latency benchmarks
  - Game logic performance benchmarks
  - Integrated hot path measurements
  - Hardware performance counter benchmarks
- ✅ **Real-time Profiling**: CPU cycle-accurate measurements using `_rdtsc()`
- ✅ **Performance Metrics**: Ticks per second, hot path latency, consume vs tick breakdown
- ✅ **Cache Performance Analysis**: cache hit rates and memory access patterns
- ✅ **Branch Prediction Analysis**: Branch misprediction rates and optimization opportunities

### Benchmark Categories
- **SPSC Benchmarks**: Queue throughput, latency, and contention testing
- **Game Benchmarks**: Pure game logic performance with varying snake counts and input loads
- **Integrated Benchmarks**: Complete hot path measurement (consume + tick) with pinned threads
- **Performance Counter Benchmarks**: Hardware-level cache hit rates, branch prediction, and IPC measurements
- **Comprehensive Analysis**: Multi-run statistical analysis across snake counts with JSON output

## Performance Constraints

1. **Non-parallelizable core logic**: Snake collisions and food consumption must be resolved sequentially
2. **Hot path critical**: Movement, collision detection, and state updates happen every tick
3. **Memory access patterns**: Random grid access creates cache pressure
4. **Real-time requirements**: Sub-millisecond tick processing

## Getting Started

### Prerequisites
- Rust 1.70+ 
- At least 2 CPU cores (for thread pinning)

### Building and Running

**Normal optimized run:**
```bash
cargo run --release
```

**With profiling enabled:**
```bash
cargo run --profile profile --features profile
```

**Run benchmarks:**
```bash
# SPSC queue performance
cargo bench --bench spsc_bench

# Game logic performance
cargo bench --bench game_bench

# Integrated hot path performance
cargo bench --bench integrated_bench

# Hardware performance counters (cache hit rate, branch prediction, IPC)
cargo bench --bench perf_counters_bench

# Comprehensive performance analysis across snake counts
cd benches && python3 perf_summary.py
```

### Profiling Output

When running with profiling enabled, you'll see real-time performance metrics:
```
Snake Battle Royale: Low Level Optimization Playground
Input generator thread started on core 0
Game logic thread started on core 1
Tick 1000: 10254.46 ticks/sec | Consume: avg=24203 cycles, min=13462 cycles, max=312781 cycles | Tick: avg=353914 cycles, min=290931 cycles, max=6333712 cycles
Tick 2000: 10653.39 ticks/sec | Consume: avg=24056 cycles, min=4450 cycles, max=312781 cycles | Tick: avg=339884 cycles, min=258011 cycles, max=6333712 cycles
```

### Performance Analysis Tools

**Hardware Performance Counters:**
```bash
# Single snake count analysis
cargo bench --bench perf_counters_bench perf_counters/100_snakes

# All snake counts with statistical analysis
cd benches && python3 perf_summary.py
```

**Legacy Performance Tools (perf directory):**
```bash
# Cache performance measurement
./perf/measure_cache.sh

# Pipeline performance analysis  
./perf/measure_pipeline.sh
```

**Note**: The new `perf_counters_bench` and `perf_summary.py` provide more integrated and reliable performance analysis. See `benches/README.md` for detailed usage.

## Project Structure

```
src/
├── main.rs              # Application entry point with thread setup
├── lib.rs               # Library root
├── game/                # Core game logic
│   ├── engine.rs        # Game state and main tick loop (vector-based)
│   ├── grid.rs          # 2D grid with spatial queries (10000×10000)
│   ├── snake.rs         # Snake movement and lifecycle
│   ├── apple.rs         # Food spawning and consumption
│   ├── generator.rs     # Deterministic and random game state generation
│   └── types.rs         # Game data structures
├── ipc/                 # Inter-process communication
│   └── spsc.rs          # Lock-free SPSC queue implementation
└── tests.rs             # Comprehensive unit tests

benches/                 # Performance benchmarks
├── spsc_bench.rs        # SPSC queue performance tests
├── game_bench.rs        # Game logic performance tests
├── integrated_bench.rs  # Complete hot path measurements
├── perf_counters_bench.rs # Hardware performance counter benchmarks
├── perf_summary.py      # Comprehensive performance analysis script
├── run_bench.sh         # Benchmark runner with various options
└── README.md            # Detailed benchmark documentation

perf/                    # Legacy performance analysis tools
├── measure_cache.sh     # Cache performance measurement scripts
├── measure_pipeline.sh  # Pipeline performance analysis
└── perf_summary.json    # Performance metrics database
```
