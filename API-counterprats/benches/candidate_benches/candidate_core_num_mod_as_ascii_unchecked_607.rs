#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};
use std::ascii;

const OPS: usize = 1 << 24;
const IN_LEN: usize = 1 << 20;
const IN_MASK: usize = IN_LEN - 1;


fn make_inputs(n: usize, seed: u64) -> Vec<u8> {
    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        out.push(((x >> 32) as u8) & 0x7f);
    }
    out
}

fn bench_candidate_core_num_mod_as_ascii_unchecked_607(c: &mut Criterion) {
    let inputs = make_inputs(IN_LEN, 0x9abc_def0_1234_5678);
    let param = format!("in{}_ops{}", IN_LEN, OPS);
    let mut g = c.benchmark_group("candidate_core_num_mod_as_ascii_unchecked_607");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));

    g.bench_with_input(BenchmarkId::new("safe_as_ascii", &param), &inputs, |b, inputs| {
        b.iter(|| {
            let ins = black_box(&inputs[..]);
            for t in 0..OPS {
                let v = black_box(ins[t & IN_MASK]);
                let ch = v.as_ascii().unwrap();
                black_box(ch);
            }
        })
    });

    g.bench_with_input(BenchmarkId::new("unsafe_as_ascii_unchecked", &param), &inputs, |b, inputs| {
        b.iter(|| {
            let ins = black_box(&inputs[..]);
            for t in 0..OPS {
                let v = black_box(ins[t & IN_MASK]);
                let ch = unsafe { v.as_ascii_unchecked() };
                black_box(ch);
            }
        })
    });

    g.finish();
}

criterion_group!(benches, bench_candidate_core_num_mod_as_ascii_unchecked_607);
criterion_main!(benches);
