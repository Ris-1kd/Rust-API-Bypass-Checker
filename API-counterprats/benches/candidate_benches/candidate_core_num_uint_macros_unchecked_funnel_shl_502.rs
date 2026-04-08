#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};
use std::intrinsics::fallback::FunnelShift;

type T = u64;
const OPS: usize = 1 << 24;
const IN_LEN: usize = 1 << 20;
const IN_MASK: usize = IN_LEN - 1;

fn make_inputs(n: usize, seed: u64) -> (Vec<T>, Vec<T>, Vec<u32>) {
    let mut x = seed;
    let mut a = Vec::with_capacity(n);
    let mut b = Vec::with_capacity(n);
    let mut rs = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        a.push(x as u64);
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        b.push(x as u64);
        rs.push(((x >> 32) as u32 % 63) + 1);
    }
    (a, b, rs)
}

fn bench_candidate_core_num_uint_macros_unchecked_funnel_shl_502(c: &mut Criterion) {
    let (a, b, rs) = make_inputs(IN_LEN, 0x1234_5678_9abc_def0);
    let param = format!("in{}_ops{}", IN_LEN, OPS);
    let mut g = c.benchmark_group("candidate_core_num_uint_macros_unchecked_funnel_shl_502");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));
    g.bench_with_input(BenchmarkId::new("safe_funnel_shl", &param), &(&a, &b, &rs), |bch, (a, b, rs)| {
        bch.iter(|| {
            let a = black_box(&a[..]);
            let b = black_box(&b[..]);
            let rs = black_box(&rs[..]);
            for t in 0..OPS {
                let x = black_box(a[t & IN_MASK]);
                let y = black_box(b[t & IN_MASK]);
                let r = black_box(rs[t & IN_MASK]);
                let v = x.funnel_shl(y, r);
                black_box(v);
            }
        })
    });
    g.bench_with_input(BenchmarkId::new("unsafe_unchecked_funnel_shl", &param), &(&a, &b, &rs), |bch, (a, b, rs)| {
        bch.iter(|| {
            let a = black_box(&a[..]);
            let b = black_box(&b[..]);
            let rs = black_box(&rs[..]);
            for t in 0..OPS {
                let x = black_box(a[t & IN_MASK]);
                let y = black_box(b[t & IN_MASK]);
                let r = black_box(rs[t & IN_MASK]);
                let v = unsafe { x.unchecked_funnel_shl(y, r) };
                black_box(v);
            }
        })
    });
    g.finish();
}

criterion_group!(benches, bench_candidate_core_num_uint_macros_unchecked_funnel_shl_502);
criterion_main!(benches);
