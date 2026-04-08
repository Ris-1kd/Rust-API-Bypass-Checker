#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};

type T = u64;
const OPS: usize = 1 << 24;
const IN_LEN: usize = 1 << 20;
const IN_MASK: usize = IN_LEN - 1;


fn make_inputs(n: usize, seed: u64) -> (Vec<T>, Vec<T>) {
    let mut x = seed;
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let a = ((x >> 32) as T) & 0x3fff_ffff;
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let b = ((x >> 32) as T) & 0x3fff_ffff;
        xs.push(a);
        ys.push(b);
    }
    (xs, ys)
}

fn bench_candidate_core_num_uint_macros_unchecked_add_886(c: &mut Criterion) {
    let (xs, ys) = make_inputs(IN_LEN, 0x1234_5678_9abc_def0);
    let param = format!("in{}_ops{}", IN_LEN, OPS);
    let mut g = c.benchmark_group("candidate_core_num_uint_macros_unchecked_add_886");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));
    g.bench_with_input(BenchmarkId::new("safe_checked_add", &param), &(&xs, &ys), |b, (xs, ys)| {
        b.iter(|| {
            let xs = black_box(&xs[..]);
            let ys = black_box(&ys[..]);
            for t in 0..OPS {
                let x = black_box(xs[t & IN_MASK]);
                let y = black_box(ys[t & IN_MASK]);
                let v = x.checked_add(y).unwrap();
                black_box(v);
            }
        })
    });
    g.bench_with_input(BenchmarkId::new("unsafe_unchecked_add", &param), &(&xs, &ys), |b, (xs, ys)| {
        b.iter(|| {
            let xs = black_box(&xs[..]);
            let ys = black_box(&ys[..]);
            for t in 0..OPS {
                let x = black_box(xs[t & IN_MASK]);
                let y = black_box(ys[t & IN_MASK]);
                let v = unsafe { x.unchecked_add(y) };
                black_box(v);
            }
        })
    });
    g.finish();
}

criterion_group!(benches, bench_candidate_core_num_uint_macros_unchecked_add_886);
criterion_main!(benches);
