# Experiment Data Notes

Date: 2026-05-18

This note collects the detailed data discussed for the revision, including
application-derived case-study candidates and optimization-level sensitivity
results.

## Source Files

- `E2E_CASE_STUDY_EXPERIMENTS.md`
- `conversations/E2E_CASE_STUDY_DISCUSSION_2026-05-18.md`
- `evaluation/results/opt_level_target_function_summary.md`
- `evaluation/results/rand_size_opt_summary.md`
- `API-counterprats/bench-results/selected_bench_overview_opt_sensitivity_all.csv`

## Application-Derived / E2E Candidate Data

These experiments are intended to be more realistic than isolated API
micro-benchmarks, but most are still component-level or application-derived
workloads rather than full whole-application end-to-end measurements.

### RustScan Port Planning

Replacement:

- Safe: `slice.swap(i, j)`
- Unsafe: pointer-based swap with valid generated indices

Validation:

- Proptest differential testing passed.
- The same generated port vectors and RNG seeds were used for safe and unsafe
  versions.

| Workload | Safe Time | Unsafe Time | Safe/Unsafe | Interpretation |
|---:|---:|---:|---:|---|
| 1,024 ports | 5.1854 us | 5.2583 us | 0.986x | Unsafe is slightly slower |
| 8,192 ports | 43.407 us | 45.454 us | 0.955x | Unsafe is slower |
| 65,535 ports | 358.13 us | 361.67 us | 0.990x | Near parity, slightly negative |

Comment:

This is a real and short application component, but it is not a useful positive
case. The compiler likely already handles the simple Fisher-Yates bounds well,
so replacing `swap` does not remove meaningful overhead.

### Arti Timeout Estimator

Path:

```text
circuit build-time history
  -> sparse histogram bins
  -> k_smallest(n_modes)
  -> itertools::k_smallest::sift_down
  -> heap.swap(...)
```

Replacement:

- Safe: `heap.swap(...)`
- Unsafe: pointer-based swap in the vendored `itertools::sift_down`

Validation:

- Proptest differential testing passed.
- Random build-time samples and mode counts were generated, then safe and
  unsafe estimators were compared for the same computed `Xm`.

| Workload | Safe Time | Unsafe Time | Safe/Unsafe | Interpretation |
|---|---:|---:|---:|---|
| 1,000 samples, 10 modes | 6.7796 us | 6.8428 us | 0.991x | Slightly negative |
| 8,000 samples, 10 modes | 51.792 us | 52.577 us | 0.985x | Slightly negative |
| 65,536 samples, 10 modes | 422.55 us | 420.99 us | 1.004x | Near parity |
| 65,536 samples, 64 modes | 420.75 us | 420.84 us | 1.000x | Parity |
| Wide 65,536 samples, 64 modes | 483.70 us | 477.59 us | 1.013x | Best observed, about 1.3% |

Comment:

Arti is semantically strong because it follows a realistic higher-level path.
However, the replacement is only a small part of the estimator. Histogram
construction and iterator logic dominate, so the observed improvement is weak.

### ring BigInt Buffer Splitting

Path:

```text
ring bigint exponentiation buffer-processing component
  -> repeated fixed-layout buffer partitioning
  -> split_at_mut(m_len)
```

Replacement:

- Safe: `split_at_mut(m_len)`
- Unsafe: unchecked mutable slice construction equivalent to
  `split_at_mut_unchecked`

Validation:

- Proptest differential testing passed.
- The test generated different region sizes and record counts and checked both
  output checksum and final buffer state.

| m_len | Safe Time | Unsafe Time | Safe/Unsafe | Interpretation |
|---:|---:|---:|---:|---|
| 1 | 21.412 us | 21.205 us | 1.010x | Weak positive |
| 2 | 41.270 us | 41.582 us | 0.993x | Slightly negative |
| 4 | 80.020 us | 79.156 us | 1.011x | Weak positive |
| 8 | 144.59 us | 146.41 us | 0.988x | Slightly negative |
| 16 | 278.66 us | 274.86 us | 1.014x | Weak positive |
| 32 | 255.38 us | 245.88 us | 1.039x | Best observed, about 3.9% |

Comment:

This is the best positive case among the realistic candidates. It should still
be described conservatively as an application-derived component, not a full
application speedup. It is also not the same as the existing Table 5 `ring`
HKDF target unless Table 5 is updated or the text clearly frames it as a
separate case study.

### rand partial_shuffle

Path:

```text
rand::seq::SliceRandom::partial_shuffle
  -> slice.swap(...)
  -> split_at_mut(...)
```

Replacement:

- Safe: `swap` and `split_at_mut`
- Unsafe: `swap_unchecked` and `split_at_mut_unchecked`

Validation:

- Proptest differential testing passed.
- Same input and same RNG seed were used for both versions.

| Safe Time | Unsafe Time | Safe/Unsafe | Interpretation |
|---:|---:|---:|---|
| 7.7491 us | 7.7445 us | 1.001x | Essentially no improvement |

Comment:

