#![allow(unused_variables, unused_mut)]
#![feature(ptr_internals)]

use std::ptr::{NonNull, Unique};

fn main() {
    let mut x: u32 = 1;

    let p_const: *const u32 = &x as *const u32;
    let p_mut: *mut u32 = &mut x as *mut u32;

    // ===== *const T::as_ref (checked, returns Option) =====
    let _r1 = unsafe { p_const.as_ref() };

    // ===== *mut T::as_ref (checked, returns Option) =====
    let _r2 = unsafe { p_mut.as_ref() };

    // ===== *mut T::as_mut (checked, returns Option) =====
    let _r3 = unsafe { p_mut.as_mut() };

    // ===== NonNull::new (checked, returns Option) =====
    let _nn = NonNull::new(p_mut);

    // ===== Unique::new (checked, returns Option) =====
    let _uq = Unique::new(p_mut);
}
