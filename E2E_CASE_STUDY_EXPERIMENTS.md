# E2E Case Study Candidate Experiments

This note records the application-level case study candidates evaluated during
the revision. The goal was to find a non-string workload where a safe API call
can be replaced with its unsafe counterpart, while preserving semantics and
showing a measurable application-level effect.

All experiments were placed under `e2e/`, which is intentionally ignored by
Git. The code is therefore treated as temporary experimental material, while
this document records the setup and observed results.

## 1. RustScan Port Planning

### Motivation

RustScan is a real Rust port scanner. Its port planning logic contains a short
application-level path:

```text
PortStrategy::pick(..., ScanOrder::Random)
  -> ports.shuffle(&mut rng)
```

This avoids network I/O by isolating the planning stage, while still using a
real project component.

### Setup

The experiment modeled RustScan's manual port-list randomization component:

```text
manual port list -> Fisher-Yates shuffle -> randomized scan order
```

Two versions were compared:

```text
safe:   slice.swap(i, j)
unsafe: ptr::swap(slice.as_mut_ptr().add(i), slice.as_mut_ptr().add(j))
```

The unsafe version uses the same generated indices as the safe version. The
precondition is that both `i` and `j` are valid indices, which follows from
`j in 0..=i` and the reverse iteration over valid slice indices.

### Validation

Property-based differential testing with Proptest was used. The test generates
random port vectors and RNG seeds, then checks that the safe and unsafe
planning results are identical.

Result:

```text
cargo test --test differential
safe_and_unchecked_planning_match ... ok
```

### Performance

Criterion benchmark results:

| Port Count | Safe Time | Unsafe Time | Safe/Unsafe |
|---:|---:|---:|---:|
| 1,024 | 5.1854 us | 5.2583 us | 0.986x |
| 8,192 | 43.407 us | 45.454 us | 0.955x |
| 65,535 | 358.13 us | 361.67 us | 0.990x |

### Assessment

This is a real and short application-level path, but it is not a good positive
case. The unsafe version is slightly slower. The likely reason is that the
compiler can already prove the indices safe in the simple Fisher-Yates loop,
so replacing `swap` with pointer-based swapping does not remove meaningful
runtime overhead.

## 2. Arti Timeout Estimator

### Motivation

Arti/Tor's circuit timeout estimator provides a more semantically meaningful
application-level path. The estimator computes timeout parameters from circuit
build-time history. In particular, it estimates the Pareto parameter `Xm` from
the most frequent histogram bins.

The relevant path is:

```text
circuit build-time history
  -> sparse histogram bins
  -> k_smallest(n_modes)
  -> itertools::k_smallest.rs::sift_down
  -> heap.swap(...)
```

This is more natural than forcing the complete circuit construction path,
because full circuit construction includes asynchronous runtime behavior,
network effects, and unrelated protocol work.

### Setup

The experiment implemented an Arti-style timeout-estimator component:

```text
synthetic circuit build times
  -> sparse histogram
  -> select top modes via itertools::k_smallest
  -> compute weighted Xm
```

Two vendored copies of `itertools` were used:

```text
safe:   k_smallest.rs::sift_down uses heap.swap(...)
unsafe: k_smallest.rs::sift_down uses ptr::swap(...)
```

### Validation

Property-based differential testing with Proptest was used. The test generates
random build-time samples and mode counts, then checks that the safe and unsafe
estimators compute the same `Xm`.

Result:

```text
cargo test --test differential
safe_and_unchecked_k_smallest_timeout_estimator_match ... ok
```

### Performance

Criterion benchmark results:

| Workload | Safe Time | Unsafe Time | Safe/Unsafe |
|---|---:|---:|---:|
| 1,000 samples, 10 modes | 6.7796 us | 6.8428 us | 0.991x |
| 8,000 samples, 10 modes | 51.792 us | 52.577 us | 0.985x |
| 65,536 samples, 10 modes | 422.55 us | 420.99 us | 1.004x |
| 65,536 samples, 64 modes | 420.75 us | 420.84 us | 1.000x |
| Wide 65,536 samples, 64 modes | 483.70 us | 477.59 us | 1.013x |

