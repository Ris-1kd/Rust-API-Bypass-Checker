#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};
use std::num::NonZeroU64;

const OPS: usize = 1 << 23;
const IN_LEN: usize = 1 << 14;
const IN_MASK: usize = IN_LEN - 1;

fn make_inputs(n: usize, seed: u64) -> Vec<u64> {
    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        out.push(((x >> 2) & 0xffff_ffff) + 1);
    }
    out
}

fn bench_candidate_core_num_nonzero_from_mut_unchecked_460(c: &mut Criterion) {
    let mut inputs = make_inputs(IN_LEN, 0xdead_beef_cafe_f00d);
    let param = format!("in{}_ops{}", IN_LEN, OPS);
    let mut g = c.benchmark_group("candidate_core_num_nonzero_from_mut_unchecked_460");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));

    g.bench_function(BenchmarkId::new("safe_from_mut", &param), |b| {
        b.iter(|| {
            let vals = black_box(&mut inputs[..]);
            for t in 0..OPS {
                let v = unsafe { vals.get_unchecked_mut(t & IN_MASK) };
                let nz = NonZeroU64::from_mut(v).unwrap();
                black_box(nz.get());
            }
        })
    });

    g.bench_function(BenchmarkId::new("unsafe_from_mut_unchecked", &param), |b| {
        b.iter(|| {
            let vals = black_box(&mut inputs[..]);
            for t in 0..OPS {
                let v = unsafe { vals.get_unchecked_mut(t & IN_MASK) };
                let nz = unsafe { NonZeroU64::from_mut_unchecked(v) };
                black_box(nz.get());
            }
        })
    });

    g.finish();
}

criterion_group!(benches, bench_candidate_core_num_nonzero_from_mut_unchecked_460);
criterion_main!(benches);
