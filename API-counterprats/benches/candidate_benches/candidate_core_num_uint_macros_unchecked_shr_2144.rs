#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};

type T = u64;
const OPS: usize = 1 << 24;
const IN_LEN: usize = 1 << 20;
const IN_MASK: usize = IN_LEN - 1;


fn make_inputs(n: usize, seed: u64) -> (Vec<T>, Vec<u32>) {
    let mut x = seed;
    let mut xs = Vec::with_capacity(n);
    let mut rs = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let r = ((x as u32) % 16) + 1;
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let v = (((x >> 32) as T) & 0x3fff_ffff) + 1;
        xs.push(v);
        rs.push(r);
    }
    (xs, rs)
}

fn bench_candidate_core_num_uint_macros_unchecked_shr_2144(c: &mut Criterion) {
    let (xs, rs) = make_inputs(IN_LEN, 0xfeed_face_cafe_babe);
    let param = format!("in{}_ops{}", IN_LEN, OPS);
    let mut g = c.benchmark_group("candidate_core_num_uint_macros_unchecked_shr_2144");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));
    g.bench_with_input(BenchmarkId::new("safe_checked_shr", &param), &(&xs, &rs), |b, (xs, rs)| {
        b.iter(|| {
            let xs = black_box(&xs[..]);
            let rs = black_box(&rs[..]);
            for t in 0..OPS {
                let x = black_box(xs[t & IN_MASK]);
                let r = black_box(rs[t & IN_MASK]);
                let v = x.checked_shr(r).unwrap();
                black_box(v);
            }
        })
    });
    g.bench_with_input(BenchmarkId::new("unsafe_unchecked_shr", &param), &(&xs, &rs), |b, (xs, rs)| {
        b.iter(|| {
            let xs = black_box(&xs[..]);
            let rs = black_box(&rs[..]);
            for t in 0..OPS {
                let x = black_box(xs[t & IN_MASK]);
                let r = black_box(rs[t & IN_MASK]);
                let v = unsafe { x.unchecked_shr(r) };
                black_box(v);
            }
        })
    });
    g.finish();
}

criterion_group!(benches, bench_candidate_core_num_uint_macros_unchecked_shr_2144);
criterion_main!(benches);