### Assessment

Arti is semantically the strongest case among the tested candidates. It gives
a realistic application-level interpretation for a heap-maintenance safe API
replacement. However, the performance benefit is small, with the best observed
case around 1.3%. The reason is that `sift_down` and its `swap` operation are
only a small part of the estimator; histogram construction and iterator logic
dominate the component cost.

## 3. Ring BigInt Buffer Splitting

### Motivation

The `ring` crate contains several buffer-partitioning paths based on
`split_at_mut`. Unlike string validation, this is still within the non-string
API scope and corresponds to a common low-level pattern: splitting a known
layout buffer into logically separate regions.

The selected pattern is adapted from `ring/src/arithmetic/bigint/exp.rs`:

```text
state buffer
  -> split_at_mut(m_len)
  -> split_at_mut(m_len)
  -> acc / base_cached / m_cached regions
```

This is not a full cryptographic operation. It isolates a buffer-processing
component so that the effect of the safe/unsafe replacement is not hidden by
large cryptographic primitive costs.

### Setup

The experiment compares:

```text
safe:   split_at_mut(m_len)
unsafe: from_raw_parts_mut(ptr, mid) + from_raw_parts_mut(ptr.add(mid), len - mid)
```

The unsafe precondition is that `m_len` partitions each record into valid
non-overlapping regions. The benchmark processes repeated records with the
layout:

```text
[acc | base_cached | m_cached]
```

### Validation

Property-based differential testing with Proptest was used. The test generates
different region sizes and record counts, then checks both the checksum and the
final mutated buffer state.

Result:

```text
cargo test --offline --test differential
safe_and_unchecked_split_processing_match ... ok
```

### Performance

Criterion benchmark results:

| m_len | Safe Time | Unsafe Time | Safe/Unsafe |
|---:|---:|---:|---:|
| 1 | 21.412 us | 21.205 us | 1.010x |
| 2 | 41.270 us | 41.582 us | 0.993x |
| 4 | 80.020 us | 79.156 us | 1.011x |
| 8 | 144.59 us | 146.41 us | 0.988x |
| 16 | 278.66 us | 274.86 us | 1.014x |
| 32 | 255.38 us | 245.88 us | 1.039x |

### Assessment

This is the strongest positive candidate found so far. The best observed case
shows about 3.9% speedup. The result is still below the desired 5% threshold,
and the larger case showed noticeable outliers, so it should be presented
conservatively. Compared with Arti, this candidate has a stronger performance
signal but a weaker end-to-end application story because it is an isolated
buffer-processing component adapted from `ring`.

## Overall Recommendation

The candidates are ranked as follows:

| Candidate | Semantic Strength | Performance Signal | Recommendation |
|---|---|---|---|
| Arti timeout estimator | Strong | Weak, around 1% | Best semantic case |
| Ring buffer split | Medium | Best, up to about 3.9% | Best positive performance case |
| RustScan planning | Medium | Negative | Do not use as positive case |

For the paper, the most defensible strategy is:

```text
Use Arti if the section prioritizes realistic application semantics.
Use ring-buffer-split if the section needs a conservative positive performance case.
Avoid claiming large end-to-end acceleration in either case.
```

Suggested wording for the ring case:

```text
We further evaluate a buffer-processing component adapted from ring's BigInt
modular exponentiation implementation. The component repeatedly partitions a
known-layout working buffer into disjoint regions using split_at_mut. We replace
these checked splits with unchecked slice construction after validating the
layout precondition. Property-based differential tests confirm identical output
states. The transformed component shows a modest speedup, up to 3.9% in our
controlled workload, indicating that application-level benefits are possible
but remain workload-dependent.
```
