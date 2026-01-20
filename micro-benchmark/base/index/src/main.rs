#![feature(portable_simd)]
#![feature(ptr_alignment_type)]
#![feature(ascii_char)]

use std::ptr::Alignment;
use std::alloc::Layout;
use std::simd::Mask;

fn main() { 
    // get & get_mut
    let mut a = [1, 2, 3, 4, 5]; 
    let mut i = 0; 
    let _v = a.get(i); 
    let mut _v_mut = a.get_mut(i); 

    // ===== slice: get (range) =====
    let a_get_r: [u32; 5] = [1, 2, 3, 4, 5];
    let s_get_r: &[u32] = &a_get_r;
    let _v_get_r = s_get_r.get(1..4);

    // ===== slice: get_mut (range) =====
    let mut a_get_mut_r: [u32; 5] = [1, 2, 3, 4, 5];
    let s_get_mut_r: &mut [u32] = &mut a_get_mut_r;
    let _v_get_mut_r = s_get_mut_r.get_mut(1..4);


    // spilt_at
    let mut slice = [1,2,3,4,5];
    let _s = slice.split_at(2);
    let mut _s_m = slice.split_at_mut(2);

    let mut slice = [1,2,3,4,5];
    slice.swap(0,2);

    let mut str:[u8;5] = [44,44,44,44,44];
    let chr: u32 = 45;
    let _s = str.as_ascii();
    let _c = std::char::from_u32(chr);

    // ===== [u8]::as_ascii =====
    let bytes_slice: &[u8] = b"world";
    let _ascii_slice = bytes_slice.as_ascii();

    // ===== str::get (range) =====
    let sbuf: [u8; 5] = *b"hello";
    let st: &str = unsafe {
        let p: *const [u8; 5] = &sbuf;
        let p: *const [u8] = p as *const [u8];
        &*(p as *const str)
    };
    let _st_get = st.get(1..4);

    // ===== str::get_mut (range) =====
    let mut sbuf_mut: [u8; 5] = *b"hello";
    let st_mut: &mut str = unsafe {
        let p: *mut [u8; 5] = &mut sbuf_mut;
        let p: *mut [u8] = p as *mut [u8];
        &mut *(p as *mut str)
    };
    let _st_get_mut = st_mut.get_mut(1..4);

    // ===== ptr::Alignment::new =====
    let _al = Alignment::new(8);

    // ===== Layout::from_size_align =====
    let _layout = Layout::from_size_align(16, 8);


    // ===== std::simd::Mask<T, N>::test / set =====
    let mut m: Mask<i8, 8> = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
    let _b = m.test(0);
    m.set(1, true);


}