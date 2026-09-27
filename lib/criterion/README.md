# criterion

A small, std-only benchmark runner for this workspace. It provides the
Criterion-style APIs used by the SHA benchmarks and has no dependencies.

Supported APIs: `Criterion`, `BenchmarkGroup`, `BenchmarkId`, `Throughput`,
`Bencher::iter`, `iter_batched`, `iter_batched_ref`, `iter_custom`,
`criterion_group!`, and `criterion_main!`. Results show the median and the
10th and 90th percentile sample times. These percentiles describe observed
variation; they are not confidence intervals.

Run a benchmark with Cargo:

```sh
cargo bench -p sha1 --bench sha1 --features disable-simd
cargo bench -p sha1 --bench sha1 --features disable-software
```

Optional arguments after `--`:

- A substring filters benchmark names, for example `-- digest/1048576`.
- `--sample-size 20` sets the number of timed samples (default: 15).
- `--measurement-time 2` sets the total target measurement time in seconds
  (default: 3).
- `--warm-up-time 1` sets the minimum warm-up time in seconds (default: 0.1).
- `--test` runs every selected case once; `--list` prints case names.
- `--save-baseline PATH` writes median times as tab-separated data.
- `--baseline PATH` compares each median with a saved baseline.

The runner has no plots, HTML reports, or statistical significance tests.
The baseline percentage is a direct comparison of two medians. Run competing
builds under similar system load and repeat close comparisons.

The SHA crates normally select SIMD at runtime and keep a software fallback.
`disable-simd` benchmarks the software path, while `disable-software` requires
SIMD support on the host. When both features are enabled (as with
`--all-features`), they cancel and normal runtime selection applies.
