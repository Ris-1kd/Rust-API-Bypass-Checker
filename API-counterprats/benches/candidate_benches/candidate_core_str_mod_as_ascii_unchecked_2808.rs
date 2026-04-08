#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};
use std::ascii;

const OPS: usize = 1 << 22;
const IN_LEN: usize = 1 << 14;
const IN_MASK: usize = IN_LEN - 1;


const WIDTH: usize = 64;
type Input = [u8; WIDTH];

fn make_inputs(n: usize, seed: u64) -> Vec<Input> {
    let alphabet = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";
    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let mut item = [0u8; WIDTH];
        for b in &mut item {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
            *b = alphabet[(x as usize) % alphabet.len()];
        }
        out.push(item);
    }
    out
}


fn bench_candidate_core_str_mod_as_ascii_unchecked_2808(c: &mut Criterion) {
    let inputs = make_inputs(IN_LEN, 0x1234_5678_9abc_def0);
    let param = format!("in{}_ops{}", IN_LEN, OPS);
    let mut g = c.benchmark_group("candidate_core_str_mod_as_ascii_unchecked_2808");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));

    g.bench_with_input(BenchmarkId::new("safe_as_ascii", &param), &inputs, |b, inputs| {
        b.iter(|| {
            let ins = black_box(&inputs[..]);
            for t in 0..OPS {
                let inp = black_box(&ins[t & IN_MASK]);
                let s = std::str::from_utf8(inp).unwrap(); let chars = s.as_ascii().unwrap();
                black_box(chars.as_ptr());
            }
        })
    });

    g.bench_with_input(BenchmarkId::new("unsafe_as_ascii_unchecked", &param), &inputs, |b, inputs| {
        b.iter(|| {
            let ins = black_box(&inputs[..]);
            for t in 0..OPS {
                let inp = black_box(&ins[t & IN_MASK]);
                let s = std::str::from_utf8(inp).unwrap(); let chars = unsafe { s.as_ascii_unchecked() };
                black_box(chars.as_ptr());
            }
        })
    });

    g.finish();
}

criterion_group!(benches, bench_candidate_core_str_mod_as_ascii_unchecked_2808);
criterion_main!(benches);
