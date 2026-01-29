#![allow(unused_variables, unused_mut)]

fn main() {
    // ===== signed integers: checked_add / checked_sub / checked_mul / checked_neg / checked_shl / checked_shr =====
    let x_i32: i32 = 123;
    let y_i32: i32 = 456;
    let sh_i32: u32 = 3;

    let _i32_add = x_i32.checked_add(y_i32);
    let _i32_sub = x_i32.checked_sub(y_i32);
    let _i32_mul = x_i32.checked_mul(y_i32);
    let _i32_neg = x_i32.checked_neg();
    let _i32_shl = x_i32.checked_shl(sh_i32);
    let _i32_shr = x_i32.checked_shr(sh_i32);

    // ===== unsigned integers: checked_add / checked_sub / checked_mul / checked_shl / checked_shr =====
    let x_u32: u32 = 123;
    let y_u32: u32 = 456;
    let sh_u32: u32 = 5;

    let _u32_add = x_u32.checked_add(y_u32);
    let _u32_sub = x_u32.checked_sub(y_u32);
    let _u32_mul = x_u32.checked_mul(y_u32);
    let _u32_shl = x_u32.checked_shl(sh_u32);
    let _u32_shr = x_u32.checked_shr(sh_u32);

    // ===== NonZero: checked_add (by underlying int) / checked_mul (by NonZero) =====
    // Use a constant to avoid introducing extra constructor calls.
    let nz = core::num::NonZeroU32::MIN;

    let _nz_add = nz.checked_add(1u32);
    let _nz_mul = nz.checked_mul(core::num::NonZeroU32::MIN);
}
