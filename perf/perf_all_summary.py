#!/usr/bin/env python3
"""
Measure cache and branch behavior across snake counts using deterministic hot path benchmark.
- Runs: perf stat -e cache-misses,cache-references,branch-instructions,branch-misses
- Bench target: cargo bench --bench game_bench -- "hot_path/{N}_snakes"
- Uses our deterministic benchmark with consistent CPU frequency and cache state
- Parses stderr (perf output), sums across cpu_atom/cpu_core, computes miss rates
- Does multiple runs per snake count and averages results
- Prints a compact table and writes JSON summary to perf_summary.json
"""

import subprocess
import json
import re
import statistics
from typing import Dict, Any, List

SNAKE_COUNTS = [100, 300, 500, 700, 900, 1000]
PERF_EVENTS = "cache-misses,cache-references,branch-instructions,branch-misses"
RUNS_PER_SNAKE_COUNT = 5  # Number of runs per snake count

# Column definitions: (header, width)
COLUMNS = [
    ("Snakes", 6),
    ("CacheMisses", 13),
    ("CacheRefs", 13),
    ("CMiss%", 8),
    ("BrMisses", 12),
    ("Branches", 12),
    ("BMiss%", 8),
]


def run_perf_for_snakes(snake_count: int) -> Dict[str, Any]:
    # Use our consistent benchmarking script that locks CPU frequency and handles cache state
    cmd = [
        "perf", "stat", "-e", PERF_EVENTS,
        "./benches/run_bench.sh", str(snake_count)
    ]

    proc = subprocess.run(cmd, capture_output=True, text=True)

    # Perf writes stats to stderr
    stderr = proc.stderr or ""

    # Aggregate metrics across cpu_atom/cpu_core entries
    total_cache_misses = 0
    total_cache_refs = 0
    total_branch_misses = 0
    total_branch_instructions = 0

    # Example lines to match:
    #   21,054,887      cpu_atom/cache-misses/           #   45.22% of all cache refs           (17.97%)
    #   925,494,307     cpu_core/cache-references/
    metric_re = re.compile(r"^\s*([\d,]+)\s+([^\s#]+)")

    for line in stderr.splitlines():
        m = metric_re.match(line)
        if not m:
            continue
        value_str, metric = m.group(1), m.group(2)
        try:
            value = int(value_str.replace(",", ""))
        except ValueError:
            continue

        if "cache-misses" in metric:
            total_cache_misses += value
        elif "cache-references" in metric:
            total_cache_refs += value
        elif "branch-misses" in metric:
            total_branch_misses += value
        elif "branch-instructions" in metric:
            total_branch_instructions += value

    # Extract timings (optional but useful)
    time_elapsed = None
    m_time = re.search(r"(\d+\.\d+) seconds time elapsed", stderr)
    if m_time:
        time_elapsed = float(m_time.group(1))

    cache_miss_rate = (total_cache_misses / total_cache_refs * 100.0) if total_cache_refs > 0 else 0.0
    branch_miss_rate = (total_branch_misses / total_branch_instructions * 100.0) if total_branch_instructions > 0 else 0.0

    return {
        "snakes": snake_count,
        "cache_misses": total_cache_misses,
        "cache_references": total_cache_refs,
        "cache_miss_rate_percent": cache_miss_rate,
        "branch_misses": total_branch_misses,
        "branch_instructions": total_branch_instructions,
        "branch_miss_rate_percent": branch_miss_rate,
        "time_elapsed_seconds": time_elapsed,
    }


