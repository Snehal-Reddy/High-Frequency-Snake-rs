# Snake Game Benchmark Requirements & Plan

## Requirements

### 1. Performance Measurement Requirements
- **Primary Goal**: Measure hot path performance of `tick()` function in `engine.rs`
- **Metric**: Average tick time across multiple ticks
- **Scope**: Focus on game state evolution performance, not initialization

### 2. Consistency Requirements
- **Deterministic Game State**: Same starting positions every run
- **Deterministic Inputs**: Same movement patterns every run
- **Deterministic Outcomes**: Same number of deaths, apple consumption, and safe movements every run
- **No Randomization**: All randomness must be seeded and reproducible

### 3. Benchmark Structure Requirements
- **Fixed Snake Count**: 1000 snakes (scalable from smaller test cases)
- **Fixed Tick Count**: Predefined number of ticks (configurable)
- **Predictable Ratios**: Exact percentages for different snake behaviors
- **Reproducible Results**: Same performance measurements across runs

### 4. Game Logic Requirements
- **Edge Wrapping**: Snakes wrap around grid boundaries (don't die from walls)
- **Collision Detection**: Snakes die when hitting other snakes
- **Apple Consumption**: Snakes grow when eating apples
- **Grid Management**: Proper collision detection and state updates

### 5. Validation Requirements
- **Pre-calculation**: All positions, movements, and outcomes must be calculated before coding
- **Dry Run Verification**: Mathematical proof that movement patterns achieve intended outcomes
- **Outcome Verification**: Ability to count and verify actual results match expectations

## Final Plan

### What We Are Simulating
We are creating a **predictable game scenario** that simulates real snake game events over a fixed number of ticks:

1. **Kill 25% of snakes** through intentional collisions
2. **Grow 25% of snakes** through apple consumption and movement patterns
3. **Keep 50% of snakes unchanged** through safe movement patterns

### How We Achieve This
- **Death Group (25%)**: Snakes move in converging patterns that cause collisions
- **Apple Group (25%)**: Snakes move in expanding patterns that cover more grid area and hit apples
- **Safe Group (50%)**: Snakes move in safe patterns that avoid collisions and wrap around edges

### Expected Outcomes
- **Consistent Results**: Same number of dead/grown/safe snakes every run
- **Performance Data**: Reliable measurements for optimization work
- **Scalable Design**: Can scale from small test cases to full 1000-snake scenarios

## Configuration

### Test Case: 40x40 Grid with 10 Snakes
- **Grid Size**: 40x40 = 1,600 cells
- **Snake Count**: 10 snakes
- **Snake Length**: 3 segments each
- **Total Occupied**: 30 cells

### Snake Groups
- **Death Group (25%)**: 2-3 snakes (snakes 0,1)
- **Apple Group (25%)**: 2-3 snakes (snakes 2,3)  
- **Safe Group (50%)**: 4-5 snakes (snakes 4,5,6,7,8,9)

### Placement Strategy
- **Row 0**: Snakes 0,1,2,3 (every 10 cells)
- **Row 10**: Snakes 4,5,6 (every 10 cells)
- **Row 20**: Snakes 7,8,9 (every 10 cells)

### Movement Patterns
- **Death Group**: Converging movement (snakes 0,1 move toward each other)
- **Apple Group**: Expanding movement (snakes 2,3 move to cover more grid area)
- **Safe Group**: Linear movement (snakes 4-9 move in safe patterns)

### Expected Outcomes
- **Collision**: Snakes 0,1 collide at tick 8 at position (10,0)
- **Apple Consumption**: Snakes 2,3 move extensively, increasing apple hit probability
- **Safe Movement**: Snakes 4-9 move without collisions, wrapping around edges

### Scaling to 1000 Snakes
- **Grid Size**: 1000x1000
- **Snake Count**: 1000 snakes
- **Group Sizes**: 250 death, 250 apple, 500 safe
- **Placement**: Grid pattern with appropriate spacing
- **Movement**: Same logic scaled to larger groups

## Implementation Notes

### Key Principles
- **Deterministic Generation**: Use fixed seeds for all random number generation
- **Pre-calculated Inputs**: Generate all movement patterns before running benchmarks
- **Validation**: Verify outcomes match expectations before using for performance measurement
- **Scalability**: Design patterns that work at both small and large scales

### Success Criteria
- [ ] Same game state generated every run
- [ ] Same inputs generated every run  
- [ ] Same outcomes (deaths, growth, survival) every run
- [ ] Performance measurements are consistent across runs
- [ ] Benchmark actually measures hot path performance, not initialization

This configuration provides a deterministic, predictable benchmark that measures the actual performance of game state evolution while maintaining consistency across runs.