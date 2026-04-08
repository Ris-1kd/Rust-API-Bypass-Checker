#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};
use std::ptr::NonNull;

const OPS: usize = 1 << 24;
const LEN: usize = 1 << 12;
const MASK: usize = LEN - 1;

fn bench_candidate_core_ptr_non_null_new_unchecked_233(c: &mut Criterion) {
    let mut data: Vec<u64> = (0..LEN as u64).collect();
    let ptrs: Vec<*mut u64> = data.iter_mut().map(|v| v as *mut u64).collect();
    let param = format!("len{}_ops{}", LEN, OPS);
    let mut g = c.benchmark_group("candidate_core_ptr_non_null_new_unchecked_233");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));

    g.bench_with_input(BenchmarkId::new("safe_new", &param), &ptrs, |b, ptrs| {
        b.iter(|| {
            let ps = black_box(&ptrs[..]);
            for t in 0..OPS {
                let p = black_box(ps[t & MASK]);
                let nn = NonNull::new(p).unwrap();
                black_box(nn);
            }
        })
    });

    g.bench_with_input(BenchmarkId::new("unsafe_new_unchecked", &param), &ptrs, |b, ptrs| {
        b.iter(|| {
            let ps = black_box(&ptrs[..]);
            for t in 0..OPS {
                let p = black_box(ps[t & MASK]);
                let nn = unsafe { NonNull::new_unchecked(p) };
                black_box(nn);
            }
        })
    });

    g.finish();
}

criterion_group!(benches, bench_candidate_core_ptr_non_null_new_unchecked_233);
criterion_main!(benches);