Scope is correct and it is already in the evaluation harness, but it is not a
good positive case.

### arrayvec swap_pop

Path:

```text
arrayvec::ArrayVec::swap_pop
  -> slice.swap(index, len - 1)
```

Replacement:

- Safe: `slice::swap`
- Unsafe: `slice::swap_unchecked`

Validation:

- Proptest differential testing passed.
- Compared against a `Vec` model.

| Safe Time | Unsafe Time | Safe/Unsafe | Interpretation |
|---:|---:|---:|---|
| 1.5843 us | 1.5697 us | 1.009x | Too small |

Comment:

Scope is correct, but the gain is too small to serve as a convincing case
study.

### bit-vec push

Path:

```text
bit_vec::BitVec::push
  -> nbits.checked_add(1).expect(...)
```

Replacement:

- Safe: `checked_add(1).expect(...)`
- Unsafe: `unchecked_add(1)`

Validation:

- Proptest differential testing passed.
- Compared final `BitVec` contents with a `Vec<bool>` model.

| Safe Time | Unsafe Time | Safe/Unsafe | Interpretation |
|---:|---:|---:|---|
| 180.95 us | 180.55 us | 1.002x | Essentially no improvement |

Comment:

Scope is correct, but the gain is too small.

### ripgrep Match::offset

Path:

```text
Searcher::find
  -> core.find(&slice[pos..])
  -> m.offset(self.core.pos())
  -> Match::offset
  -> checked_add(...).unwrap()
```

Replacement:

- Safe: `usize::checked_add(amount).unwrap()`
- Unsafe: `usize::unchecked_add(amount)`

Validation:

- Proptest differential testing passed.
- Generated valid `Match { start, end }` values and offsets, then compared safe
  and unchecked shifted matches.

| Workload | Safe Time | Unsafe Time | Safe/Unsafe | Interpretation |
|---|---:|---:|---:|---|
| 16K matches | 12.169 us | 10.274 us | 1.184x | Strong positive |
| 65K matches | 49.388 us | 41.093 us | 1.202x | Strong positive |
| 262K matches | 231.86 us | 172.61 us | 1.343x | Strong positive |

Comment:

This is the strongest positive extracted component. Scope is correct because
the replacement is a standard-library integer API pair. However, it is not a
whole ripgrep end-to-end benchmark and it is not currently aligned with an
existing Table 5 target unless reported separately.

### Arrow ScalarBuffer checked_mul

Path:

```text
arrow_buffer::ScalarBuffer::new
  -> offset.checked_mul(size).expect(...)
  -> len.checked_mul(size).expect(...)
```

Replacement:

- Safe: `checked_mul(...).expect(...)`
- Unsafe: `unchecked_mul(...)`

Validation:

- Proptest differential testing passed.
- Same `Buffer`, `offset`, and `len` were used to compare typed slice contents.

| Workload | Safe Time | Unsafe Time | Safe/Unsafe | Interpretation |
|---|---:|---:|---:|---|
| 16K slices | 250.03 us | 244.98 us | 1.021x | Weak positive |
| 65K slices | 1.0309 ms | 1.0233 ms | 1.007x | Weak |
| 262K slices | 4.2940 ms | 4.1849 ms | 1.026x | Weak positive |

Comment:

This is non-string and semantically clean, but the improvement is only about
1-3%, so it is not strong enough to replace the ring/ripgrep candidates.

## Target-Function Optimization-Level Sensitivity

Source file:

- `evaluation/results/opt_level_target_function_summary.md`

These data measure safe/unsafe target-function-level benchmarks at O1, O2, and
O3 for five crate targets.

| Crate | Opt | Safe Mean | Unsafe Mean | Safe/Unsafe | Speedup |
|---|---:|---:|---:|---:|---:|
| arrayvec | O1 | 1.8238 us | 2.1072 us | 0.8655x | -13.45% |
| arrayvec | O2 | 2.2430 us | 2.2064 us | 1.0166x | +1.66% |
| arrayvec | O3 | 2.1327 us | 2.2799 us | 0.9354x | -6.46% |
| bit-vec | O1 | 199.9500 us | 204.1800 us | 0.9793x | -2.07% |
| bit-vec | O2 | 184.9800 us | 180.1900 us | 1.0266x | +2.66% |
| bit-vec | O3 | 185.1400 us | 176.9200 us | 1.0465x | +4.65% |
| itertools | O1 | 1.2477 ms | 1.3393 ms | 0.9316x | -6.84% |
| itertools | O2 | 1.2796 ms | 1.3036 ms | 0.9816x | -1.84% |
| itertools | O3 | 1.3359 ms | 1.3105 ms | 1.0194x | +1.94% |
| rand | O1 | 32.0910 us | 31.8600 us | 1.0073x | +0.73% |
| rand | O2 | 11.4060 us | 9.8726 us | 1.1553x | +15.53% |
| rand | O3 | 9.7637 us | 9.8401 us | 0.9922x | -0.78% |
| ring | O1 | 18.2860 us | 17.9110 us | 1.0209x | +2.09% |
| ring | O2 | 12.8840 us | 12.4910 us | 1.0315x | +3.15% |
| ring | O3 | 12.6880 us | 12.8020 us | 0.9911x | -0.89% |

