use std;

// https://internals.rust-lang.org/t/safe-trasnsmute-for-slices-e-g-u64-u32-particularly-simd-types/2871
#[inline(always)]
#[allow(dead_code)] // Only used on 32-bit builds currently
pub fn u32_as_u8<'a>(src: &'a [u32]) -> &'a [u8] {
    unsafe {
        std::slice::from_raw_parts(src.as_ptr() as *mut u8, src.len() * 4)
    }
}

// https://internals.rust-lang.org/t/safe-trasnsmute-for-slices-e-g-u64-u32-particularly-simd-types/2871
#[inline(always)]
#[allow(dead_code)] // Only used on 32-bit builds currently
pub fn u32_as_u8_mut<'a>(src: &'a mut [u32]) -> &'a mut [u8] {
    unsafe {
        std::slice::from_raw_parts_mut(src.as_mut_ptr() as *mut u8,
                                        src.len() * 4)
    }
}
