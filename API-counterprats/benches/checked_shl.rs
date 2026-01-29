#![feature(core_intrinsics)]
#![allow(unused_variables, unused_mut)]

use core::intrinsics;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};

const OPS: usize = 1 << 24;       // ops per measured iteration
const IN_LEN: usize = 1 << 20;    // precomputed inputs (power-of-two)
const IN_MASK: usize = IN_LEN - 1;

type T = u64;
const BITS: u32 = T::BITS;

fn make_vals(n: usize, seed: u64) -> Vec<T> {
    assert!(n.is_power_of_two());
    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        out.push(x as T);
    }
    out
}

fn make_shifts_inbounds(n: usize, seed: u64) -> Vec<u32> {
    assert!(n.is_power_of_two());
    assert!(BITS >= 2);

    // choose rhs in [1, BITS-1] so it's always valid and non-degenerate
    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let rhs = ((x as u32) % (BITS - 1)) + 1;
        out.push(rhs);
    }
    out
}

fn bench_checked_shl_pairs_a(c: &mut Criterion) {
    let vals = make_vals(IN_LEN, 0x1234_5678_9abc_def0);
    let shs = make_shifts_inbounds(IN_LEN, 0xdead_beef_cafe_f00d);
    let param = format!("type{}_in{}_ops{}", BITS, IN_LEN, OPS);

    let mut g = c.benchmark_group("cache_hot_checked_shl");
    g.throughput(Throughput::Elements(OPS as u64));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));

    g.bench_with_input(
        BenchmarkId::new("safe_checked_shl_blackbox", &param),
        &(&vals, &shs),
        |b, (vals, shs)| {
            b.iter(|| {
                let xs: &[T] = black_box(&vals[..]);
                let rs: &[u32] = black_box(&shs[..]);

                for t in 0..OPS {
                    let x: T = black_box(xs[t & IN_MASK]);
                    let r: u32 = black_box(rs[t & IN_MASK]);

                    // Always Some by construction (r < BITS).
                    let y = x.checked_shl(r).unwrap();
                    black_box(y);
                }
            })
        },
    );

    g.bench_with_input(
        BenchmarkId::new("unsafe_unchecked_shl_blackbox", &param),
        &(&vals, &shs),
        |b, (vals, shs)| {
            b.iter(|| {
                let xs: &[T] = black_box(&vals[..]);
                let rs: &[u32] = black_box(&shs[..]);

                for t in 0..OPS {
                    let x: T = black_box(xs[t & IN_MASK]);
                    let r: u32 = black_box(rs[t & IN_MASK]);

                    // Safety: r is generated in [1, BITS-1], so r < BITS holds.
                    let y: T = unsafe { intrinsics::unchecked_shl(x, r) };
                    black_box(y);
                }
            })
        },
    );

    g.finish();
}

criterion_group!(benches, bench_checked_shl_pairs_a);
criterion_main!(benches);
