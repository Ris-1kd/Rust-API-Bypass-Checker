#![allow(unused_variables, unused_mut)]

use std::ffi::CStr;
use std::ffi::CString;

fn main() {
    // ===== CStr::from_bytes_with_nul =====
    let c_bytes: &[u8] = b"hi\0";
    let _cstr_res = CStr::from_bytes_with_nul(c_bytes);

    // ===== str::from_utf8 =====
    let utf8_bytes: &[u8] = b"hello";
    let _s_res = core::str::from_utf8(utf8_bytes);

    // ===== str::from_utf8_mut =====
    let mut utf8_buf: [u8; 5] = *b"world";
    let _s_mut_res = core::str::from_utf8_mut(&mut utf8_buf);

    // ===== CString::new =====
    let _cstring_new_res = CString::new("hello");

    // ===== CString::from_vec_with_nul =====
    let v_with_nul: Vec<u8> = vec![b'h', b'i', 0];
    let _cstring_from_vec_with_nul_res = CString::from_vec_with_nul(v_with_nul);

    // ===== String::from_utf8 =====
    let v_utf8: Vec<u8> = vec![b'h', b'e', b'l', b'l', b'o'];
    let _string_from_utf8_res = String::from_utf8(v_utf8);
}
