#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};
use std::{collections::{BTreeMap, BTreeSet}, ops::Bound};

const OPS: usize = 1 << 18;
const LEN: usize = 1 << 14;
const MASK: usize = LEN - 1;

fn make_inputs() -> Vec<u64> {
    (0..LEN as u64).map(|v| (v << 1) + 1).collect()
}

fn bench_candidate_alloc_collections_btree_map_insert_after_unchecked_3418(c: &mut Criterion) {
    let keys = make_inputs();
    let param = format!("len{}_ops{}", LEN, OPS);
    let mut g = c.benchmark_group("candidate_alloc_collections_btree_map_insert_after_unchecked_3418");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(60);
    g.measurement_time(Duration::from_secs(4));
    g.bench_with_input(BenchmarkId::new("safe_insert_after", &param), &keys, |b, keys| {
        b.iter(|| {
            let ks = black_box(&keys[..]);
            for t in 0..OPS {
                let key = black_box(ks[t & MASK]);
                let mut map: BTreeMap<u64, u64> = std::collections::BTreeMap::<u64, u64>::new();
                let cursor = map.lower_bound_mut(std::ops::Bound::Unbounded);
                let mut cursor = unsafe { cursor.with_mutable_key() };
                cursor.insert_after(key, key).unwrap();
                black_box(map.len());
            }
        })
    });
    g.bench_with_input(BenchmarkId::new("unsafe_insert_after_unchecked", &param), &keys, |b, keys| {
        b.iter(|| {
            let ks = black_box(&keys[..]);
            for t in 0..OPS {
                let key = black_box(ks[t & MASK]);
                let mut map: BTreeMap<u64, u64> = std::collections::BTreeMap::<u64, u64>::new();
                let cursor = map.lower_bound_mut(std::ops::Bound::Unbounded);
                let mut cursor = unsafe { cursor.with_mutable_key() };
                unsafe { cursor.insert_after_unchecked(key, key) };
                black_box(map.len());
            }
        })
    });
    g.finish();
}

criterion_group!(benches, bench_candidate_alloc_collections_btree_map_insert_after_unchecked_3418);
criterion_main!(benches);
