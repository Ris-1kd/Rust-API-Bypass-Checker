use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;
use std::hint::black_box;

const OUT_LEN: usize = 1 << 12; // 1024
const OUT_MASK: usize = OUT_LEN - 1;

/// deterministic pseudo-random u32 list, masked to avoid overflow when added
fn gen_u32s_no_overflow(n: usize, seed: u64) -> Vec<u32> {
    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        // keep values small enough so a+b won't overflow u32
        out.push(((x >> 32) as u32) & 0x3FFF_FFFF);
    }
    out
}

fn bench_checked_add_vs_unchecked_add_cache_hot(c: &mut Criterion) {
    // cache-hot operands
    let n: usize = 4096;
    let xs = gen_u32s_no_overflow(n, 0x1111_2222_3333_4444);
    let ys = gen_u32s_no_overflow(n, 0xaaaa_bbbb_cccc_dddd);

    // amplify work per iter
    let rounds: usize = 4096; // total adds per iter ~ 16M
    let total_ops: u64 = (n as u64) * (rounds as u64);

    let mut group = c.benchmark_group("cache_hot_checked_add");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(8));
    group.sample_size(80);
    group.throughput(Throughput::Elements(total_ops));

    group.bench_with_input(
        BenchmarkId::new("checked_add", format!("ops{}", total_ops)),
        &(&xs, &ys),
        |b, (xs, ys)| {
            b.iter(|| {
                let xs = black_box(xs.as_slice());
                let ys = black_box(ys.as_slice());

                // ring buffer to consume results with minimal loop-carried dependency
                let mut acc_buf = [0u32; OUT_LEN];
                let mut w: usize = 0;

                for _ in 0..rounds {
                    for i in 0..n {
                        // always Some by construction
                        let v = xs[i].checked_add(ys[i]).unwrap();
                        acc_buf[i] = v;
                        // w = (w + 1) & OUT_MASK;
                    }
                }

                // tiny checksum outside hot loop
                // let mut chk: u32 = 0;
                // for k in (0..OUT_LEN).step_by(64) {
                //     chk = chk.wrapping_add(acc_buf[k]);
                // }
                // black_box(chk)
                black_box(acc_buf);
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("unchecked_add", format!("ops{}", total_ops)),
        &(&xs, &ys),
        |b, (xs, ys)| {
            b.iter(|| {
                let xs = black_box(xs.as_slice());
                let ys = black_box(ys.as_slice());

                let mut acc_buf = [0u32; OUT_LEN];
                let mut w: usize = 0;

                for _ in 0..rounds {
                    for i in 0..n {
                        // unsafe, assumes no overflow
                        let v = unsafe { xs[i].unchecked_add(ys[i]) };
                        acc_buf[i] = v;
                        // w = (w + 1) & OUT_MASK;
                    }
                }
                black_box(acc_buf);
                // let mut chk: u32 = 0;
                // for k in (0..OUT_LEN).step_by(64) {
                //     chk = chk.wrapping_add(acc_buf[k]);
                // }
                // black_box(chk)
            })
        },
    );

    group.finish();
}

criterion_group!(benches, bench_checked_add_vs_unchecked_add_cache_hot);
criterion_main!(benches);
