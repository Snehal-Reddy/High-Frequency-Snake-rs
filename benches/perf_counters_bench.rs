use criterion::{Criterion, criterion_group, criterion_main, BatchSize};
use high_frequency_snake::game::{
    engine::GameState,
    generator::{DeterministicGenerator, DeterministicConfig},
    types::{Direction, Input},
};
use perf_event_open::config::{Cpu, Opts, Proc};
use perf_event_open::count::Counter;
use perf_event_open::event::hw::Hardware;
use std::hint::black_box;
use std::sync::{Arc, Mutex};

const MIN_SNAKES: usize = 100;
const MAX_SNAKES: usize = 1000;
const SNAKE_STEP: usize = 100;

// Global metrics collector for all benchmark runs
lazy_static::lazy_static! {
    static ref GLOBAL_METRICS: Arc<Mutex<Vec<PerfMetrics>>> = Arc::new(Mutex::new(Vec::new()));
}

/// Performance counter wrapper for measuring hardware events during tick()
struct PerfCounters {
    cache_access: Counter,
    cache_misses: Counter,
    branch_insts: Counter,
    branch_misses: Counter,
    instructions: Counter,
    cycles: Counter,
}

impl PerfCounters {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let target = (Proc::CURRENT, Cpu::ALL);
        let opts = Opts::default();
        
        Ok(Self {
            cache_access: Counter::new(Hardware::CacheAccess, target, opts.clone())?,
            cache_misses: Counter::new(Hardware::CacheMiss, target, opts.clone())?,
            branch_insts: Counter::new(Hardware::BranchInstr, target, opts.clone())?,
            branch_misses: Counter::new(Hardware::BranchMiss, target, opts.clone())?,
            instructions: Counter::new(Hardware::Instr, target, opts.clone())?,
            cycles: Counter::new(Hardware::CpuCycle, target, opts)?,
        })
    }
    
    fn enable(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.cache_access.enable()?;
        self.cache_misses.enable()?;
        self.branch_insts.enable()?;
        self.branch_misses.enable()?;
        self.instructions.enable()?;
        self.cycles.enable()?;
        Ok(())
    }
    
    fn disable(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.cache_access.disable()?;
        self.cache_misses.disable()?;
        self.branch_insts.disable()?;
        self.branch_misses.disable()?;
        self.instructions.disable()?;
        self.cycles.disable()?;
        Ok(())
    }
    
    fn read_metrics(&self) -> Result<PerfMetrics, Box<dyn std::error::Error>> {
        let cache_access = self.cache_access.stat()?.count;
        let cache_misses = self.cache_misses.stat()?.count;
        let branch_insts = self.branch_insts.stat()?.count;
        let branch_misses = self.branch_misses.stat()?.count;
        let instructions = self.instructions.stat()?.count;
        let cycles = self.cycles.stat()?.count;
        
        Ok(PerfMetrics {
            cache_access,
            cache_misses,
            branch_insts,
            branch_misses,
            instructions,
            cycles,
            cache_hit_rate: if cache_access > 0 { 1.0 - (cache_misses as f64 / cache_access as f64) } else { 0.0 },
            branch_prediction_rate: if branch_insts > 0 { 1.0 - (branch_misses as f64 / branch_insts as f64) } else { 0.0 },
            instructions_per_cycle: if cycles > 0 { instructions as f64 / cycles as f64 } else { 0.0 },
        })
    }
}

#[derive(Debug, Clone)]
struct PerfMetrics {
    cache_access: u64,
    cache_misses: u64,
    branch_insts: u64,
    branch_misses: u64,
    instructions: u64,
    cycles: u64,
    cache_hit_rate: f64,
    branch_prediction_rate: f64,
    instructions_per_cycle: f64,
}

/// Generate deterministic inputs for predictable outcomes
/// This is the same function as in game_bench.rs
fn generate_deterministic_inputs(num_snakes: usize, num_ticks: usize) -> Vec<Input> {
    let mut inputs = Vec::new();
    let death_group_size = num_snakes / 4;
    let apple_group_size = num_snakes / 4;
    
    for tick in 0..num_ticks {
        for snake_id in 0..num_snakes as u32 {
            let direction = match snake_id {
                // Death group: converging movement
                id if (id as usize) < death_group_size => {
                    if id % 2 == 0 {
                        Direction::Right
                    } else {
                        Direction::Left
                    }
                },
                // Apple group: zigzag search patterns
                id if (id as usize) < death_group_size + apple_group_size => {
                    match tick % 4 {
                        0 => Direction::Right,
                        1 => Direction::Down,
                        2 => Direction::Left,
                        _ => Direction::Up,
                    }
                },
                // Safe group: linear movement
                _ => {
                    if tick % 10 == 9 {
                        Direction::Down
                    } else {
                        Direction::Right
                    }
                },
            };
            
            inputs.push(Input { snake_id, direction });
        }
    }
    
    inputs
}

