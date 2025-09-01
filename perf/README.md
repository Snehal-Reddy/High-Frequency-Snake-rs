# perf quick start

Minimal steps to collect low-level CPU/cache metrics for the game tick benchmarks.

## Prereqs

```bash
# perf tools
sudo apt install linux-tools-generic

# allow measurements (temporary)
echo 1 | sudo tee /proc/sys/kernel/perf_event_paranoid

# build in release
git rev-parse --is-inside-work-tree >/dev/null 2>&1 && true
cargo build --release
```

## Single measurements

- Cache metrics for N snakes
```bash
cd perf
./measure_cache.sh 500
# → writes cache_results_500_snakes.txt
```

- Pipeline metrics for N snakes
```bash
cd perf
./measure_pipeline.sh 500
# → writes pipeline_results_500_snakes.txt
```

## All snakes summary (100…1000)

Runs perf and prints a compact table, plus a JSON summary.
```bash
cd perf
./perf_all_summary.py
# → prints table
# → writes perf_summary.json
```

## Outputs

- cache_results_<N>_snakes.txt: raw perf stat output for cache events
- pipeline_results_<N>_snakes.txt: raw perf stat output for pipeline events
- perf_summary.json: aggregated cache/branch metrics across snake counts
