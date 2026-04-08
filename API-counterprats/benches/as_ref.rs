#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, ptr, time::Duration};

const LEN: usize = 4096;          // cache-hot backing storage (power-of-two)
const OPS: usize = 1 << 24;       // operations per measured iteration

// 这组很“短”，我建议把索引序列也做成 cache-hot，否则 miss 噪声会盖过 as_ref。
// 1<<16 => 65536 * usize(8B) = 512KB，通常比 1<<20 更稳。
const IDX_LEN: usize = 1 << 16;
const IDX_MASK: usize = IDX_LEN - 1;

// 让 ptr table 里有少量 null，用来阻止编译器证明“所有 p 都非空”。
const NULL_STRIDE: usize = 256;  // 每 256 个放一个 null（约 0.39%）

fn make_ptr_table(data: &[u64]) -> Vec<*const u64> {
    assert!(data.len().is_power_of_two());
    let base = data.as_ptr();
    let mut out = Vec::with_capacity(data.len());

    for i in 0..data.len() {
        let p = if (i & (NULL_STRIDE - 1)) == 0 {
            ptr::null()
        } else {
            unsafe { base.add(i) }
        };
        out.push(p);
    }
    out
}

fn make_nonnull_indices(ptrs: &[*const u64], n: usize, seed: u64) -> Vec<usize> {
    assert!(n.is_power_of_two());
    assert!(ptrs.len().is_power_of_two());

    let mut x = seed;
    let mut out = Vec::with_capacity(n);

    for _ in 0..n {
        // 生成一个索引，若恰好指向 null，就再滚一次（null 很稀疏，开销可忽略）
        loop {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
            let i = (x as usize) & (ptrs.len() - 1);
            if !ptrs[i].is_null() {
                out.push(i);
                break;
            }
        }
    }
    out
}

fn bench_ptr_as_ref_pairs_a(c: &mut Criterion) {
    let data: Vec<u64> = (0..LEN as u64).collect();
    let ptrs = make_ptr_table(&data);
    let idxs = make_nonnull_indices(&ptrs, IDX_LEN, 0x1234_5678_9abc_def0);

    let param = format!("len{}_ops{}_nullstride{}", LEN, OPS, NULL_STRIDE);

    let mut g = c.benchmark_group("cache_hot_ptr_as_ref");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));

    // 统一的“消费”方式：把 Option<&u64> 转成地址，再用 4 累加器混合（不写内存）
    #[inline(always)]
    fn consume_opt(opt: Option<&u64>, t: usize, a0: &mut u64, a1: &mut u64, a2: &mut u64, a3: &mut u64) {
        // black_box 一下，避免 unchecked 分支中 “opt 恒为 Some” 被完全常量折叠
        let opt = black_box(opt);

        let addr: u64 = opt
            .map(|r| (r as *const u64 as usize) as u64)
            .unwrap_or(0);

        match t & 3 {
            0 => *a0 ^= addr,
            1 => *a1 = a1.wrapping_add(addr),
            2 => *a2 ^= addr.rotate_left(13),
            _ => *a3 = a3.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(addr),
        }
    }

    g.bench_with_input(
        BenchmarkId::new("ptr_as_ref", &param),
        &(&ptrs, &idxs),
        |b, (ptrs, idxs)| {
            b.iter(|| {
                let ps: &[*const u64] = black_box(&ptrs[..]);
                let is: &[usize] = black_box(&idxs[..]);

                let mut a0 = 0u64;
                let mut a1 = 0u64;
                let mut a2 = 0u64;
                let mut a3 = 0u64;

                for t in 0..OPS {
                    let i = black_box(is[t & IDX_MASK]);
                    let p: *const u64 = black_box(ps[i]);

                    // Safety: i 由 make_nonnull_indices 保证非 null，
                    // 且 p 指向 data 内部，生命周期覆盖整个 bench。
                    let opt:&u64 = unsafe { p.as_ref().unwrap() };
                    black_box(opt);

                    // consume_opt(opt, t, &mut a0, &mut a1, &mut a2, &mut a3);
                }

                black_box(a0 ^ a1 ^ a2 ^ a3);
            })
        },
    );

    g.bench_with_input(
        BenchmarkId::new("unchecked_no_nullcheck", &param),
        &(&ptrs, &idxs),
        |b, (ptrs, idxs)| {
            b.iter(|| {
                let ps: &[*const u64] = black_box(&ptrs[..]);
                let is: &[usize] = black_box(&idxs[..]);

                let mut a0 = 0u64;
                let mut a1 = 0u64;
                let mut a2 = 0u64;
                let mut a3 = 0u64;

                for t in 0..OPS {
                    let i = black_box(is[t & IDX_MASK]);
                    let p: *const u64 = black_box(ps[i]);

                    // Safety: 同上，p 在运行时保证非 null 且有效。
                    let opt = unsafe { Some(&*p).unwrap() };

                    // consume_opt(opt, t, &mut a0, &mut a1, &mut a2, &mut a3);
                    black_box(opt);
                }

                black_box(a0 ^ a1 ^ a2 ^ a3);
            })
        },
    );

    g.finish();
}

criterion_group!(benches, bench_ptr_as_ref_pairs_a);
criterion_main!(benches);
