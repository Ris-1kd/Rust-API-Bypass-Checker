#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};

const OPS: usize = 1 << 24;       // number of ops per measured iteration
const IN_LEN: usize = 1 << 20;    // precomputed inputs (power-of-two)
const IN_MASK: usize = IN_LEN - 1;

// Produce only valid Unicode scalar values (exclude surrogate range).
// Map x into [0, 0x10F800), then skip surrogate gap by +0x800 if needed.
fn make_valid_scalar_u32s(n: usize, seed: u64) -> Vec<u32> {
    assert!(n.is_power_of_two());

    const RANGE: u32 = 0x10F800; // 0x110000 - 0x800
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

fn bench_from_u32_pairs_a(c: &mut Criterion) {
    let inputs = make_valid_scalar_u32s(IN_LEN, 0x1234_5678_9abc_def0);
    let param = format!("in{}_ops{}", IN_LEN, OPS);

    let mut g = c.benchmark_group("cache_hot_from_u32");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));

    g.bench_with_input(
        BenchmarkId::new("safe_from_u32_blackbox", &param),
        &inputs,
        |b, inputs| {
            b.iter(|| {
                let ins: &[u32] = black_box(&inputs[..]);

                for t in 0..OPS {
                    let v: u32 = black_box(ins[t & IN_MASK]);

                    // Always Some by construction.
                    let c = char::from_u32(v).unwrap();

                    // Consume result without extra stores.
                    black_box(c);
                }
            })
        },
    );

    g.bench_with_input(
        BenchmarkId::new("unsafe_from_u32_unchecked_blackbox", &param),
        &inputs,
        |b, inputs| {
            b.iter(|| {
                let ins: &[u32] = black_box(&inputs[..]);

                for t in 0..OPS {
                    let v: u32 = black_box(ins[t & IN_MASK]);

                    // Safety: v is always a valid Unicode scalar value.
                    let c = unsafe { char::from_u32_unchecked(v) };

                    black_box(c);
                }
            })
        },
    );

    g.finish();
}

criterion_group!(benches, bench_from_u32_pairs_a);
criterion_main!(benches);
