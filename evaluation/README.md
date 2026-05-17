# Opt-Level Evaluation Harness

This directory contains paired safe/unsafe crate copies for target-function-level
optimization-sensitivity experiments.

## Layout

- `crates/itertools-safe` and `crates/itertools-unsafe`: `kmerge` path reaching `sift_down`.
- `crates/rand-safe` and `crates/rand-unsafe`: `partial_shuffle`.
- `crates/ring-safe` and `crates/ring-unsafe`: HKDF `Okm::fill` path reaching `fill_okm`.
- `crates/arrayvec-safe` and `crates/arrayvec-unsafe`: `swap_pop`.
- `crates/bit-vec-safe` and `crates/bit-vec-unsafe`: `push`.

Each crate pair has a `bypasser_target` Criterion benchmark, except `ring`,
whose benchmark is in its existing `bench` package as `bypasser_hkdf`.

## Running One Case

```bash
cd evaluation/crates/arrayvec-safe
CARGO_PROFILE_BENCH_OPT_LEVEL=2 cargo bench --bench bypasser_target -- --noplot

cd ../arrayvec-unsafe
CARGO_PROFILE_BENCH_OPT_LEVEL=2 cargo bench --bench bypasser_target -- --noplot
```

For `ring`, run from the nested benchmark package:

```bash
cd evaluation/crates/ring-safe/bench
CARGO_PROFILE_BENCH_OPT_LEVEL=2 cargo bench --bench bypasser_hkdf -- --noplot
```

## Running All Configurations

```bash
bash evaluation/run_opt_level_benches.sh
```

The script evaluates `opt-level=1`, `opt-level=2`, and `opt-level=3` for each
safe/unsafe pair and stores raw Criterion logs under `evaluation/results/`.
