#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};

const OPS: usize = 1 << 24;
const IN_LEN: usize = 1 << 20;
const IN_MASK: usize = IN_LEN - 1;

fn make_valid_scalar_u32s(n: usize, seed: u64) -> Vec<u32> {
    const RANGE: u32 = 0x10F800;
    const SURROGATE_START: u32 = 0xD800;
    const SURROGATE_GAP: u32 = 0x800;
    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut v = (x as u32) % RANGE;
        if v >= SURROGATE_START {
            v += SURROGATE_GAP;
        }
        out.push(v);
    }
    out
}

fn bench_candidate_core_char_mod_from_u32_unchecked_141(c: &mut Criterion) {
    let inputs = make_valid_scalar_u32s(IN_LEN, 0x1234_5678_9abc_def0);
    let param = format!("in{}_ops{}", IN_LEN, OPS);
    let mut g = c.benchmark_group("candidate_core_char_mod_from_u32_unchecked_141");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));

    g.bench_with_input(BenchmarkId::new("safe_from_u32", &param), &inputs, |b, inputs| {
        b.iter(|| {
            let ins: &[u32] = black_box(&inputs[..]);
            for t in 0..OPS {
                let v = black_box(ins[t & IN_MASK]);
                let c = char::from_u32(v).unwrap();
                black_box(c);
            }
        })
    });

    g.bench_with_input(BenchmarkId::new("unsafe_from_u32_unchecked", &param), &inputs, |b, inputs| {
        b.iter(|| {
            let ins: &[u32] = black_box(&inputs[..]);
            for t in 0..OPS {
                let v = black_box(ins[t & IN_MASK]);
                let c = unsafe { char::from_u32_unchecked(v) };
                black_box(c);
            }
        })
    });

    g.finish();
}

criterion_group!(benches, bench_candidate_core_char_mod_from_u32_unchecked_141);
criterion_main!(benches);
