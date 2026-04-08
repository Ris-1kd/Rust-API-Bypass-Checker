# Candidate Bench Audit (2026-03-30)

This note summarizes the first full pass over the generated candidate Criterion results in this directory.

## Result files

- Detailed per-bench result: `candidate_bench_summary_2026-03-30.csv`
- Compact per-bench result: `candidate_bench_summary_compact_2026-03-30.csv`
- Family aggregate: `candidate_bench_summary_by_family_2026-03-30.csv`

## Family summary

These aggregates are over source-scan entries, not yet de-duplicated semantic APIs.

| Safety family | Count | Avg safe/unsafe | Min | Max |
| --- | ---: | ---: | ---: | ---: |
| Aliasing_Pinning_and_Exclusive_Access | 5 | 1.233765 | 0.994754 | 2.060385 |
| Arithmetic_and_Bit_Preconditions | 21 | 1.170050 | 0.853208 | 2.128918 |
| Bounds_Length_and_Position | 4 | 1.052974 | 1.001170 | 1.127473 |
| Encoding_and_Text_Validity | 7 | 8.609126 | 1.228868 | 10.116458 |
| Layout_Initialization_and_Shape | 2 | 1.120124 | 1.095565 | 1.144682 |
| Ordering_and_Cursor_Position | 8 | 0.969270 | 0.917603 | 1.002638 |
| Value_Validity_and_Representability | 13 | 1.315386 | 0.976389 | 2.247322 |
| Variant_and_Control_Flow_Assumptions | 3 | 1.067799 | 1.043064 | 1.111083 |

## High-confidence findings

- `Encoding_and_Text_Validity` is the outlier family by a large margin. This is expected for APIs like `from_utf8[_unchecked]`, `from_bytes_with_nul[_unchecked]`, and `from_vec_with_nul[_unchecked]`, because the safe path performs full validation over the input while the unsafe path skips it.
- `Ordering_and_Cursor_Position` is close to parity and often slightly favors the safe version. For these BTree cursor benches, cursor setup and container work dominate the explicit safe/unsafe check.
- `Arithmetic_and_Bit_Preconditions` mostly lands near parity, with a few stronger wins for the `*_exact` APIs where the checked path does materially more work.

## Audit issues

### Invalid bench that should not be used for conclusions

- `candidate_alloc_string_from_utf8_unchecked_1013`
  - Source pair is `alloc::string::String::from_utf8` vs `String::from_utf8_unchecked`.
  - The generated bench currently calls `std::str::from_utf8` vs `std::str::from_utf8_unchecked`.
  - Its recorded ratio is therefore not a valid measurement for the `String` API pair.

### Configuration mismatch

- 55 generated benches use `sample_size(80)` and `measurement_time(Duration::from_secs(5))`.
- 8 BTree cursor benches use `sample_size(60)` and `measurement_time(Duration::from_secs(4))`:
  - `candidate_alloc_collections_btree_map_insert_after_unchecked_3418`
  - `candidate_alloc_collections_btree_map_insert_after_unchecked_3623`
  - `candidate_alloc_collections_btree_map_insert_before_unchecked_3461`
  - `candidate_alloc_collections_btree_map_insert_before_unchecked_3641`
  - `candidate_alloc_collections_btree_set_insert_after_unchecked_2371`
  - `candidate_alloc_collections_btree_set_insert_after_unchecked_2457`
  - `candidate_alloc_collections_btree_set_insert_before_unchecked_2389`
  - `candidate_alloc_collections_btree_set_insert_before_unchecked_2475`

This means the current result set is not yet perfectly uniform in Criterion settings across all 63 generated benches.

### Duplicate semantic families from multiple source locations

The current source-scan is location-based, so some rows correspond to the same conceptual API family appearing in multiple source entry points or wrappers. Examples:

- `from_utf8_unchecked` / `from_utf8`
  - `core/src/str/converts.rs:178`
  - `core/src/str/mod.rs:316`
  - `alloc/src/string.rs:1013` (also currently mis-benched as noted above)
- `from_utf8_unchecked_mut` / `from_utf8_mut`
  - `core/src/str/converts.rs:208`
  - `core/src/str/mod.rs:341`
- `from_u32_unchecked` / `from_u32`
  - `core/src/char/methods.rs:237`
  - `core/src/char/mod.rs:141`
- `as_ascii_unchecked` / `as_ascii`
  - array, char, num, slice, and str variants all appear separately

So the family aggregates above are useful for an initial directional view, but they are not yet a de-duplicated semantic benchmark inventory.

## Interpretation notes for the largest ratios

- `core::str::from_utf8[_unchecked]` and `core::ffi::CStr::from_bytes_with_nul[_unchecked]`
  - Large ratios are expected because the safe version validates every byte and the unsafe version does not.
- `alloc::sync::Arc::get_mut[_unchecked]`
  - The ratio is plausibly real. The safe version must prove uniqueness and returns `Option<&mut T>`, while the unsafe version skips that uniqueness check.
- `slice/array as_ascii[_unchecked]`
  - Large ratios are also plausible because the safe version checks every byte for ASCII validity.

## Recommended next cleanup

- Fix and rerun `candidate_alloc_string_from_utf8_unchecked_1013` so it measures `String::from_utf8` vs `String::from_utf8_unchecked`.
- Unify the 8 BTree cursor benches onto the same Criterion settings as the other generated benches.
- Optionally build a de-duplicated view of conceptual API pairs before comparing family-level averages too aggressively.