def run_multiple_perf_measurements(snake_count: int, runs: int = RUNS_PER_SNAKE_COUNT) -> Dict[str, Any]:
    """Run multiple perf measurements and return averaged results"""
    print(f"  Running {runs} measurements for {snake_count} snakes...")
    
    results = []
    for run_num in range(runs):
        print(f"    Run {run_num + 1}/{runs}...", end=" ", flush=True)
        
        try:
            result = run_perf_for_snakes(snake_count)
            results.append(result)
            print("✓")
        except Exception as e:
            print(f"✗ Error: {e}")
            continue
    
    if not results:
        print(f"    No successful runs for {snake_count} snakes")
        return {
            "snakes": snake_count,
            "cache_misses": 0,
            "cache_references": 0,
            "cache_miss_rate_percent": 0.0,
            "branch_misses": 0,
            "branch_instructions": 0,
            "branch_miss_rate_percent": 0.0,
            "time_elapsed_seconds": 0.0,
            "runs_completed": 0,
            "std_dev_cache_misses": 0.0,
            "std_dev_branch_misses": 0.0,
        }
    
    # Calculate averages
    avg_result = {
        "snakes": snake_count,
        "cache_misses": int(statistics.mean(r["cache_misses"] for r in results)),
        "cache_references": int(statistics.mean(r["cache_references"] for r in results)),
        "cache_miss_rate_percent": statistics.mean(r["cache_miss_rate_percent"] for r in results),
        "branch_misses": int(statistics.mean(r["branch_misses"] for r in results)),
        "branch_instructions": int(statistics.mean(r["branch_instructions"] for r in results)),
        "branch_miss_rate_percent": statistics.mean(r["branch_miss_rate_percent"] for r in results),
        "time_elapsed_seconds": statistics.mean(r["time_elapsed_seconds"] for r in results) if results[0]["time_elapsed_seconds"] else None,
        "runs_completed": len(results),
    }
    
    # Calculate standard deviations if we have multiple runs
    if len(results) > 1:
        avg_result["std_dev_cache_misses"] = statistics.stdev(r["cache_misses"] for r in results)
        avg_result["std_dev_branch_misses"] = statistics.stdev(r["branch_misses"] for r in results)
    else:
        avg_result["std_dev_cache_misses"] = 0.0
        avg_result["std_dev_branch_misses"] = 0.0
    
    return avg_result


def _fmt_header() -> str:
    parts: List[str] = []
    for header, width in COLUMNS:
        parts.append(f"{header:>{width}}")
    return " ".join(parts)


def _fmt_row(res: Dict[str, Any]) -> str:
    values = [
        f"{res['snakes']:>{COLUMNS[0][1]}d}",
        f"{res['cache_misses']:>{COLUMNS[1][1]},}",
        f"{res['cache_references']:>{COLUMNS[2][1]},}",
        f"{res['cache_miss_rate_percent']:>{COLUMNS[3][1]-1}.2f}%",
        f"{res['branch_misses']:>{COLUMNS[4][1]},}",
        f"{res['branch_instructions']:>{COLUMNS[5][1]},}",
        f"{res['branch_miss_rate_percent']:>{COLUMNS[6][1]-1}.2f}%",
    ]
    return " ".join(values)


def main() -> None:
    print("=== Measuring cache and branch metrics across snake counts ===")
    print("Using deterministic hot_path benchmark with consistent CPU frequency and cache state")
    print(f"Running {RUNS_PER_SNAKE_COUNT} measurements per snake count for reliability")
    print()
    
    results: List[Dict[str, Any]] = []

    # Header
    header = _fmt_header()
    print(header)
    print("-" * len(header))

    for n in SNAKE_COUNTS:
        print(f"\nMeasuring {n} snakes:")
        res = run_multiple_perf_measurements(n, RUNS_PER_SNAKE_COUNT)
        results.append(res)
        print(f"  Average results: {_fmt_row(res)}")
        if res["runs_completed"] > 1:
            print(f"  Std dev - Cache misses: {res['std_dev_cache_misses']:,.0f}, Branch misses: {res['std_dev_branch_misses']:,.0f}")

    # Save JSON summary
    out_file = "perf_summary.json"
    with open(out_file, "w") as f:
        json.dump({
            "metadata": {
                "runs_per_snake_count": RUNS_PER_SNAKE_COUNT,
                "snake_counts": SNAKE_COUNTS,
                "perf_events": PERF_EVENTS,
                "benchmark": "hot_path",
                "description": "Deterministic hot path benchmark with consistent CPU frequency and cache state"
            },
            "results": results
        }, f, indent=2)

    print(f"\nSaved summary to: {out_file}")
    print(f"Each snake count was measured {RUNS_PER_SNAKE_COUNT} times and averaged")


if __name__ == "__main__":
    main()
