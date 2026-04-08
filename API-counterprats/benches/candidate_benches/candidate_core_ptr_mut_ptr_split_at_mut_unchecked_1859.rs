#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};

const LEN: usize = 4096;
const OPS: usize = 1 << 24;
const MID_LEN: usize = 1 << 20;
const MID_MASK: usize = MID_LEN - 1;

fn make_mids(len: usize, n: usize, seed: u64) -> Vec<usize> {
    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        out.push(((x as usize) & (len - 2)) + 1);
    }
    out
}

fn bench_candidate_core_ptr_mut_ptr_split_at_mut_unchecked_1859(c: &mut Criterion) {
    let mids = make_mids(LEN, MID_LEN, 0xfeed_face_cafe_babe);
    let mut data: Vec<u64> = (0..LEN as u64).collect();
    let param = format!("len{}_ops{}", LEN, OPS);
    let mut g = c.benchmark_group("candidate_core_ptr_mut_ptr_split_at_mut_unchecked_1859");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));

    g.bench_with_input(BenchmarkId::new("checked_split_at_mut", &param), &mids, |b, mids| {
        b.iter(|| {
            let ptr: *mut [u64] = black_box(&mut data[..] as *mut [u64]);
            let mids = black_box(&mids[..]);
            for t in 0..OPS {
                let mid = black_box(mids[t & MID_MASK]);
                let (l, r) = unsafe { ptr.split_at_mut(mid) };
                black_box((l as *mut () as usize as u64) ^ (r as *mut () as usize as u64));
            }
        })
    });

    g.bench_with_input(BenchmarkId::new("unchecked_split_at_mut", &param), &mids, |b, mids| {
        b.iter(|| {
            let ptr: *mut [u64] = black_box(&mut data[..] as *mut [u64]);
            let mids = black_box(&mids[..]);
            for t in 0..OPS {
                let mid = black_box(mids[t & MID_MASK]);
                let (l, r) = unsafe { ptr.split_at_mut_unchecked(mid) };
                black_box((l as *mut () as usize as u64) ^ (r as *mut () as usize as u64));
            }
        })
    });

    g.finish();
}

criterion_group!(benches, bench_candidate_core_ptr_mut_ptr_split_at_mut_unchecked_1859);
criterion_main!(benches);
