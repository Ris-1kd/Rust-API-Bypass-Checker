# Rust API Bypass Checker

A conservative MIR-based checker for a restricted set of Rust safe/unsafe API counterparts. The current implementation focuses on local numerical and pointer-nullness conditions under which specific unchecked calls may be safe, while explicitly downgrading unsupported calls to unknown and keeping only a tiny wrapper-like exception set at call boundaries.

## Overview

This tool analyzes MIR-level control-flow and integer constraints around a small supported fragment of checked/unchecked APIs. In the current implementation, it is intended to:

- `slice.get(index)` → `slice.get_unchecked(index)` when bounds are proven safe
- `slice.split_at(mid)` → `slice.split_at_unchecked(mid)` when split bounds are proven safe
- `slice.swap(a, b)` under locally provable in-bounds indices
- `integer.checked_add(other)` when overflow is provably impossible
- `ptr.as_ref()`, `ptr.as_mut()`, and `NonNull::new(ptr)` when pointer nullness is locally known

## Current Demo Scope

The project is currently being narrowed into a self-contained demo artifact. The intended claim is conservative:

> For a known set of standard-library API pairs, the analyzer can prove selected call-site preconditions from local MIR facts and report replacement opportunities.

The demo does not claim whole-program optimization, whole-application speedup, full alias analysis, full raw-pointer reasoning, or semantic preservation for arbitrary Rust programs. Unsupported calls and memory effects are downgraded to local unknown, and replacement candidates are suppressed when their preconditions depend on those unknown facts.

The primary demo matrix is:

| API family | Required condition | Demo case |
|---|---|---|
| `checked_add` | result is within the integer type range | `tests/checked_add` |
| `slice::get` / `get_mut` | `index < slice.len()` | `tests/get` |
| `slice::split_at` / `split_at_mut` | `mid <= slice.len()` | `tests/split_at`, `tests/ring_buffer_split` |
| `slice::swap` | `i < slice.len() && j < slice.len()` | `tests/swap`, `case-study/kmerge_impl.rs` |

See [DEMO_SCOPE.md](DEMO_SCOPE.md) for the detailed demo boundary and non-goals.

## Example

```rust
fn process_array(arr: &[i32]) {
    for i in 0..arr.len() {
        // Redundant bounds check - loop condition guarantees i < arr.len()
        let value = arr.get(i).unwrap(); // Can optimize to arr[i] or arr.get_unchecked(i)
        println!("Value is {}", value);
    }
}
```

The tool aims to recognize that the bounds check in `arr.get(i)` is locally redundant and to surface a diagnostic within its supported fragment, rather than to perform automatic rewriting.
Ordinary helper calls remain local unknowns by default, except for a tiny wrapper-like shim set that is intentionally suppressed rather than analyzed interprocedurally.

## Supported Fragment

The analyzer is intentionally narrow.

- The abstract state combines interval-style numerical facts with local pointer-nullness facts.
- The active numerical domain is `interval`.
- The current reasoning is intraprocedural in spirit: default descent into ordinary callees is disabled.
- Special handling is limited to a small whitelist of local checked/unchecked APIs, such as `get`, `split_at`, `swap`, `checked_add`, and pointer nullness checks.
- A tiny micro-wrapper exception set suppresses selected boolean function-trait shims without restoring general interprocedural descent.
- Calls outside this fragment are downgraded to local unknowns at the call boundary.

## Result Semantics

Diagnostics should be interpreted conservatively.

- A supported diagnostic comes from the supported numerical or pointer-nullness fragment.
- An unsupported or call-boundary diagnostic means the analyzer deliberately stopped and downgraded the result to unknown.
- Selected boolean callback wrappers may also be downgraded to unknown silently when they are treated as local shims rather than reportable boundaries.
- The absence of a diagnostic is not a global proof of safety.

## Requirements

* Rust nightly (`nightly-2025-01-10`)
* Dependencies:
  ```sh
  $ rustup component add rustc-dev llvm-tools-preview
  $ sudo apt-get install libgmp-dev libmpfr-dev libz3-dev  # Ubuntu
  ```

## Installation

```sh
$ git clone https://github.com/Rust-API/Rust-API-Bypass-Checker.git
$ cd Rust-API-Bypass-Checker
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

## Demo Output Shape

When a supported safe API call is proven to satisfy the corresponding unchecked precondition, the analyzer emits a replacement-candidate diagnostic:

```text
[Bypasser] Replacement candidate

Safe API:
  slice::split_at(mid)

Suggested replacement:
  unsafe { slice::split_at_unchecked(mid) }

Required condition:
  mid <= slice.len()

Analysis result:
  proven from local MIR split-index facts
```

The diagnostic is a reporting aid, not an automatic source rewrite.

## Test Cases

- `tests/checked_add/`: Local integer overflow-check scenarios
- `tests/get/`: Local slice bounds-check scenarios
- `tests/split_at/`: Local split-index reasoning scenarios
- `tests/swap/`: Local two-index bounds reasoning scenarios
- `tests/nullness/`: Local pointer nullness scenarios for checked pointer APIs
- `case-study/`: A larger MIR case used to stress-test the reduced `swap` support path under local callback-boundary downgrades

## License

See [LICENSE](LICENSE) and [licenses](licenses).

## Acknowledgments

Built upon:
- [MirChecker](https://github.com/lizhuohua/rust-mir-checker) - Original MIR analysis framework from which this checker was narrowed and adapted
- [MIRAI](https://github.com/facebookexperimental/MIRAI) - Static analysis techniques
- [Crab](https://github.com/seahorn/crab) - Abstract domain implementations
