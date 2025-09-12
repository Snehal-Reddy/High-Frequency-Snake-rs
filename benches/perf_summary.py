#!/usr/bin/env python3
"""
Measure cache and branch behavior across snake counts using perf_counters_bench.
- Runs: cargo bench --bench perf_counters_bench -- "perf_counters/{N}_snakes"
- Uses our deterministic benchmark with consistent CPU frequency and cache state
- Parses stdout to extract global average results
- Does multiple runs per snake count and averages results
- Prints a compact table and writes JSON summary to perf_summary.json

Usage:
  python3 perf_summary.py           # Run for all snake counts [100, 300, 500, 700, 900, 1000]
  python3 perf_summary.py 1000     # Run only for 1000 snakes
"""

import subprocess
import json
import re
import statistics
import sys
from typing import Dict, Any, List

DEFAULT_SNAKE_COUNTS = [100, 300, 500, 700, 900, 1000]
RUNS_PER_SNAKE_COUNT = 3  # Number of runs per snake count

# Column definitions: (header, width)
COLUMNS = [
    ("Snakes", 6),
    ("CacheHit%", 10),
    ("CacheHits", 12),
    ("CacheMisses", 12),
    ("BrPred%", 9),
    ("IPC", 6),
    ("Iterations", 10),
]


def run_perf_bench_for_snakes(snake_count: int) -> Dict[str, Any]:
    """Run the perf_counters_bench for a specific snake count and parse results"""
    cmd = [
        "cargo", "bench", "--bench", "perf_counters_bench", 
        f"perf_counters/{snake_count}_snakes"
    ]

    proc = subprocess.run(cmd, capture_output=True, text=True)
    
    if proc.returncode != 0:
        print(f"    ✗ Benchmark failed for {snake_count} snakes")
        return None

    # Parse the global average results from stdout
    stdout = proc.stdout or ""
    
    # Look for the global average section
    global_avg_match = re.search(
        r"🎯 GLOBAL AVERAGE RESULTS ACROSS ALL BENCHMARK RUNS.*?"
        r"Average Cache Hit Rate: ([\d.]+)% \(([\d,]+) total hits, ([\d,]+) total misses\).*?"
        r"Average Branch Prediction Rate: ([\d.]+)%.*?"
        r"Average Instructions Per Cycle: ([\d.]+).*?"
        r"Total Iterations Across All Runs: ([\d,]+)",
        stdout, re.DOTALL
    )
    
    if not global_avg_match:
        print(f"    ✗ Could not parse results for {snake_count} snakes")
        return None
    
    cache_hit_rate = float(global_avg_match.group(1))
    total_cache_hits = int(global_avg_match.group(2).replace(",", ""))
    total_cache_misses = int(global_avg_match.group(3).replace(",", ""))
    branch_prediction_rate = float(global_avg_match.group(4))
    ipc = float(global_avg_match.group(5))
    total_iterations = int(global_avg_match.group(6).replace(",", ""))
    
    return {
        "snakes": snake_count,
        "cache_hit_rate_percent": cache_hit_rate,
        "total_cache_hits": total_cache_hits,
        "total_cache_misses": total_cache_misses,
        "branch_prediction_rate_percent": branch_prediction_rate,
        "instructions_per_cycle": ipc,
        "total_iterations": total_iterations,
    }


