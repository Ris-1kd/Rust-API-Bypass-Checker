#![feature(slice_as_chunks)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};

const LEN: usize = 4096;          // cache-hot working set (power-of-two)
const OPS: usize = 1 << 24;       // operations per measured iteration
const SEQ_LEN: usize = 1 << 20;   // precomputed selector bits (power-of-two)
const OUT_LEN: usize = 1 << 10;   // small ring buffer
const OUT_MASK: usize = OUT_LEN - 1;
const SEQ_MASK: usize = SEQ_LEN - 1;

const CHUNK: usize = 8;

fn make_bits(n: usize, seed: u64) -> Vec<u8> {
    assert!(n.is_power_of_two());
    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        out.push((x as u8) & 1);
    }
    out
}

fn bench_as_chunks_mut_pairs(c: &mut Criterion) {
    assert!(LEN % CHUNK == 0);
    let bits = make_bits(SEQ_LEN, 0xdead_beef_cafe_f00d);
    let param = format!("len{}_n{}_ops{}", LEN, CHUNK, OPS);

    // Two buffers to avoid hoisting/CSE of as_chunks_mut across iterations.
    let mut a: Vec<u32> = (0..LEN as u32).collect();
    let mut bbuf: Vec<u32> = (0..LEN as u32).rev().collect();

    let mut g = c.benchmark_group("cache_hot_as_chunks_mut");
    g.throughput(Throughput::Elements(OPS as u64));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));

    g.bench_with_input(
        BenchmarkId::new("safe_as_chunks_mut", &param),
        &bits,
        |b, bits| {
            b.iter(|| {
                let bs: &[u8] = black_box(&bits[..]);

                let mut out = [0u64; OUT_LEN];
                let mut w = 0usize;

                for t in 0..OPS {
                    let sel = bs[t & SEQ_MASK];

                    // Select between two distinct slices, keeping everything safe.
                    let s: &mut [u32] = if sel == 0 {
                        black_box(&mut a[..])
                    } else {
                        black_box(&mut bbuf[..])
                    };

                    let (chunks, rem) = s.as_chunks_mut::<CHUNK>();
                    black_box(chunks);
                    black_box(rem);
                    // out[w] =
                    //     (chunks.as_mut_ptr() as usize as u64) ^ (rem.as_mut_ptr() as usize as u64);
                    // w = (w + 1) & OUT_MASK;
                }

                // let mut chk: u64 = 0;
                // for k in (0..OUT_LEN).step_by(64) {
                //     chk = chk.wrapping_add(out[k]);
                // }
                // black_box(chk);
            })
        },
    );

    g.bench_with_input(
        BenchmarkId::new("unsafe_as_chunks_unchecked_mut", &param),
        &bits,
        |b, bits| {
            b.iter(|| {
                let bs: &[u8] = black_box(&bits[..]);

                let mut out = [0u64; OUT_LEN];
                let mut w = 0usize;

                for t in 0..OPS {
                    let sel = bs[t & SEQ_MASK];

                    let s: &mut [u32] = if sel == 0 {
                        black_box(&mut a[..])
                    } else {
                        black_box(&mut bbuf[..])
                    };

                    // Safety: LEN % CHUNK == 0, and CHUNK != 0.
                    let chunks: &mut [[u32; CHUNK]] =
                        unsafe { s.as_chunks_unchecked_mut::<CHUNK>() };
                        black_box(chunks);
                    // let end = unsafe { s.as_mut_ptr().add(s.len()) };

                    // out[w] = (chunks.as_mut_ptr() as usize as u64) ^ (end as usize as u64);
                    // w = (w + 1) & OUT_MASK;
                }

                // let mut chk: u64 = 0;
                // for k in (0..OUT_LEN).step_by(64) {
                //     chk = chk.wrapping_add(out[k]);
                // }
                // black_box(chk);
            })
        },
    );

    g.finish();
}

criterion_group!(benches, bench_as_chunks_mut_pairs);
criterion_main!(benches);
