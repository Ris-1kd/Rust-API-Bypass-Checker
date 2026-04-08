#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};

const BUF_LEN: usize = 4096;      // cache-hot backing bytes (power-of-two)
const SLICE_LEN: usize = 64;      // fixed slice length per op (keeps cost manageable & stable)

const OPS: usize = 1 << 21;       // operations per measured iteration
const SEQ_LEN: usize = 1 << 16;   // cache-hot slice sequence (power-of-two)
const SEQ_MASK: usize = SEQ_LEN - 1;

fn make_ascii_buf(len: usize) -> Vec<u8> {
    // Pure ASCII => always valid UTF-8 for any subslice.
    let alphabet = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        out.push(alphabet[i % alphabet.len()]);
    }
    out
}

fn make_offsets_inbounds(buf_len: usize, slice_len: usize, n: usize, seed: u64) -> Vec<usize> {
    assert!(buf_len >= slice_len);
    assert!(n.is_power_of_two());

    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    let max_off = buf_len - slice_len;

    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let off = (x as usize) % (max_off + 1);
        out.push(off);
    }
    out
}

fn bench_from_utf8_pairs_a(c: &mut Criterion) {
    let buf = make_ascii_buf(BUF_LEN);
    let offs = make_offsets_inbounds(BUF_LEN, SLICE_LEN, SEQ_LEN, 0x1234_5678_9abc_def0);

    // Pre-materialize slices so range slicing doesn't appear in the hot loop.
    let slices: Vec<&[u8]> = offs.iter().map(|&o| &buf[o..o + SLICE_LEN]).collect();
    let param = format!("buf{}_slice{}_ops{}", BUF_LEN, SLICE_LEN, OPS);

    let mut g = c.benchmark_group("cache_hot_from_utf8");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));

    #[inline(always)]
    fn consume_str(s: &str, t: usize, a0: &mut u64, a1: &mut u64, a2: &mut u64, a3: &mut u64) {
        let s = black_box(s);
        let p = s.as_ptr() as usize as u64;
        let l = s.len() as u64;
        let v = p ^ l.wrapping_mul(0x9E37_79B9_7F4A_7C15);

        match t & 3 {
            0 => *a0 ^= v,
            1 => *a1 = a1.wrapping_add(v),
            2 => *a2 ^= v.rotate_left(13),
            _ => *a3 = a3.wrapping_mul(0xD6E8_FEB8_6659_FD93).wrapping_add(v),
        }
    }

    g.bench_with_input(
        BenchmarkId::new("safe_from_utf8_blackbox", &param),
        &slices,
        |b, slices| {
            b.iter(|| {
                let ss: &[&[u8]] = black_box(&slices[..]);

                let mut a0 = 0u64;
                let mut a1 = 0u64;
                let mut a2 = 0u64;
                let mut a3 = 0u64;

                for t in 0..OPS {
                    let v: &[u8] = black_box(ss[t & SEQ_MASK]);

                    // Always Ok because v is ASCII.
                    let s = std::str::from_utf8(v).unwrap();
                    consume_str(s, t, &mut a0, &mut a1, &mut a2, &mut a3);
                }

                black_box(a0 ^ a1 ^ a2 ^ a3);
            })
        },
    );

    g.bench_with_input(
        BenchmarkId::new("unsafe_from_utf8_unchecked_blackbox", &param),
        &slices,
        |b, slices| {
            b.iter(|| {
                let ss: &[&[u8]] = black_box(&slices[..]);

                let mut a0 = 0u64;
                let mut a1 = 0u64;
                let mut a2 = 0u64;
                let mut a3 = 0u64;

                for t in 0..OPS {
                    let v: &[u8] = black_box(ss[t & SEQ_MASK]);

                    // Safety: v is always valid UTF-8 (ASCII).
                    let s = unsafe { std::str::from_utf8_unchecked(v) };
                    consume_str(s, t, &mut a0, &mut a1, &mut a2, &mut a3);
                }

                black_box(a0 ^ a1 ^ a2 ^ a3);
            })
        },
    );

    g.finish();
}

criterion_group!(benches, bench_from_utf8_pairs_a);
criterion_main!(benches);
