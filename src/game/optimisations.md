Baseline: tick()
tick() avg time: 10.560 µs

Experiment 1: tick_new()
Hypothesis: Random grid read and write is bad for cache performance

tick() avg time: 15.160 µs (43% slower)
Result: FAILED - Cache-aware implementation is significantly slower

Root Cause Analysis:
- Bucket management overhead: 512 clear() operations per tick
- Record collection overhead: MovementRecord allocation + bit shifting
- Multi-phase processing:
- Spatial bucketing causes cache 
- Tail clearing adds extra loop iteration

Conclusion: FAILURE! Algorithmic overhead > cache benefits. Legacy's simple single-pass approach is more efficient than complex spatial batching.

Experiment 2: TinyDeque
Hypothesis: TinyDeque provides stack-allocated deque with automatic heap spill for better cache locality

tick() avg time: ~6.4 µs (no significant change from VecDeque)
Result: NEUTRAL - No performance improvement detected

Root Cause Analysis:
- Most snakes are short (3-4 segments), fit in 16-element stack allocation
- Snake body access is infrequent (only head/tail), grid access dominates
- TinyDeque provides same API as VecDeque with automatic stack/heap transition
- Cache locality improvement minimal compared to grid access patterns
- Performance is equivalent to VecDeque

Conclusion: NEUTRAL! TinyDeque works correctly but provides no performance benefit. Snake body storage is not the bottleneck. Grid access patterns dominate performance.

