#![feature(slice_as_chunks)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};

const LEN: usize = 4096;          // cache-hot working set (power-of-two)
const OPS: usize = 1 << 24;       // operations per measured iteration
const SEQ_LEN: usize = 1 << 20;   // precomputed sequence length (power-of-two)
const OUT_LEN: usize = 1 << 10;   // small ring buffer
const OUT_MASK: usize = OUT_LEN - 1;
const SEQ_MASK: usize = SEQ_LEN - 1;

const CHUNK: usize = 8;

fn make_lens_mul_of_chunk(len: usize, n: usize, seed: u64) -> Vec<usize> {
    assert!(len.is_power_of_two());
    assert!(n.is_power_of_two());
    assert!(CHUNK != 0);
    assert!(len >= CHUNK);
    assert!(len % CHUNK == 0);

    let blocks = len / CHUNK; // choose k in [1, blocks], l = k*CHUNK
    assert!(blocks.is_power_of_two());

    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let k = ((x as usize) & (blocks - 1)) + 1;
        out.push(k * CHUNK);
    }
    out
}

fn bench_as_chunks_pairs(c: &mut Criterion) {
    let lens = make_lens_mul_of_chunk(LEN, SEQ_LEN, 0x1234_5678_9abc_def0);
    let param = format!("len{}_n{}_ops{}", LEN, CHUNK, OPS);

    let data_ro: Vec<u32> = (0..LEN as u32).collect();

    // Pre-materialize subslices so range slicing is not in the hot loop.
    // All lengths are multiples of CHUNK => remainder is always empty.
    let subs: Vec<&[u32]> = lens.iter().map(|&l| &data_ro[..l]).collect();

    let mut g = c.benchmark_group("cache_hot_as_chunks");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));

    g.bench_with_input(
        BenchmarkId::new("safe_as_chunks", &param),
        &subs,
        |b, subs| {
            b.iter(|| {
                let ss: &[&[u32]] = black_box(&subs[..]);

                let mut out = [0u64; OUT_LEN];
                let mut w = 0usize;

                for t in 0..OPS {
                    let s: &[u32] = black_box(ss[t & SEQ_MASK]);

                    let (chunks, rem) = s.as_chunks::<CHUNK>();
                    // out[w] =
                    //     (chunks.as_ptr() as usize as u64) ^ (rem.as_ptr() as usize as u64);
                    // w = (w + 1) & OUT_MASK;
                    black_box(chunks);
                    black_box(rem);
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
        BenchmarkId::new("unsafe_as_chunks_unchecked", &param),
        &subs,
        |b, subs| {
            b.iter(|| {
                let ss: &[&[u32]] = black_box(&subs[..]);

                let mut out = [0u64; OUT_LEN];
                let mut w = 0usize;

                for t in 0..OPS {
                    let s: &[u32] = black_box(ss[t & SEQ_MASK]);

                    // Safety: s.len() is always a multiple of CHUNK, and CHUNK != 0.
                    let chunks: &[[u32; CHUNK]] = unsafe { s.as_chunks_unchecked::<CHUNK>() };

                    black_box(chunks);
                    // Remainder is always empty => its ptr equals end-of-slice.
                    // let end = unsafe { s.as_ptr().add(s.len()) };

                    // out[w] = (chunks.as_ptr() as usize as u64) ^ (end as usize as u64);
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

criterion_group!(benches, bench_as_chunks_pairs);
criterion_main!(benches);
