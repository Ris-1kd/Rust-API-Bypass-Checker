#![feature(ascii_char, btree_cursors, core_intrinsics, core_intrinsics_fallbacks, exact_bitshifts, exact_div, funnel_shifts, get_mut_unchecked, nonzero_from_mut, nonzero_ops, ptr_alignment_type, raw_slice_split, slice_swap_unchecked, unchecked_neg, unchecked_shifts)]
#![allow(unused_variables, unused_mut)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};
use std::ffi::CStr;

const OPS: usize = 1 << 23;
const IN_LEN: usize = 1 << 14;
const IN_MASK: usize = IN_LEN - 1;
type Input = [u8; 17];

fn make_inputs(n: usize) -> Vec<Input> {
    let alphabet = b"abcdefghijklmnop";
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let mut item = [0u8; 17];
        for j in 0..16 {
            item[j] = alphabet[(i + j) % alphabet.len()];
        }
        item[16] = 0;
        out.push(item);
    }
    out
}

fn bench_candidate_core_ffi_c_str_from_bytes_with_nul_unchecked_388(c: &mut Criterion) {
    let inputs = make_inputs(IN_LEN);
    let param = format!("in{}_ops{}", IN_LEN, OPS);
    let mut g = c.benchmark_group("candidate_core_ffi_c_str_from_bytes_with_nul_unchecked_388");
    g.throughput(Throughput::Elements(OPS as u64));
    g.warm_up_time(Duration::from_secs(3));
    g.sample_size(80);
    g.measurement_time(Duration::from_secs(5));
    g.bench_with_input(BenchmarkId::new("safe_from_bytes_with_nul", &param), &inputs, |b, inputs| {
        b.iter(|| {
            let ins = black_box(&inputs[..]);
            for t in 0..OPS {
                let v = black_box(&ins[t & IN_MASK][..]);
                let s = CStr::from_bytes_with_nul(v).unwrap();
                black_box((s.as_ptr() as usize as u64) ^ (s.to_bytes().len() as u64));
            }
        })
    });
    g.bench_with_input(BenchmarkId::new("unsafe_from_bytes_with_nul_unchecked", &param), &inputs, |b, inputs| {
        b.iter(|| {
            let ins = black_box(&inputs[..]);
            for t in 0..OPS {
                let v = black_box(&ins[t & IN_MASK][..]);
                let s = unsafe { CStr::from_bytes_with_nul_unchecked(v) };
                black_box((s.as_ptr() as usize as u64) ^ (s.to_bytes().len() as u64));
            }
        })
    });
    g.finish();
}

criterion_group!(benches, bench_candidate_core_ffi_c_str_from_bytes_with_nul_unchecked_388);
criterion_main!(benches);
