#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};

const LEN: usize = 4096;
const PAIRS_LEN: usize = 1 << 12;
const ROUNDS: usize = 1 << 12;

fn make_pairs(len: usize, n: usize, seed: u64) -> Vec<(usize, usize)> {
    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let i = (x as usize) % len;
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut j = (x as usize) % len;
        if i == j { j = (j + 1) % len; }
        out.push((i, j));
    }
    out
}

fn bench_candidate_core_slice_mod_swap_unchecked_948(c: &mut Criterion) {
    let base: Vec<u64> = (0..LEN as u64).collect();
    let pairs = make_pairs(LEN, PAIRS_LEN, 0x1234_5678_9abc_def0);
    let total = (PAIRS_LEN * ROUNDS) as u64;
    let param = format!("len{}_ops{}", LEN, total);
    let mut g = c.benchmark_group("candidate_core_slice_mod_swap_unchecked_948");
    g.throughput(Throughput::Elements(total));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));
    g.bench_with_input(BenchmarkId::new("safe_swap", &param), &pairs, |b, pairs| {
        b.iter(|| {
            let mut data = base.clone();
            let s = black_box(data.as_mut_slice());
            for _ in 0..ROUNDS {
                for &(i, j) in pairs.iter() {
                    s.swap(i, j);
                }
            }
            black_box(s[0] ^ s[LEN / 2] ^ s[LEN - 1]);
        })
    });
    g.bench_with_input(BenchmarkId::new("unsafe_swap_unchecked", &param), &pairs, |b, pairs| {
        b.iter(|| {
            let mut data = base.clone();
            let s = black_box(data.as_mut_slice());
            for _ in 0..ROUNDS {
                for &(i, j) in pairs.iter() {
                    unsafe { s.swap_unchecked(i, j) };
                }
            }
            black_box(s[0] ^ s[LEN / 2] ^ s[LEN - 1]);
        })
    });
    g.finish();
}

criterion_group!(benches, bench_candidate_core_slice_mod_swap_unchecked_948);
criterion_main!(benches);