/// Benchmark performance counters during tick() execution
/// This measures the same hot path as hot_path_bench but with hardware performance counters
fn perf_counters_bench(c: &mut Criterion) {
    // Clear global metrics at the start
    {
        let mut global = GLOBAL_METRICS.lock().unwrap();
        global.clear();
    }
    
    let mut group = c.benchmark_group("perf_counters");
    
    for num_snakes in (MIN_SNAKES..=MAX_SNAKES).step_by(SNAKE_STEP) {
        group.bench_function(&format!("{}_snakes", num_snakes), |b| {
            // Collect metrics for averaging within this run
            let mut all_metrics = Vec::new();
            // Generate deterministic inputs outside measurement (same as hot_path_bench)
            let inputs = generate_deterministic_inputs(num_snakes, 1);
            
            // Use iter_batched_ref for expensive setup costs (same as hot_path_bench)
            b.iter_batched_ref(
                || {
                    let config = DeterministicConfig::default();
                    DeterministicGenerator::generate_predictable_outcomes(num_snakes, config)
                },
                |game_state| {
                    // Create performance counters
                    let counters = PerfCounters::new().expect("Failed to create perf counters");
                    
                    // CACHE WARMUP: Execute tick() once without measuring to warm up caches
                    // This helps ensure more consistent cache performance across iterations
                    black_box(game_state.tick(&inputs));
                    
                    // Enable counters for the actual measurement
                    counters.enable().expect("Failed to enable counters");
                    
                    // Execute tick() - THIS IS THE ONLY THING BEING MEASURED
                    // Same as hot_path_bench: black_box(game_state.tick(&inputs));
                    black_box(game_state.tick(&inputs));
                    
                    // Disable counters
                    counters.disable().expect("Failed to disable counters");
                    
                    // Read metrics (this happens outside the measured section)
                    let metrics = counters.read_metrics().expect("Failed to read metrics");
                    
                    // Store metrics for averaging
                    all_metrics.push(metrics.clone());
                    
                    // Also store in global metrics
                    {
                        let mut global = GLOBAL_METRICS.lock().unwrap();
                        global.push(metrics.clone());
                    }
                    
                    // Print individual iteration metrics
                    println!("Snakes: {}, Cache Hit Rate: {:.4}% ({} hits, {} misses), Branch Prediction: {:.4}%, IPC: {:.4}", 
                             num_snakes, 
                             metrics.cache_hit_rate * 100.0,
                             metrics.cache_access - metrics.cache_misses,
                             metrics.cache_misses,
                             metrics.branch_prediction_rate * 100.0, 
                             metrics.instructions_per_cycle);
                    
                    // Return metrics for potential further analysis
                    black_box(metrics);
                },
                BatchSize::LargeInput,
            );
            
        });
    }
    
    group.finish();
    
    // Print global summary across all runs
    {
        let global = GLOBAL_METRICS.lock().unwrap();
        if !global.is_empty() {
            let total_iterations = global.len();
            let avg_cache_hit_rate = global.iter().map(|m| m.cache_hit_rate).sum::<f64>() / total_iterations as f64;
            let avg_branch_prediction = global.iter().map(|m| m.branch_prediction_rate).sum::<f64>() / total_iterations as f64;
            let avg_ipc = global.iter().map(|m| m.instructions_per_cycle).sum::<f64>() / total_iterations as f64;
            let total_cache_access = global.iter().map(|m| m.cache_access).sum::<u64>();
            let total_cache_misses = global.iter().map(|m| m.cache_misses).sum::<u64>();
            let total_cache_hits = total_cache_access - total_cache_misses;
            
            let separator = "=".repeat(80);
            println!("\n{}", separator);
            println!("🎯 GLOBAL AVERAGE RESULTS ACROSS ALL BENCHMARK RUNS");
            println!("{}", separator);
            println!("Average Cache Hit Rate: {:.4}% ({} total hits, {} total misses)", 
                     avg_cache_hit_rate * 100.0, total_cache_hits, total_cache_misses);
            println!("Average Branch Prediction Rate: {:.4}%", avg_branch_prediction * 100.0);
            println!("Average Instructions Per Cycle: {:.4}", avg_ipc);
            println!("Total Iterations Across All Runs: {}", total_iterations);
            println!("{}", separator);
        }
    }
}

criterion_group!(benches, perf_counters_bench);
criterion_main!(benches);
