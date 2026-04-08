#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};

const OPS: usize = 1 << 24;
const LEN: usize = 1 << 20;
const MASK: usize = LEN - 1;


fn make_inputs() -> Vec<Option<u64>> {
    (0..LEN as u64).map(Some).collect()
}

fn bench_candidate_core_option_unwrap_unchecked_1128(c: &mut Criterion) {
    let inputs = make_inputs();
    let param = format!("len{}_ops{}", LEN, OPS);
    let mut g = c.benchmark_group("candidate_core_option_unwrap_unchecked_1128");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));
    g.bench_with_input(BenchmarkId::new("safe", &param), &inputs, |b, inputs| {
        b.iter(|| {
            let ins = black_box(&inputs[..]);
            for t in 0..OPS {
                let v = ins[t & MASK].unwrap();
                black_box(v);
            }
        })
    });
    g.bench_with_input(BenchmarkId::new("unsafe", &param), &inputs, |b, inputs| {
        b.iter(|| {
            let ins = black_box(&inputs[..]);
            for t in 0..OPS {
                let v = unsafe { ins[t & MASK].unwrap_unchecked() };
                black_box(v);
            }
        })
    });
    g.finish();
}

criterion_group!(benches, bench_candidate_core_option_unwrap_unchecked_1128);
criterion_main!(benches);
