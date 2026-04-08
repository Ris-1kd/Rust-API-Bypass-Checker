#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};
use std::num::NonZeroU64;

const OPS: usize = 1 << 24;
const IN_LEN: usize = 1 << 20;
const IN_MASK: usize = IN_LEN - 1;

fn make_inputs(n: usize, seed: u64) -> Vec<u64> {
    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        out.push(((x >> 1) & 0xffff_ffff) + 1);
    }
    out
}

fn bench_candidate_core_num_nonzero_new_unchecked_419(c: &mut Criterion) {
    let inputs = make_inputs(IN_LEN, 0x1234_5678_9abc_def0);
    let param = format!("in{}_ops{}", IN_LEN, OPS);
    let mut g = c.benchmark_group("candidate_core_num_nonzero_new_unchecked_419");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));
    g.bench_with_input(BenchmarkId::new("safe_new", &param), &inputs, |b, inputs| {
        b.iter(|| {
            let ins = black_box(&inputs[..]);
            for t in 0..OPS {
                let v = black_box(ins[t & IN_MASK]);
                let nz = NonZeroU64::new(v).unwrap();
                black_box(nz.get());
            }
        })
    });
    g.bench_with_input(BenchmarkId::new("unsafe_new_unchecked", &param), &inputs, |b, inputs| {
        b.iter(|| {
            let ins = black_box(&inputs[..]);
            for t in 0..OPS {
                let v = black_box(ins[t & IN_MASK]);
                let nz = unsafe { NonZeroU64::new_unchecked(v) };
                black_box(nz.get());
            }
        })
    });
    g.finish();
}

criterion_group!(benches, bench_candidate_core_num_nonzero_new_unchecked_419);
criterion_main!(benches);
