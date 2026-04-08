#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};
use std::{alloc::Layout, ptr::Alignment};

const OPS: usize = 1 << 24;
const IN_LEN: usize = 1 << 16;
const IN_MASK: usize = IN_LEN - 1;

fn make_inputs(n: usize, seed: u64) -> (Vec<usize>, Vec<Alignment>) {
    let mut x = seed;
    let mut sizes = Vec::with_capacity(n);
    let mut aligns = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let lg = ((x >> 32) as usize % 6) + 3;
        let align = Alignment::new(1usize << lg).unwrap();
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        let size = ((x as usize) & 0x3fff) * align.as_usize();
        sizes.push(size);
        aligns.push(align);
    }
    (sizes, aligns)
}

fn bench_candidate_core_alloc_layout_from_size_alignment_unchecked_156(c: &mut Criterion) {
    let (sizes, aligns) = make_inputs(IN_LEN, 0x1234_5678_9abc_def0);
    let param = format!("in{}_ops{}", IN_LEN, OPS);
    let mut g = c.benchmark_group("candidate_core_alloc_layout_from_size_alignment_unchecked_156");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));

    g.bench_with_input(BenchmarkId::new("safe_from_size_alignment", &param), &(&sizes, &aligns), |b, (sizes, aligns)| {
        b.iter(|| {
            let ss = black_box(&sizes[..]);
            let as_ = black_box(&aligns[..]);
            for t in 0..OPS {
                let size = black_box(ss[t & IN_MASK]);
                let align = black_box(as_[t & IN_MASK]);
                let layout = Layout::from_size_alignment(size, align).unwrap();
                black_box((layout.size(), layout.align()));
            }
        })
    });

    g.bench_with_input(BenchmarkId::new("unsafe_from_size_alignment_unchecked", &param), &(&sizes, &aligns), |b, (sizes, aligns)| {
        b.iter(|| {
            let ss = black_box(&sizes[..]);
            let as_ = black_box(&aligns[..]);
            for t in 0..OPS {
                let size = black_box(ss[t & IN_MASK]);
                let align = black_box(as_[t & IN_MASK]);
                let layout = unsafe { Layout::from_size_alignment_unchecked(size, align) };
                black_box((layout.size(), layout.align()));
            }
        })
    });

    g.finish();
}

criterion_group!(benches, bench_candidate_core_alloc_layout_from_size_alignment_unchecked_156);
criterion_main!(benches);
