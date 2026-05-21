# Demo Scope

This repository is being narrowed into a demonstrable Rust MIR analysis
artifact. The goal is not to optimize arbitrary Rust programs. The goal is to
show that, for a small set of standard-library safe/unsafe API counterparts,
the analyzer can prove selected call-site preconditions from local MIR facts
and report replacement candidates.

## Supported Fragment

The demo assumes a deliberately small fragment:

- MIR-level local analysis, with only narrowly bounded helper handling.
- Interval-style numerical facts for integer-like MIR places.
- Local pointer-nullness facts where the current implementation can support
  them cleanly.
- Conservative handling of unsupported calls and memory effects.
- No precise alias analysis, memory-object model, raw-pointer arithmetic model,
  or full interprocedural semantic proof.

When an operation falls outside this fragment, the analyzer should downgrade
the relevant facts to unknown and suppress replacement candidates that depend
on those facts.

## Demo API Matrix

| API family | Required condition | Primary demo case | Role |
|---|---|---|---|
| `checked_add` | result is within the integer type range | `tests/checked_add` | Minimal numerical case |
| `slice::get` / `get_mut` | `index < slice.len()` | `tests/get` | Minimal bounds case |
| `slice::split_at` / `split_at_mut` | `mid <= slice.len()` | `tests/split_at`, `tests/ring_buffer_split` | Bounds plus returned-slice length propagation |
| `slice::swap` | `i < slice.len() && j < slice.len()` | `tests/swap`, `case-study/kmerge_impl.rs` | Real extracted bounds case |

The toy cases are kept to make the pipeline reproducible. The main non-toy
case is the extracted `itertools` `sift_down`/`kmerge` code in `case-study`,
where the analyzer should reason about the bounds of `heap.swap(pos, child)`.

## Output Contract

For a proven safe call site, the analyzer should emit a readable replacement
candidate:

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

For unsupported code, the analyzer should report that the result was downgraded
to local unknown, or remain silent if the unsupported site is intentionally
treated as a non-reportable micro-wrapper.

## Current Smoke Commands

The current toy demo functions are all entry `3` in their standalone source
files. Use `--emit=metadata --out-dir target/demo-smoke` while running the
analyzer directly on individual files; this avoids rustc trying to link each
analyzed file into the same repository-root `main` binary.

```sh
mkdir -p target/demo-smoke
env RUST_LOG=warn ./target/debug/api-bypass tests/checked_add/src/main.rs --entry_def_id_index 3 --emit=metadata --out-dir target/demo-smoke
env RUST_LOG=warn ./target/debug/api-bypass tests/get/src/main.rs --entry_def_id_index 3 --emit=metadata --out-dir target/demo-smoke
env RUST_LOG=warn ./target/debug/api-bypass tests/split_at/src/main.rs --entry_def_id_index 3 --emit=metadata --out-dir target/demo-smoke
env RUST_LOG=warn ./target/debug/api-bypass tests/swap/src/main.rs --entry_def_id_index 3 --emit=metadata --out-dir target/demo-smoke
env RUST_LOG=warn ./target/debug/api-bypass tests/ring_buffer_split/src/main.rs --entry_def_id_index 3 --emit=metadata --out-dir target/demo-smoke
```

Expected current behavior:

| Command target | Expected candidate output |
|---|---|
| `tests/checked_add` | one `integer.checked_add(rhs)` candidate |
| `tests/get` | one `slice::get(index)` candidate |
| `tests/split_at` | one `slice::split_at(mid)` candidate |
| `tests/swap` | one `slice::swap(i, j)` candidate |
| `tests/ring_buffer_split` | two `slice::split_at_mut(mid)` candidates, plus conservative call-boundary warnings from the loop body |

## Non-Goals

- Do not claim whole-program optimization.
- Do not claim whole-application speedup.
- Do not claim soundness for arbitrary Rust programs.
- Do not hard-code success by function name.
- Do not analyze full crates when a real extracted component is enough for the
  demo claim.

The artifact should be presented as a conservative checker for replacement
opportunities within its supported fragment.
