# Bypasser Supported Fragment

This document records the analyzer boundary after the numerical-only reduction completed on 2026-03-30.

## Current Scope

The analyzer is now intentionally conservative:

- The abstract state is numerical-only.
- The only active numerical domain is `interval`.
- Heap, alias, ownership, promoted-constant, and symbolic object tracking are not modeled as first-class analysis state.
- Calls outside the supported fragment are explicitly downgraded to `Unsupported` and their destinations are forgotten to `unknown`.

## Supported Cases

The current supported fragment is limited to local, intraprocedural-style checks whose safety condition can be expressed with integer constraints:

- Integer `checked_add`
  - The analyzer checks whether the addition may overflow.
  - The returned `Option` is not modeled precisely; it is forgotten to `unknown`.
- Primitive slice/array local bounds checks
  - `Index::index`
  - `slice::get`
  - `slice::get_mut`
  - `slice::get_unchecked`
  - `slice::get_unchecked_mut`
  - `slice::split_at`
  - `slice::split_at_mut`
  - `slice::swap`
  - These are only treated as supported when the indexed/sliced sequence element type is a primitive scalar (`bool`, `char`, integers, floats).
  - The analyzer only reasons about the local bound/split condition.
  - The returned reference, `Option`, or sub-slice is not modeled precisely; it is forgotten to `unknown`.
  - For `swap`, the post-swap contents are not modeled precisely; sequence-derived facts are forgotten after the bounds checks.
- `std::mem::size_of::<T>()`
  - This is inlined as a concrete byte size when layout is available.

## Explicitly Unsupported

The following are intentionally outside the supported fragment and now produce `Unsupported` diagnostics instead of pretending to analyze them:

- Heap-backed reconstruction and ownership transfer
  - `Vec::from_raw_parts`
  - `Into<Vec<_>>`
  - `__rust_alloc` / `__rust_alloc_zeroed`
- Pointer arithmetic and relational pointer reasoning
  - `offset`, `add`, `sub`
  - `byte_offset`, `byte_add`, `byte_sub`
  - `offset_from`, `byte_offset_from`
  - wrapping variants of the above
- Reference/ownership conversion side effects
  - `From`
  - `as_mut_ptr`

## Result Semantics

At the moment, analyzer outputs should be interpreted as:

- Supported diagnostic:
  - The reported check comes from the supported numerical fragment.
- Unsupported diagnostic:
  - The analyzer reached a call site outside the supported fragment and downgraded the result to `unknown`.
- No diagnostic:
  - Not a global proof of safety.
  - It only means no diagnostic was produced under the current supported fragment and current precision.

## Current Output Summary

The analyzer summary now records:

- `total_diagnostics`
- `supported_diagnostics`
- `unsupported_diagnostics`
- `supported_special_calls`
- `unsupported_special_calls`

These numbers are intended to make the paper claim align with the actual implementation boundary.

## Intended Paper Claim

The implementation now supports a narrower and more defensible claim:

- This is a conservative numerical checker for a restricted fragment of Rust unchecked/bypass-related APIs.
- It is not a full Rust heap/alias/interprocedural memory analyzer.
