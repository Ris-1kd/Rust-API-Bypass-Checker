#![allow(unused_variables, unused_mut)]
#![feature(slice_as_chunks)]
#![feature(nonzero_from_mut)]

fn main() {
    // ===== [T]::as_chunks<const N: usize> =====
    // N must be non-zero; len should be a multiple of N for a clean split.
    let a_chunks: [u32; 4] = [1, 2, 3, 4];
    let s_chunks: &[u32] = &a_chunks;
    let (_chunked, _remainder) = s_chunks.as_chunks::<2>();

    // ===== [T]::as_chunks_mut<const N: usize> =====
    let mut a_chunks_mut: [u32; 4] = [1, 2, 3, 4];
    let s_chunks_mut: &mut [u32] = &mut a_chunks_mut;
    let (_chunked_mut, _remainder_mut) = s_chunks_mut.as_chunks_mut::<2>();

    // ===== core::num::NonZero*::new (integer nonzero constructors) =====
    // This exercises the safe checked constructor that returns Option.
    let _nz_u32 = core::num::NonZeroU32::new(1);

    // ===== core::num::NonZero<T>::new (generic nonzero) =====
    // This is the checked constructor (Option-returning).
    let _nz_generic = core::num::NonZero::<u32>::new(1);

    // ===== core::num::NonZero<T>::from_mut =====
    // The referenced value must be non-zero to get Some(&mut NonZero<T>).
    let mut x: u32 = 1;
    let _nz_from_mut = core::num::NonZero::<u32>::from_mut(&mut x);
}
