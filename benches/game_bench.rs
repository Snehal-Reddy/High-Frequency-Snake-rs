use criterion::{Criterion, criterion_group, criterion_main};
use high_frequency_snake::game::{
    engine::GameState,
    types::{Direction, Input},
};
use rand::Rng;
use std::hint::black_box;

const NUM_TICKS: usize = 1000;
const MIN_SNAKES: usize = 100;
const MAX_SNAKES: usize = 1000;
const SNAKE_STEP: usize = 100;

/// Generate random inputs for a given number of snakes
fn generate_random_inputs(num_snakes: usize, input_ratio: f64) -> Vec<Input> {
    let mut rng = rand::rng();
    let num_inputs = (num_snakes as f64 * input_ratio) as usize;
    
    (0..num_inputs)
        .map(|_| Input {
            snake_id: rng.random_range(0..num_snakes) as u32,
            direction: rng.random(),
        })
        .collect()
}

/// Benchmark game.tick() with varying snake counts and no inputs
fn game_tick_no_inputs_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_tick_no_inputs");

    for num_snakes in (MIN_SNAKES..=MAX_SNAKES).step_by(SNAKE_STEP) {
        group.bench_function(&format!("{}_snakes", num_snakes), |b| {
            // Setup outside measurement
            let mut game_state = GameState::random();
            while game_state.snakes.len() < num_snakes {
                game_state = GameState::random();
            }
            
            // Measure only the game.tick() call
            b.iter(|| {
                black_box(game_state.tick(&[]));
            });
        });
    }

    group.finish();
}

/// Benchmark game.tick() with varying snake counts and light input load
fn game_tick_light_inputs_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_tick_light_inputs");

    for num_snakes in (MIN_SNAKES..=MAX_SNAKES).step_by(SNAKE_STEP) {
        group.bench_function(&format!("{}_snakes", num_snakes), |b| {
            // Setup outside measurement
            let mut game_state = GameState::random();
            while game_state.snakes.len() < num_snakes {
                game_state = GameState::random();
            }
            
            // Generate inputs outside measurement
            let inputs = generate_random_inputs(num_snakes, 0.1);
            
            // Measure only the game.tick() call
            b.iter(|| {
                black_box(game_state.tick(&inputs));
            });
        });
    }

    group.finish();
}

/// Benchmark game.tick() with varying snake counts and heavy input load
fn game_tick_heavy_inputs_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_tick_heavy_inputs");

    for num_snakes in (MIN_SNAKES..=MAX_SNAKES).step_by(SNAKE_STEP) {
        group.bench_function(&format!("{}_snakes", num_snakes), |b| {
            // Setup outside measurement
            let mut game_state = GameState::random();
            while game_state.snakes.len() < num_snakes {
                game_state = GameState::random();
            }
            
            // Generate inputs outside measurement
            let inputs = generate_random_inputs(num_snakes, 0.5);
            
            // Measure only the game.tick() call
            b.iter(|| {
                black_box(game_state.tick(&inputs));
            });
        });
    }

    group.finish();
}

/// Benchmark game.tick() with varying snake counts and maximum input load
fn game_tick_max_inputs_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_tick_max_inputs");

    for num_snakes in (MIN_SNAKES..=MAX_SNAKES).step_by(SNAKE_STEP) {
        group.bench_function(&format!("{}_snakes", num_snakes), |b| {
            // Setup outside measurement
            let mut game_state = GameState::random();
            while game_state.snakes.len() < num_snakes {
                game_state = GameState::random();
            }
            
            // Generate inputs outside measurement
            let inputs = generate_random_inputs(num_snakes, 1.0);
            
            // Measure only the game.tick() call
            b.iter(|| {
                black_box(game_state.tick(&inputs));
            });
        });
    }

    group.finish();
}

/// Benchmark single game.tick() call latency (micro-benchmark)
fn game_tick_latency_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_tick_latency");

    for num_snakes in [100, 500, 1000] {
        group.bench_function(&format!("{}_snakes_single_tick", num_snakes), |b| {
            let mut game_state = GameState::random();
            
            // Ensure we have the target number of snakes
            while game_state.snakes.len() < num_snakes {
                game_state = GameState::random();
            }
            
            // Generate inputs for 25% of snakes
            let inputs = generate_random_inputs(num_snakes, 0.25);
            
            b.iter(|| {
                black_box(game_state.tick(&inputs));
            });
        });
    }

    group.finish();
}

/// Benchmark game state initialization performance
fn game_state_init_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_state_init");

    group.bench_function("random_init", |b| {
        b.iter(|| {
            black_box(GameState::random());
        });
    });

    group.bench_function("empty_init", |b| {
        b.iter(|| {
            black_box(GameState::new());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    game_tick_no_inputs_bench,
    game_tick_light_inputs_bench,
    game_tick_heavy_inputs_bench,
    game_tick_max_inputs_bench,
    game_tick_latency_bench,
    game_state_init_bench
);
criterion_main!(benches);
