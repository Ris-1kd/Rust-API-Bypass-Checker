#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};
use std::rc::Rc;

const OPS: usize = 1 << 23;
const LEN: usize = 1 << 14;
const MASK: usize = LEN - 1;

fn make_inputs() -> Vec<Rc<u64>> {
    (0..LEN as u64).map(Rc::new).collect()
}

fn bench_candidate_alloc_rc_get_mut_unchecked_1983(c: &mut Criterion) {
    let param = format!("len{}_ops{}", LEN, OPS);
    let mut safe_items = make_inputs();
    let mut unsafe_items = make_inputs();
    let mut g = c.benchmark_group("candidate_alloc_rc_get_mut_unchecked_1983");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));
    g.bench_function(BenchmarkId::new("safe_get_mut", &param), |b| {
        b.iter(|| {
            let items = black_box(&mut safe_items[..]);
            for t in 0..OPS {
                let item = unsafe { items.get_unchecked_mut(t & MASK) };
                let v = Rc::get_mut(item).unwrap();
                *v = v.wrapping_add(1);
                black_box(*v);
            }
        })
    });
    g.bench_function(BenchmarkId::new("unsafe_get_mut_unchecked", &param), |b| {
        b.iter(|| {
            let items = black_box(&mut unsafe_items[..]);
            for t in 0..OPS {
                let item = unsafe { items.get_unchecked_mut(t & MASK) };
                let v = unsafe { Rc::get_mut_unchecked(item) };
                *v = v.wrapping_add(1);
                black_box(*v);
            }
        })
    });
    g.finish();
}

criterion_group!(benches, bench_candidate_alloc_rc_get_mut_unchecked_1983);
criterion_main!(benches);
