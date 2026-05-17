# HKDF Application-Level Rerun Results

This rerun follows the setup described in `hkdf.md`.

## Setup

- Safe version: `/tmp/hkdf-e2e-rerun/ring-safe`
- Transformed version: `/tmp/hkdf-e2e-rerun/ring-unsafe`
- Benchmark harnesses:
  - `/tmp/hkdf-e2e-rerun/bench-safe`
  - `/tmp/hkdf-e2e-rerun/bench-unsafe`
- Target path: application-level HKDF helper -> `ring::hkdf::Okm::fill()` -> internal `fill_okm()`
- Replacement inside `fill_okm()`:
  - `split_at_mut(...)` -> `split_at_mut_unchecked(...)`
  - `checked_add(1).unwrap()` -> `unchecked_add(1)`
- Workload: 2048-byte HKDF output.
- Benchmark command: Criterion default configuration through `cargo bench --bench e2e_compare -- --quiet`.
- Execution order: five confirmed sequential rounds, each running safe first and transformed second.

## Per-Round Results

| Round | Safe mean (us) | Unsafe mean (us) | Safe/Unsafe | Mean reduction | Safe slope (us) | Unsafe slope (us) | Slope reduction |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 10.929 | 10.358 | 1.055 | 5.23% | 10.866 | 10.366 | 4.60% |
| 2 | 10.486 | 10.214 | 1.027 | 2.60% | 10.379 | 10.266 | 1.09% |
| 3 | 10.188 | 10.386 | 0.981 | -1.94% | 10.165 | 10.347 | -1.80% |
| 4 | 10.388 | 10.073 | 1.031 | 3.02% | 10.403 | 10.051 | 3.38% |
| 5 | 10.245 | 10.691 | 0.958 | -4.36% | 10.243 | 10.728 | -4.74% |

## Summary

| Metric | Average | Median | Minimum | Maximum |
|---|---:|---:|---:|---:|
| Safe mean (us) | 10.447 | 10.388 | 10.188 | 10.929 |
| Unsafe mean (us) | 10.345 | 10.358 | 10.073 | 10.691 |
| Mean safe/unsafe | 1.010 | 1.027 | 0.958 | 1.055 |
| Mean latency reduction | 0.91% | 2.60% | -4.36% | 5.23% |
| Safe slope (us) | 10.411 | 10.379 | 10.165 | 10.866 |
| Unsafe slope (us) | 10.352 | 10.347 | 10.051 | 10.728 |
| Slope safe/unsafe | 1.006 | 1.011 | 0.955 | 1.048 |
| Slope latency reduction | 0.51% | 1.09% | -4.74% | 4.60% |

## Interpretation

The rerun does not reproduce the previously recorded 9.3% improvement in `hkdf.md`.
Across five confirmed sequential rounds, the transformed version has only a low-single-digit average advantage and the direction is not stable across all runs.
This supports describing the HKDF application-level case as semantically validated with at most slight performance improvement, rather than as evidence of a robust end-to-end speedup.
