#!/usr/bin/env python3
"""
Measure cache and branch behavior across snake counts.
- Runs: perf stat -e cache-misses,cache-references,branch-instructions,branch-misses
- Bench target: cargo bench --bench game_bench -- "game_tick_max_inputs/{N}_snakes"
- Parses stderr (perf output), sums across cpu_atom/cpu_core, computes miss rates
- Prints a compact table and writes JSON summary to cache_branch_summary.json
"""

import subprocess
import json
import re
from typing import Dict, Any, List

SNAKE_COUNTS = [100, 200, 300, 400, 500, 600, 700, 800, 900, 1000]
PERF_EVENTS = "cache-misses,cache-references,branch-instructions,branch-misses"

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
    cmd = [
        "perf", "stat", "-e", PERF_EVENTS,
        "cargo", "bench", "--bench", "game_bench", "--",
        f"game_tick_max_inputs/{snake_count}_snakes",
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
    results: List[Dict[str, Any]] = []

    # Header
    header = _fmt_header()
    print(header)
    print("-" * len(header))

    for n in SNAKE_COUNTS:
        res = run_perf_for_snakes(n)
        results.append(res)
        print(_fmt_row(res))

    # Save JSON summary
    out_file = "perf_summary.json"
    with open(out_file, "w") as f:
        json.dump({"results": results}, f, indent=2)

    print("\nSaved summary to:", out_file)


if __name__ == "__main__":
    main()
