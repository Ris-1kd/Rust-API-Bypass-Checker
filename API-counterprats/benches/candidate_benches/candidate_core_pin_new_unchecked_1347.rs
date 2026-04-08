#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};

const OPS: usize = 1 << 24;
const LEN: usize = 1 << 14;
const MASK: usize = LEN - 1;

fn bench_candidate_core_pin_new_unchecked_1347(c: &mut Criterion) {
    let mut safe_vals: Vec<u64> = (0..LEN as u64).collect();
    let mut unsafe_vals: Vec<u64> = (0..LEN as u64).collect();
    let param = format!("len{}_ops{}", LEN, OPS);
    let mut g = c.benchmark_group("candidate_core_pin_new_unchecked_1347");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));
    g.bench_function(BenchmarkId::new("safe", &param), |b| {
        b.iter(|| {
            let vals = black_box(&mut safe_vals[..]);
            for t in 0..OPS {
                let v = unsafe { vals.get_unchecked_mut(t & MASK) };
                let p = std::pin::Pin::new(v); black_box(p);
            }
        })
    });
    g.bench_function(BenchmarkId::new("unsafe", &param), |b| {
        b.iter(|| {
            let vals = black_box(&mut unsafe_vals[..]);
            for t in 0..OPS {
                let v = unsafe { vals.get_unchecked_mut(t & MASK) };
                let p = unsafe { std::pin::Pin::new_unchecked(v) }; black_box(p);
            }
        })
    });
    g.finish();
}

criterion_group!(benches, bench_candidate_core_pin_new_unchecked_1347);
criterion_main!(benches);
