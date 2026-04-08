#![feature(slice_split_at_unchecked)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};

const LEN: usize = 4096;          // cache-hot working set (power-of-two)
const OPS: usize = 1 << 24;       // number of split operations per measured iteration
const MID_LEN: usize = 1 << 20;   // precomputed mids (power-of-two)
const OUT_LEN: usize = 1 << 10;   // small ring buffer
const OUT_MASK: usize = OUT_LEN - 1;
const MID_MASK: usize = MID_LEN - 1;

fn make_mids_inbounds(len: usize, n: usize, seed: u64) -> Vec<usize> {
    assert!(len >= 2);
    assert!(len.is_power_of_two());
    assert!(n.is_power_of_two());

    let mut x = seed;
    let mut out = Vec::with_capacity(n);

    for _ in 0..n {
        // deterministic and cheap; done outside the measured loop
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);

        // Choose mid in [1, len-1] to avoid degenerate empty splits.
        let mid = ((x as usize) & (len - 2)) + 1;
        out.push(mid);
    }
    out
}

fn bench_split_at_pairs(c: &mut Criterion) {
    let mids: Vec<usize> = make_mids_inbounds(LEN, MID_LEN, 0xdead_beef_cafe_f00d);
    let param = format!("len{}_ops{}", LEN, OPS);

    // =========================
    // Group 1: split_at (safe vs unsafe)
    // =========================
    let data_ro: Vec<u32> = (0..LEN as u32).collect();

    let mut g1 = c.benchmark_group("cache_hot_split_at");
    g1.throughput(Throughput::Elements(OPS as u64));
    g1.warm_up_time(Duration::from_secs(3));
    g1.sample_size(80);
    g1.measurement_time(Duration::from_secs(5));

    g1.bench_with_input(
        BenchmarkId::new("safe_split_at", &param),
        &(&data_ro, &mids),
        |b, (data, mids)| {
            b.iter(|| {
                let s: &[u32] = black_box(&data[..]);
                let ms: &[usize] = black_box(&mids[..]);

                // ring buffer forces observable stores without an accumulator dependency chain
                let mut out = [0u64; OUT_LEN];
                let mut w = 0usize;

                for t in 0..OPS {
                    let mid = black_box(ms[t & MID_MASK]);

                    let (l, r) = s.split_at(mid);

                    // Use pointer-derived values to "consume" (l, r) with minimal extra work.
                    out[w] = (l.as_ptr() as usize as u64) ^ (r.as_ptr() as usize as u64);
                    w = (w + 1) & OUT_MASK;
                }

                // small post-pass checksum outside hot loop
                let mut chk: u64 = 0;
                for k in (0..OUT_LEN).step_by(64) {
                    chk = chk.wrapping_add(out[k]);
                }
                black_box(chk);
            })
        },
    );

    g1.bench_with_input(
        BenchmarkId::new("unsafe_split_at_unchecked", &param),
        &(&data_ro, &mids),
        |b, (data, mids)| {
            b.iter(|| {
                let s: &[u32] = black_box(&data[..]);
                let ms: &[usize] = black_box(&mids[..]);

                let mut out = [0u64; OUT_LEN];
                let mut w = 0usize;

                for t in 0..OPS {
                    let mid = black_box(ms[t & MID_MASK]);

                    // Safety: mid is generated in [1, LEN-1].
                    let (l, r) = unsafe { s.split_at_unchecked(mid) };
                    out[w] = (l.as_ptr() as usize as u64) ^ (r.as_ptr() as usize as u64);
                    w = (w + 1) & OUT_MASK;
                }

                let mut chk: u64 = 0;
                for k in (0..OUT_LEN).step_by(64) {
                    chk = chk.wrapping_add(out[k]);
                }
                black_box(chk);
            })
        },
    );

    g1.finish();

    // =========================
    // Group 2: split_at_mut (safe vs unsafe)
    // =========================
    let mut data_rw: Vec<u32> = (0..LEN as u32).collect();

    let mut g2 = c.benchmark_group("cache_hot_split_at_mut");
    g2.throughput(Throughput::Elements(OPS as u64));
    g2.warm_up_time(Duration::from_secs(3));
    g2.sample_size(80);
    g2.measurement_time(Duration::from_secs(5));

    g2.bench_with_input(
        BenchmarkId::new("safe_split_at_mut", &param),
        &mids,
        |b, mids| {
            b.iter(|| {
                let s: &mut [u32] = black_box(&mut data_rw[..]);
                let ms: &[usize] = black_box(&mids[..]);

                let mut out = [0u64; OUT_LEN];
                let mut w = 0usize;

                for t in 0..OPS {
                    let mid = black_box(ms[t & MID_MASK]);

                    let (l, r) = s.split_at_mut(mid);
                    out[w] = (l.as_mut_ptr() as usize as u64) ^ (r.as_mut_ptr() as usize as u64);
                    w = (w + 1) & OUT_MASK;
                }

                let mut chk: u64 = 0;
                for k in (0..OUT_LEN).step_by(64) {
                    chk = chk.wrapping_add(out[k]);
                }
                black_box(chk);
            })
        },
    );

    g2.bench_with_input(
        BenchmarkId::new("unsafe_split_at_mut_unchecked", &param),
        &mids,
        |b, mids| {
            b.iter(|| {
                let s: &mut [u32] = black_box(&mut data_rw[..]);
                let ms: &[usize] = black_box(&mids[..]);

                let mut out = [0u64; OUT_LEN];
                let mut w = 0usize;

                for t in 0..OPS {
                    let mid = black_box(ms[t & MID_MASK]);

                    // Safety: mid is generated in [1, LEN-1].
                    let (l, r) = unsafe { s.split_at_mut_unchecked(mid) };

                    out[w] = (l.as_mut_ptr() as usize as u64) ^ (r.as_mut_ptr() as usize as u64);
                    w = (w + 1) & OUT_MASK;
                }

                let mut chk: u64 = 0;
                for k in (0..OUT_LEN).step_by(64) {
                    chk = chk.wrapping_add(out[k]);
                }
                black_box(chk);
            })
        },
    );

    g2.finish();
}

criterion_group!(benches, bench_split_at_pairs);
criterion_main!(benches);
