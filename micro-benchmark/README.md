# Micro-benchmark Suite

This directory contains a small, self-contained suite used to (1) document the target standard-library APIs and (2) demonstrate how we derive their fully qualified paths, and (3) measure the performance gap between safe and unsafe counterparts under micro-benchmarks. The contents are intentionally kept minimal and orthogonal, so reviewers can inspect and run each part independently.

## Directory Layout

### 1) `API-info/`: Curated API metadata (CSV)

`API-info/` stores the API lists that we manually collected and validated.  
Each CSV file corresponds to one API category (e.g., index-related, overflow-related, nonzero-related) and records the set of standard-library APIs we use in the paper.

These CSV files are used as human-readable ground truth for:
- documenting the target API set,
- driving the construction of small demo crates under `base/`,
- and cross-checking the extracted fully qualified paths.

### 2) `src/`: Minimal compiler frontend (DefPath dumper)

`src/` implements a minimal Rust compiler frontend based on `rustc_driver`.  
Its only purpose is to compile and analyze a target crate under `base/`, then traverse the crate’s MIR to collect all direct function calls and print the callee’s fully qualified definition path (DefPath), such as:

- `core::slice::<impl [T]>::get`
- `alloc::vec::Vec<T>::...`

This module is a lightweight demonstration tool. It does not perform dataflow analysis, summarization, or any safety reasoning. The output is used to validate that the APIs referenced in `base/` are indeed the intended standard-library functions, and to obtain their exact compiler-level paths.

### 3) `bench/` (or `benches/`): Independent micro-benchmark module

`bench/` contains micro-benchmarks that compare safe APIs against their unsafe counterparts (when applicable). The benchmarks are designed to quantify the potential speedup when a call site is proven safe and can be replaced with an unsafe variant.

This module is intentionally separated from `src/` and `API-info/`:
- `bench/` focuses only on timing and benchmarking methodology,
- it does not depend on the DefPath dumper,
- and it does not parse or require the CSV metadata at runtime.

## Independence of the Three Components

The three parts are designed to be non-interfering:

- `API-info/` provides curated CSV metadata for documentation and review.
- `src/` provides a minimal compiler frontend that extracts exact DefPaths from crates under `base/`.
- `bench/` provides micro-benchmarks for safe vs. unsafe performance evaluation.

They can be inspected, built, and executed separately. This separation keeps the benchmarking logic free from compiler-internal dependencies, and keeps the compiler frontend free from benchmarking and measurement noise.

## Typical Usage (High-level)

- To inspect the target API set, start from `API-info/`.
- To see how fully qualified paths are obtained, run the DefPath dumper in `src/` on a chosen crate under `base/`.
- To reproduce the micro-benchmark results, run the benchmarks under `bench/`.

Each part is intentionally small and self-explanatory, so reviewers can quickly verify the intended behavior with minimal setup.
