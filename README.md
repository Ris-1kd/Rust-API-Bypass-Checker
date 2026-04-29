# Rust API Bypass Checker

A conservative MIR-based checker for a restricted set of Rust safe/unsafe API counterparts. The current implementation focuses on local numerical conditions under which specific unchecked calls may be safe, while explicitly downgrading unsupported calls to unknown and keeping only a tiny wrapper-like exception set at call boundaries.

## Overview

This tool analyzes MIR-level control-flow and integer constraints around a small supported fragment of checked/unchecked APIs. In the current implementation, it is intended to:

- `slice.get(index)` → `slice.get_unchecked(index)` when bounds are proven safe
- `slice.split_at(mid)` → `slice.split_at_unchecked(mid)` when split bounds are proven safe
- `slice.swap(a, b)` under locally provable in-bounds indices
- `integer.checked_add(other)` when overflow is provably impossible

## Example

```rust
fn process_array(arr: &[i32]) {
    for i in 0..arr.len() {
        // Redundant bounds check - loop condition guarantees i < arr.len()
        let value = arr.get(i).unwrap(); // Can optimize to arr[i] or arr.get_unchecked(i)
        println!("{}", value);
    }
}
```

The tool aims to recognize that the bounds check in `arr.get(i)` is locally redundant and to surface a diagnostic within its supported fragment, rather than to perform automatic rewriting.

## Supported Fragment

The analyzer is intentionally narrow.

- The abstract state is numerical-only.
- The active numerical domain is `interval`.
- The current reasoning is intraprocedural in spirit: default descent into ordinary callees is disabled.
- Special handling is limited to a small whitelist of local checked/unchecked APIs, such as `get`, `split_at`, `swap`, and `checked_add`.
- A tiny micro-wrapper exception set suppresses selected boolean function-trait shims without restoring general interprocedural descent.
- Calls outside this fragment are downgraded to local unknowns at the call boundary.

## Result Semantics

Diagnostics should be interpreted conservatively.

- A supported diagnostic comes from the supported numerical fragment.
- An unsupported or call-boundary diagnostic means the analyzer deliberately stopped and downgraded the result to unknown.
- Selected boolean callback wrappers may also be downgraded to unknown silently when they are treated as local shims rather than reportable boundaries.
- The absence of a diagnostic is not a global proof of safety.

## Requirements

* Rust nightly (`nightly-2025-01-10`)
* Dependencies:
  ```sh
$ rustup component add rustc-dev llvm-tools-preview
  $ sudo apt-get install libgmp-dev libmpfr-dev libppl-dev libz3-dev llvm-15 clang-15 libclang-15-dev  # Ubuntu
  $ export LIBCLANG_PATH=`llvm-config-15 --libdir`/libclang.so
  ```

## Installation

```sh
$ git clone --recursive https://github.com/Rust-API/Rust-API-Bypass.git
$ cd rust-api-bypass
$ export LIBCLANG_PATH=`llvm-config-15 --libdir`/libclang.so
$ export RUSTFLAGS="-Clink-args=-fuse-ld=lld"
$ cargo build
```

The root crate currently pins `nightly-2025-01-10`. A separate benchmark-only workspace under `API-counterprats/` may use a newer nightly for Criterion experiments.

## Usage

```sh
# Analyze a crate via main.rs or lib.rs

# Inspect candidate entry functions
$ ./target/debug/api-bypass <file> --show_reachable_entries

# Analyze a particular function as the root of a local numerical run
$ ./target/debug/api-bypass <file> --entry_def_id_index <defid> 
```

### Options

- `--entry_def_id_index <function>`: Entry function DefId (acquired via `show_reachable_entries`)
- `--show_all_entries`: Display all candidate entry functions within the current crate.
- `--show_reachable_entries`: Display entry candidates discovered by the current front-end scan.

## Test Cases

- `tests/checked_add/`: Local integer overflow-check scenarios
- `tests/get/`: Local slice bounds-check scenarios
- `tests/split_at/`: Local split-index reasoning scenarios
- `tests/swap/`: Local two-index bounds reasoning scenarios
- `case-study/`: A larger MIR case used to stress-test the reduced `swap` support path under local callback-boundary downgrades

## License

See [LICENSE](LICENSE) and [licenses](licenses).

## Acknowledgments

Built upon:
- [MirChecker](https://github.com/lizhuohua/rust-mir-checker) - Original MIR analysis framework from which this checker was narrowed and adapted
- [MIRAI](https://github.com/facebookexperimental/MIRAI) - Static analysis techniques
- [Crab](https://github.com/seahorn/crab) - Abstract domain implementations