def run_multiple_perf_measurements(snake_count: int, runs: int = RUNS_PER_SNAKE_COUNT) -> Dict[str, Any]:
    """Run multiple perf measurements and return averaged results"""
    print(f"  Running {runs} measurements for {snake_count} snakes...")
    
    results = []
    for run_num in range(runs):
        print(f"    Run {run_num + 1}/{runs}...", end=" ", flush=True)
        
        try:
            result = run_perf_bench_for_snakes(snake_count)
            if result:
                results.append(result)
                print("✓")
            else:
                print("✗")
        except Exception as e:
            print(f"✗ Error: {e}")
            continue
    
    if not results:
        print(f"    No successful runs for {snake_count} snakes")
        return {
            "snakes": snake_count,
            "cache_hit_rate_percent": 0.0,
            "total_cache_hits": 0,
            "total_cache_misses": 0,
            "branch_prediction_rate_percent": 0.0,
            "instructions_per_cycle": 0.0,
            "total_iterations": 0,
            "runs_completed": 0,
            "std_dev_cache_hit_rate": 0.0,
            "std_dev_branch_prediction": 0.0,
            "std_dev_ipc": 0.0,
        }
    
    # Calculate averages
    avg_result = {
        "snakes": snake_count,
        "cache_hit_rate_percent": statistics.mean(r["cache_hit_rate_percent"] for r in results),
        "total_cache_hits": int(statistics.mean(r["total_cache_hits"] for r in results)),
        "total_cache_misses": int(statistics.mean(r["total_cache_misses"] for r in results)),
        "branch_prediction_rate_percent": statistics.mean(r["branch_prediction_rate_percent"] for r in results),
        "instructions_per_cycle": statistics.mean(r["instructions_per_cycle"] for r in results),
        "total_iterations": int(statistics.mean(r["total_iterations"] for r in results)),
        "runs_completed": len(results),
    }
    
    # Calculate standard deviations if we have multiple runs
    if len(results) > 1:
        avg_result["std_dev_cache_hit_rate"] = statistics.stdev(r["cache_hit_rate_percent"] for r in results)
        avg_result["std_dev_branch_prediction"] = statistics.stdev(r["branch_prediction_rate_percent"] for r in results)
        avg_result["std_dev_ipc"] = statistics.stdev(r["instructions_per_cycle"] for r in results)
    else:
        avg_result["std_dev_cache_hit_rate"] = 0.0
        avg_result["std_dev_branch_prediction"] = 0.0
        avg_result["std_dev_ipc"] = 0.0
    
    return avg_result


def _fmt_header() -> str:
    parts: List[str] = []
    for header, width in COLUMNS:
        parts.append(f"{header:>{width}}")
    return " ".join(parts)


def _fmt_row(res: Dict[str, Any]) -> str:
    values = [
        f"{res['snakes']:>{COLUMNS[0][1]}d}",
        f"{res['cache_hit_rate_percent']:>{COLUMNS[1][1]-1}.2f}%",
        f"{res['total_cache_hits']:>{COLUMNS[2][1]},}",
        f"{res['total_cache_misses']:>{COLUMNS[3][1]},}",
        f"{res['branch_prediction_rate_percent']:>{COLUMNS[4][1]-1}.2f}%",
        f"{res['instructions_per_cycle']:>{COLUMNS[5][1]-1}.2f}",
        f"{res['total_iterations']:>{COLUMNS[6][1]},}",
    ]
    return " ".join(values)


def main() -> None:
    # Parse command line arguments
    if len(sys.argv) > 1:
        try:
            snake_count = int(sys.argv[1])
            snake_counts = [snake_count]
            print(f"=== Measuring cache and branch metrics for {snake_count} snakes ===")
        except ValueError:
            print(f"Error: '{sys.argv[1]}' is not a valid number")
            print("Usage: python3 perf_summary.py [snake_count]")
            print("  If no snake_count provided, runs for all counts: [100, 300, 500, 700, 900, 1000]")
            sys.exit(1)
    else:
        snake_counts = DEFAULT_SNAKE_COUNTS
        print("=== Measuring cache and branch metrics across snake counts ===")
    
    print("Using perf_counters_bench with deterministic hot path and cache warmup")
    print(f"Running {RUNS_PER_SNAKE_COUNT} measurements per snake count for reliability")
    print()
    
    results: List[Dict[str, Any]] = []

    # Header
    header = _fmt_header()
    print(header)
    print("-" * len(header))

    for n in snake_counts:
        print(f"\nMeasuring {n} snakes:")
        res = run_multiple_perf_measurements(n, RUNS_PER_SNAKE_COUNT)
        results.append(res)
        print(f"  Average results: {_fmt_row(res)}")
        if res["runs_completed"] > 1:
            print(f"  Std dev - Cache hit rate: {res['std_dev_cache_hit_rate']:.2f}%, "
                  f"Branch prediction: {res['std_dev_branch_prediction']:.2f}%, "
                  f"IPC: {res['std_dev_ipc']:.2f}")

    # Save JSON summary
    out_file = "perf_summary.json"
    with open(out_file, "w") as f:
        json.dump({
            "metadata": {
                "runs_per_snake_count": RUNS_PER_SNAKE_COUNT,
                "snake_counts": snake_counts,
                "benchmark": "perf_counters_bench",
                "description": "Deterministic hot path benchmark with cache warmup and hardware performance counters"
            },
            "results": results
        }, f, indent=2)

    print(f"\nSaved summary to: {out_file}")
    print(f"Each snake count was measured {RUNS_PER_SNAKE_COUNT} times and averaged")


if __name__ == "__main__":
    main()
