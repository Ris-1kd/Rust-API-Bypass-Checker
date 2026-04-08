use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;
use std::hint::black_box;

fn make_indices_inbounds(len: usize, n: usize, seed: u64) -> Vec<usize> {
    assert!(len > 0);
    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        // deterministic pseudo-random, always in-bounds
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        out.push((x as usize) % len);
    }
    out
}

fn bench_vec_get_vs_get_unchecked_cache_hot(c: &mut Criterion) {
    // Cache-hot data: 4096 * 4 bytes = 16KB (usually fits in L1 on modern CPUs)
    let data_len: usize = 4096;
    let v: Vec<u32> = (0..data_len as u32).collect();
    let s: &[u32] = v.as_slice();
    let mut v1: Vec<u32> = (0..data_len as u32).collect();
    let m: &mut[u32] = v1.as_mut_slice();
    // Keep index list small-ish so it also stays hot; sequential iteration is prefetch-friendly.
    let idxs_len: usize = 4096; // 4096 * 8 bytes = 32KB on 64-bit
    let idxs = make_indices_inbounds(data_len, idxs_len, 0x1234_5678_9abc_def0);

    // Amplify work per Criterion iteration without enlarging buffers.
    // Total loads per iter = idxs_len * rounds.
    let rounds: usize = 4096; // 4096 * 4096 = 16,777,216 loads per iter

    let total_ops_per_iter: u64 = (idxs_len as u64) * (rounds as u64);

    let mut group = c.benchmark_group("cache_hot_vec_get");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(8));
    group.sample_size(80);
    group.throughput(Throughput::Elements(total_ops_per_iter));

   
    

    // --- Unsafe: slice.get_unchecked_mut(idx) (no bounds check).
    group.bench_with_input(
        BenchmarkId::new(
            "get_unchecked_mut_inbounds",
            format!("len{}_ops{}", data_len, total_ops_per_iter),
        ),
        &idxs,
        |b, idxs| {
            b.iter(|| {
                let slice: &mut [u32] = black_box(m);
                let mut acc: u32 = 0;

                for _ in 0..rounds {
                    for &idx in idxs.iter() {
                        // Safety: idxs are generated in-bounds.
                        let v: &mut u32 = unsafe { slice.get_unchecked_mut(idx) };
                        acc = acc.wrapping_add(*v);
                    }
                }

                black_box(acc)
            })
        },
    );

    // --- Safe: slice.get_mut(idx).unwrap()
    group.bench_with_input(
        BenchmarkId::new(
            "get_mut_inbounds",
            format!("len{}_ops{}", data_len, total_ops_per_iter),
        ),
        &idxs,
        |b, idxs| {
            b.iter(|| {
                let slice: &mut [u32] = black_box(m);
                let mut acc: u32 = 0;

                for _ in 0..rounds {
                    for &idx in idxs.iter() {
                        // In-bounds by construction -> always Some on the hot path.
                        let v: &mut u32 = slice.get_mut(idx).unwrap();
                        acc = acc.wrapping_add(*v);
                    }
                }

                black_box(acc)
            })
        },
    );
    
     // --- Safe: slice.get(idx) + match (no unwrap/panic structure).
    // Because idx is always in-bounds, this measures the steady-state cost of:
    // bounds check + Option discriminant + load.
    group.bench_with_input(
        BenchmarkId::new("get_match_inbounds", format!("len{}_ops{}", data_len, total_ops_per_iter)),
        &idxs,
        |b, idxs| {
            b.iter(|| {
                let slice = black_box(s);
                let mut acc: u32 = 0;

                for _ in 0..rounds {
                    for &idx in idxs.iter() {
                        // In-bounds by construction -> always Some on the hot path.
                        let v = slice.get(idx).unwrap();
                        acc = acc.wrapping_add(*v);
                    }
                }
                black_box(acc)
            })
        },
    );

    // --- Unsafe: slice.get_unchecked(idx) (no bounds check).
    group.bench_with_input(
        BenchmarkId::new(
            "get_unchecked_inbounds",
            format!("len{}_ops{}", data_len, total_ops_per_iter),
        ),
        &idxs,
        |b, idxs| {
            b.iter(|| {
                let slice = black_box(s);
                let mut acc: u32 = 0;

                for _ in 0..rounds {
                    for &idx in idxs.iter() {
                        let v = unsafe { slice.get_unchecked(idx) };
                        acc = acc.wrapping_add(*v);
                    }
                }

                black_box(acc)
            })
        },
    );

    group.finish();
}

criterion_group!(benches, bench_vec_get_vs_get_unchecked_cache_hot);
criterion_main!(benches);
