# Performance Evaluation Within API Counterparts

This directory contains a small, self-contained suite:

- `API-info/` provides curated CSV metadata for documentation and review.
- `src/` provides a minimal compiler frontend that extracts exact DefPaths from crates under `base/`.
- `benches/` provides micro-benchmarks for safe vs. unsafe performance evaluation.


## Directory Layout

### 1) `API-info/`: Curated API metadata (CSV)

`API-info/` stores the API lists that we manually collected and validated. Each CSV file corresponds to one API category.

### 2) `src/`: Minimal compiler frontend (DefPath dumper)

`src/` implements a minimal Rust compiler frontend based on `rustc_driver`.  
Its only purpose is to compile and analyze a target crate under `base/`, then traverse the crate’s MIR to collect all direct function calls and print the callee’s fully qualified definition path (DefPath), such as:

- `core::slice::<impl [T]>::get`

To run this demonstration, use the following commands:
```sh
$ cargo build
$ ./target/debug/micro-benchmark ./base/index/src/main.rs 
```



### 3) `benches/` : Independent micro-benchmark module

`benches/` contains micro-benchmarks with `criterion` that compare safe APIs against their unsafe counterparts. 
The benchmarks are designed to quantify the potential speedup when a call site is proven safe and can be replaced with an unsafe variant.

To run the evaluation, use the following command:

```sh
$ cargo bench --bench <bench_name>
```

The bench name list are shown in `Cargo.toml`.
