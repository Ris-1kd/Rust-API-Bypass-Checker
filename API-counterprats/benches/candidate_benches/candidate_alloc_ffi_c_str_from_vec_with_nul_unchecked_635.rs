#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};
use std::ffi::CString;

const OPS: usize = 1 << 21;
const IN_LEN: usize = 1 << 14;
const IN_MASK: usize = IN_LEN - 1;

fn make_inputs(n: usize) -> Vec<Vec<u8>> {
    let alphabet = b"abcdefghijklmnopqrstuvwxyz";
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let mut item = Vec::with_capacity(33);
        for j in 0..32 {
            item.push(alphabet[(i + j) % alphabet.len()]);
        }
        item.push(0);
        out.push(item);
    }
    out
}

fn bench_candidate_alloc_ffi_c_str_from_vec_with_nul_unchecked_635(c: &mut Criterion) {
    let inputs = make_inputs(IN_LEN);
    let param = format!("in{}_ops{}", IN_LEN, OPS);
    let mut g = c.benchmark_group("candidate_alloc_ffi_c_str_from_vec_with_nul_unchecked_635");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));
    g.bench_with_input(BenchmarkId::new("safe_from_vec_with_nul", &param), &inputs, |b, inputs| {
        b.iter(|| {
            let ins = black_box(&inputs[..]);
            for t in 0..OPS {
                let v = black_box(ins[t & IN_MASK].clone());
                let s = CString::from_vec_with_nul(v).unwrap();
                black_box((s.as_ptr() as usize as u64) ^ (s.as_bytes().len() as u64));
            }
        })
    });
    g.bench_with_input(BenchmarkId::new("unsafe_from_vec_with_nul_unchecked", &param), &inputs, |b, inputs| {
        b.iter(|| {
            let ins = black_box(&inputs[..]);
            for t in 0..OPS {
                let v = black_box(ins[t & IN_MASK].clone());
                let s = unsafe { CString::from_vec_with_nul_unchecked(v) };
                black_box((s.as_ptr() as usize as u64) ^ (s.as_bytes().len() as u64));
            }
        })
    });
    g.finish();
}

criterion_group!(benches, bench_candidate_alloc_ffi_c_str_from_vec_with_nul_unchecked_635);
criterion_main!(benches);
