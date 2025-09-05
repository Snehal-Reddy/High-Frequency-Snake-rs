# High-Frequency Snake: Low Level Optimization Playground

A high-frequency game server optimization project using a massive multiplayer snake battle royale as the test scenario.


## Current Status: Active Development

This is an active project focused on learning low-level Rust optimizations. The core infrastructure is in place with comprehensive benchmarking, profiling capabilities, and performance optimizations including cache-aware data structures and branch prediction improvements.

## Project Goal

Learn and experiment with low-level optimization techniques in Rust applying them to a game server that pushes tick rates to extreme limits.

**Target Performance**: 20,000+ ticks per second (0.05ms per tick) with 1000+ concurrent snakes.

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
- ✅ **Grid System**: 4000×4000 cell grid with efficient spatial queries
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
- ✅ **Real-time Profiling**: CPU cycle-accurate measurements using `_rdtsc()`
- ✅ **Performance Metrics**: Ticks per second, hot path latency, consume vs tick breakdown
- ✅ **Cache Performance Analysis**: L1/L2/L3 cache hit rates and memory access patterns
- ✅ **Branch Prediction Analysis**: Branch misprediction rates and optimization opportunities

### Benchmark Categories
- **SPSC Benchmarks**: Queue throughput, latency, and contention testing
- **Game Benchmarks**: Pure game logic performance with varying snake counts and input loads
- **Integrated Benchmarks**: Complete hot path measurement (consume + tick) with pinned threads
- **Cache Benchmarks**: Memory access pattern analysis and cache efficiency measurements
- **Branch Prediction Benchmarks**: Conditional branch performance and optimization validation

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

**Cache Performance Measurement:**
```bash
./perf/measure_cache.sh
```

**Pipeline Performance Analysis:**
```bash
./perf/measure_pipeline.sh
```

**Comprehensive Performance Report:**
```bash
python3 perf/perf_all_summary.py
```

## Project Structure

```
src/
├── main.rs              # Application entry point with thread setup
├── lib.rs               # Library root
├── game/                # Core game logic
│   ├── engine.rs        # Game state and main tick loop (vector-based)
│   ├── grid.rs          # 2D grid with spatial queries (4000×4000)
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
└── integrated_bench.rs  # Complete hot path measurements

perf/                    # Performance analysis tools
├── debug_perf.py        # Performance debugging utilities
├── measure_cache.sh     # Cache performance measurement scripts
├── measure_pipeline.sh  # Pipeline performance analysis
├── perf_all_summary.py  # Comprehensive performance reporting
└── perf_summary.json    # Performance metrics database
```


**Next Steps:**
- Performance optimization based on benchmark results
- SIMD optimizations for game logic
- Memory layout improvements
- Cache-aware data structure redesign
- Branch prediction optimization