Interpretation:

- The target-function-level results are not monotonic across O1, O2, and O3.
- Local safe/unsafe replacement effects are often small enough to be affected
  by surrounding code generation and measurement noise.
- O3 is still a defensible main setting because it is Cargo's default release
  profile and represents the most common performance-oriented release
  configuration.
- These data should not be used to claim that higher optimization always
  increases or decreases the replacement benefit.

## Size-Oriented Optimization Levels

Source file:

- `evaluation/results/rand_size_opt_summary.md`

Only `rand` was measured for `Os` and `Oz` in this summary.

| Crate | Opt | Safe Mean | Unsafe Mean | Safe/Unsafe | Speedup |
|---|---:|---:|---:|---:|---:|
| rand | Os | 7.8598 us | 7.9128 us | 0.9933x | -0.67% |
| rand | Oz | 11.6100 us | 10.3410 us | 1.1227x | +12.27% |

Interpretation:

- `Os` and `Oz` are size-oriented configurations, not the main runtime
  performance setting.
- Their behavior can differ from O1/O2/O3, so they are better discussed as
  sensitivity checks rather than primary evaluation settings.

## API-Level Optimization Sensitivity

Source file:

- `API-counterprats/bench-results/selected_bench_overview_opt_sensitivity_all.csv`

These data measure selected standard-library API pairs directly across O1, O2,
O3, Os, and Oz. They are API-level measurements, not target-function-level or
application-derived measurements.

| API Pair | O1 Ratio | O2 Ratio | O3 Ratio | Os Ratio | Oz Ratio |
|---|---:|---:|---:|---:|---:|
| `str::from_utf8` | 9.413x | 10.900x | 9.586x | 9.979x | 10.921x |
| `CStr::from_bytes_with_nul` | 9.575x | 8.806x | 9.988x | 10.110x | 6.385x |
| `u32::shl_exact` | 1.723x | 1.954x | 1.922x | 1.894x | 1.751x |
| `i32::shl_exact` | 1.689x | 1.970x | 1.972x | 1.982x | 1.803x |
| `u32::shr_exact` | 1.239x | 1.289x | 1.359x | 1.731x | 1.659x |
| `i32::checked_neg` | 1.050x | 1.024x | 1.242x | 1.176x | 1.125x |
| `slice::as_ascii` | 2.847x | 2.834x | 2.243x | 2.211x | 3.102x |
| `char::from_u32` | 1.500x | 1.494x | 1.408x | 1.408x | 1.573x |
| `Alignment::new` | 1.668x | 1.663x | 1.571x | 1.604x | 1.418x |
| `Arc::get_mut` | 1.276x | 2.039x | 2.065x | 1.289x | 1.217x |
| `Rc::get_mut` | 1.114x | 1.087x | 1.107x | 1.098x | 1.053x |
| `Pin::get_unchecked_mut` | 1.018x | 1.174x | 0.979x | 0.989x | 0.975x |
| `slice::split_at` | 1.158x | 1.055x | 1.046x | 1.059x | 0.965x |
| `slice::swap` | 1.030x | 0.999x | 1.000x | 1.000x | 1.038x |
| `Option::unwrap` | 0.886x | 0.960x | 1.051x | 1.028x | 1.027x |
| `Result::unwrap_err` | 1.164x | 0.982x | 1.036x | 0.996x | 1.026x |
| `BTreeMap::insert_after` | 1.026x | 1.014x | 1.009x | 1.020x | 1.068x |
| `BTreeSet::insert_before` | 1.000x | 0.851x | 0.979x | 1.019x | 1.058x |
| `Layout::from_size_align` | 1.542x | 1.115x | 1.118x | 1.483x | 1.431x |
| `Layout::from_size_alignment` | 0.992x | 0.991x | 1.024x | 1.059x | 1.008x |

Interpretation:

- API-level trends remain highly API-dependent across optimization levels.
- Some APIs show robust gains across all optimization levels, especially
  validation-heavy APIs such as `str::from_utf8`.
- Some APIs remain near parity across all levels, e.g. `slice::swap`.
- These API-level results are useful for sensitivity discussion, but they do
  not directly prove target-function or application-level speedups.

## Conservative Paper Takeaways

Potential wording direction:

```text
We use Cargo's default release profile as the main configuration because it is
the standard performance-oriented setting for Rust release builds. To evaluate
sensitivity to optimization levels, we additionally measured selected API-level
and target-function-level benchmarks under O1, O2, and O3, with a small
size-optimization check under Os/Oz. The results do not show a monotonic trend:
the relative benefit depends on the API semantics, how much code surrounds the
replacement, and whether the replaced check lies on a hot execution path. This
supports our choice to report the main results under the standard release
configuration while treating optimization-level differences as a sensitivity
factor rather than a separate optimization claim.
```
