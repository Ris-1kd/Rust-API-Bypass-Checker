# Rust API Bypass Checker: Redundant Safety Check Optimizer

A static analysis tool that identifies redundant safety checks in Rust programs to improve performance. By analyzing MIR (Mid-level Intermediate Representation), the tool detects where safe APIs can be safely replaced with their unchecked counterparts without compromising memory safety.

## Overview

This tool analyzes code patterns to identify unnecessary safety checks that are already guaranteed by program logic, enabling performance optimizations such as:

- `slice.get(index)` → `slice.get_unchecked(index)` when bounds are proven safe
- `integer.checked_add(other)` → `integer.wrapping_add(other)` when overflow is impossible
- Array indexing optimizations when bounds are logically guaranteed

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

The tool identifies that the bounds check in `arr.get(i)` is redundant and suggests using direct indexing for better performance.

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

## Usage

```sh
# Analyze a file
$ ./target/debug/mir-checker <file> --entry main --domain interval --widening_delay 5 --narrowing_iteration 5

# Example with test case
$ ./target/debug/mir-checker ./tests/get/src/main.rs --entry main --domain interval --widening_delay 5 --narrowing_iteration 5
```

### Options

- `--entry <function>`: Entry function (default: `main`)
- `--domain <domain>`: Abstract domain (`interval`, `octagon`, `polyhedra`, etc.)
- `--widening_delay <N>`: Iterations before widening (recommended: 5)
- `--narrowing_iteration <N>`: Max narrowing iterations (recommended: 5)

## Test Cases

- `tests/checked_add/`: Integer arithmetic optimization scenarios
- `tests/get/`: Slice access pattern optimizations
- `tests/split_at/`: Slice splitting boundary check optimizations
- `tests/swap/`: Element swapping safety validation


## Architecture

- **MIR Analysis Engine**: Processes Rust's mid-level IR for control flow analysis
- **Abstract Domains**: Multiple numerical domains for precise range analysis
- **Pattern Recognition**: Identifies optimization opportunities in known API patterns
- **Optimization Advisor**: Reports safe-to-remove redundant checks

## Debugging

```sh
$ export RUST_LOG=rust_mir_checker=debug
$ ./target/debug/mir-checker <file> --entry main --domain interval
```

## License

See [LICENSE](LICENSE) and [licenses](licenses).

## Acknowledgments

Built upon:
- [MirChecker](https://github.com/lizhuohua/rust-mir-checker) - MIR analysis framework
- [MIRAI](https://github.com/facebookexperimental/MIRAI) - Static analysis techniques
- [Crab](https://github.com/seahorn/crab) - Abstract domain implementations