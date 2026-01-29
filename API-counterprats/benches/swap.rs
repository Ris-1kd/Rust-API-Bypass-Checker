#![feature(slice_swap_unchecked)]

use criterion::{ criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;
use std::hint::black_box;

fn make_pairs_inbounds(len: usize, n: usize, seed: u64) -> Vec<(usize, usize)> {
    assert!(len >= 2);
    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        // deterministic pseudo-random
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut i = (x as usize) % len;

        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut j = (x as usize) % len;

        // ensure i != j to avoid degenerate swap cases
        if i == j {
            j = (j + 1) % len;
        }

        out.push((i, j));
    }
    out
}

fn bench_swap_vs_swap_unchecked_cache_hot(c: &mut Criterion) {
    // Cache-hot data: 4096 * 4 bytes = 16KB
    let data_len: usize = 4096;
    let base: Vec<u32> = (0..data_len as u32).collect();

    // Pair list: small-ish, repeated many rounds
    let pairs_len: usize = 4096;
    let pairs = make_pairs_inbounds(data_len, pairs_len, 0x1234_5678_9abc_def0);

    // Amplify work per iteration
    let rounds: usize = 4096;
    let total_swaps_per_iter: u64 = (pairs_len as u64) * (rounds as u64);

    let mut group = c.benchmark_group("cache_hot_slice_swap");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(8));
    group.sample_size(80);
    group.throughput(Throughput::Elements(total_swaps_per_iter));

    // --- Safe swap()
    group.bench_with_input(
        BenchmarkId::new("swap_inbounds", format!("len{}_ops{}", data_len, total_swaps_per_iter)),
        &pairs,
        |b, pairs| {
            b.iter(|| {
                // fresh copy each iter so state doesn't drift between samples
                // (keeps the workload stable and avoids any weird long-run artifacts)
                let mut v = base.clone();
                let s = black_box(v.as_mut_slice());

                for _ in 0..rounds {
                    for &(i, j) in pairs.iter() {
                        s.swap(i, j);
                    }
                }

                // lightweight checksum: sample a few positions deterministically
                // (prevents "unused work" concerns while keeping overhead tiny)
                let chk = s[0]
                    ^ s[data_len / 4]
                    ^ s[data_len / 2]
                    ^ s[(data_len * 3) / 4]
                    ^ s[data_len - 1];
                black_box(chk)
            })
        },
    );

    // --- Unsafe swap_unchecked()
    group.bench_with_input(
        BenchmarkId::new(
            "swap_unchecked_inbounds",
            format!("len{}_ops{}", data_len, total_swaps_per_iter),
        ),
        &pairs,
        |b, pairs| {
            b.iter(|| {
                let mut v = base.clone();
                let s = black_box(v.as_mut_slice());

                for _ in 0..rounds {
                    for &(i, j) in pairs.iter() {
                        unsafe {
                            s.swap_unchecked(i, j);
                        }
                    }
                }

                let chk = s[0]
                    ^ s[data_len / 4]
                    ^ s[data_len / 2]
                    ^ s[(data_len * 3) / 4]
                    ^ s[data_len - 1];
                black_box(chk)
            })
        },
    );

    group.finish();
}

criterion_group!(benches, bench_swap_vs_swap_unchecked_cache_hot);
criterion_main!(benches);
