#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};

const OPS: usize = 1 << 22;
const IN_LEN: usize = 1 << 14;
const IN_MASK: usize = IN_LEN - 1;


const WIDTH: usize = 64;
type Input = [u8; WIDTH];

fn make_inputs(n: usize) -> Vec<Input> {
    let alphabet = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let mut item = [0u8; WIDTH];
        for (j, b) in item.iter_mut().enumerate() {
            *b = alphabet[(i + j) % alphabet.len()];
        }
        out.push(item);
    }
    out
}

fn bench_candidate_core_str_converts_from_utf8_unchecked_mut_208(c: &mut Criterion) {
    let mut inputs = make_inputs(IN_LEN);
    let param = format!("in{}_ops{}", IN_LEN, OPS);
    let mut g = c.benchmark_group("candidate_core_str_converts_from_utf8_unchecked_mut_208");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));

    g.bench_function(BenchmarkId::new("safe_from_utf8", &param), |b| {
        b.iter(|| {
            let ins = black_box(&mut inputs[..]);
            for t in 0..OPS {
                let inp = unsafe { ins.get_unchecked_mut(t & IN_MASK) };
                let s = std::str::from_utf8_mut(&mut inp[..]).unwrap();
                black_box((s.as_ptr() as usize as u64) ^ (s.len() as u64));
            }
        })
    });

    g.bench_function(BenchmarkId::new("unsafe_from_utf8_unchecked", &param), |b| {
        b.iter(|| {
            let ins = black_box(&mut inputs[..]);
            for t in 0..OPS {
                let inp = unsafe { ins.get_unchecked_mut(t & IN_MASK) };
                let s = unsafe { std::str::from_utf8_unchecked_mut(&mut inp[..]) };
                black_box((s.as_ptr() as usize as u64) ^ (s.len() as u64));
            }
        })
    });

    g.finish();
}

criterion_group!(benches, bench_candidate_core_str_converts_from_utf8_unchecked_mut_208);
criterion_main!(benches);
