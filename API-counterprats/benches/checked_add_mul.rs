#![feature(unchecked_math)]
#![allow(unused_variables, unused_mut)]

use criterion::{ criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;
use std::hint::black_box;

const OPS: usize = 1 << 24;        // number of ops per measured iteration
const IN_LEN: usize = 1 << 20;     // precomputed inputs (power-of-two)
const IN_MASK: usize = IN_LEN - 1;

const OUT_LEN: usize = 1 << 10;    // small ring buffer to keep results observable
const OUT_MASK: usize = OUT_LEN - 1;

// Deterministic input generator (outside hot loop).
fn make_pairs_add(n: usize, seed: u64) -> (Vec<u32>, Vec<u32>) {
    assert!(n.is_power_of_two());
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    let mut x = seed;
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        // keep values in [0, 2^31), so x+y never overflows u32
        let a = (x as u32) & 0x7FFF_FFFF;
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let b = (x as u32) & 0x7FFF_FFFF;
        xs.push(a);
        ys.push(b);
    }
    (xs, ys)
}

fn make_pairs_mul(n: usize, seed: u64) -> (Vec<u32>, Vec<u32>) {
    assert!(n.is_power_of_two());
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    let mut x = seed;
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        // keep values in [0, 65535], so a*b < 2^32 and never overflows u32
        let a = (x as u32) & 0x0000_FFFF;
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let b = (x as u32) & 0x0000_FFFF;
        xs.push(a);
        ys.push(b);
    }
    (xs, ys)
}

fn bench_checked_add_mul(c: &mut Criterion) {
    let (xs_add, ys_add) = make_pairs_add(IN_LEN, 0x1234_5678_9abc_def0);
    let (xs_mul, ys_mul) = make_pairs_mul(IN_LEN, 0xdead_beef_cafe_f00d);

    let mut group = c.benchmark_group("cache_hot_checked_ops");
    group.throughput(Throughput::Elements(OPS as u64));
    group.sample_size(80);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(5));

    // -----------------------
    // checked_add vs add_unchecked
    // -----------------------
    let param_add = format!("ops{}", OPS);

    group.bench_with_input(
        BenchmarkId::new("checked_add_unwrap", &param_add),
        &(&xs_add, &ys_add),
        |b, (xs, ys)| {
            b.iter(|| {
                let xs: &[u32] = black_box(&xs[..]);
                let ys: &[u32] = black_box(&ys[..]);

                let mut out = [0u32; OUT_LEN];
                let mut w = 0usize;

                for t in 0..OPS {
                    let i = t & IN_MASK;
                    let v = xs[i].checked_add(ys[i]).unwrap();
                    out[w] = v;
                    w = (w + 1) & OUT_MASK;
                }
                black_box(out);
                // let mut chk: u32 = 0;
                // for k in (0..OUT_LEN).step_by(64) {
                //     chk = chk.wrapping_add(out[k]);
                // }
                // black_box(chk);
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("unchecked_add", &param_add),
        &(&xs_add, &ys_add),
        |b, (xs, ys)| {
            b.iter(|| {
                let xs: &[u32] = black_box(&xs[..]);
                let ys: &[u32] = black_box(&ys[..]);

                let mut out = [0u32; OUT_LEN];
                let mut w = 0usize;

                for t in 0..OPS {
                    let i = t & IN_MASK;
                    let v = unsafe { xs[i].unchecked_add(ys[i]) };
                    // black_box(v);
                    out[w] = v;
                    w = (w + 1) & OUT_MASK;
                }

                black_box(out);
                // let mut chk: u32 = 0;
                // for k in (0..OUT_LEN).step_by(64) {
                //     chk = chk.wrapping_add(out[k]);
                // }
                // black_box(chk);
            })
        },
    );

    // -----------------------
    // checked_mul vs mul_unchecked
    // -----------------------
    let param_mul = format!("ops{}", OPS);

    group.bench_with_input(
        BenchmarkId::new("checked_mul_unwrap", &param_mul),
        &(&xs_mul, &ys_mul),
        |b, (xs, ys)| {
            b.iter(|| {
                let xs: &[u32] = black_box(&xs[..]);
                let ys: &[u32] = black_box(&ys[..]);

                let mut out = [0u32; OUT_LEN];
                let mut w = 0usize;

                for t in 0..OPS {
                    let i = t & IN_MASK;
                    let v = xs[i].checked_mul(ys[i]).unwrap();
                    black_box(v);
                    // out[w] = v;
                    // w = (w + 1) & OUT_MASK;
                }

                // let mut chk: u32 = 0;
                // for k in (0..OUT_LEN).step_by(64) {
                //     chk = chk.wrapping_add(out[k]);
                // }
                // black_box(chk);
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("unchecked_mul", &param_mul),
        &(&xs_mul, &ys_mul),
        |b, (xs, ys)| {
            b.iter(|| {
                let xs: &[u32] = black_box(&xs[..]);
                let ys: &[u32] = black_box(&ys[..]);

                let mut out = [0u32; OUT_LEN];
                let mut w = 0usize;

                for t in 0..OPS {
                    let i = t & IN_MASK;
                    let v = unsafe { xs[i].unchecked_mul(ys[i]) };
                    black_box(v);
                    // out[w] = v;
                    // w = (w + 1) & OUT_MASK;
                }

                // let mut chk: u32 = 0;
                // for k in (0..OUT_LEN).step_by(64) {
                //     chk = chk.wrapping_add(out[k]);
                // }
                // black_box(chk);
            })
        },
    );

    group.finish();
}

criterion_group!(benches, bench_checked_add_mul);
criterion_main!(benches);
