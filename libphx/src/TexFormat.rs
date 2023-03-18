use crate::internal::Memory::*;
use glam::Vec3;
use libc;
pub type TexFormat = i32;

#[no_mangle]
pub static TexFormat_R8: TexFormat = 0x8229_i32;

#[no_mangle]
pub static TexFormat_R16: TexFormat = 0x822a_i32;

#[no_mangle]
pub static TexFormat_R16F: TexFormat = 0x822d_i32;

#[no_mangle]
pub static TexFormat_R32F: TexFormat = 0x822e_i32;

#[no_mangle]
pub static TexFormat_RG8: TexFormat = 0x822b_i32;

#[no_mangle]
pub static TexFormat_RG16: TexFormat = 0x822c_i32;

#[no_mangle]
pub static TexFormat_RG16F: TexFormat = 0x822f_i32;

#[no_mangle]
pub static TexFormat_RG32F: TexFormat = 0x8230_i32;

#[no_mangle]
pub static TexFormat_RGB8: TexFormat = 0x8051_i32;

#[no_mangle]
pub static TexFormat_RGBA8: TexFormat = 0x8058_i32;

#[no_mangle]
pub static TexFormat_RGBA16: TexFormat = 0x805b_i32;

#[no_mangle]
pub static TexFormat_RGBA16F: TexFormat = 0x881a_i32;

#[no_mangle]
pub static TexFormat_RGBA32F: TexFormat = 0x8814_i32;

#[no_mangle]
pub static TexFormat_Depth16: TexFormat = 0x81a5_i32;

#[no_mangle]
pub static TexFormat_Depth24: TexFormat = 0x81a6_i32;

#[no_mangle]
pub static TexFormat_Depth32F: TexFormat = 0x8cac_i32;

#[no_mangle]
pub unsafe extern "C" fn TexFormat_Components(mut this: TexFormat) -> i32 {
    match this {
        33321 | 33322 | 33325 | 33326 | 33189 | 33190 | 36012 => return 1_i32,
        33323 | 33324 | 33327 | 33328 => return 2_i32,
        32849 => return 3_i32,
        32856 | 32859 | 34842 | 34836 => return 4_i32,
        _ => {}
    }
    return 0_i32;
}

#[no_mangle]
pub unsafe extern "C" fn TexFormat_GetSize(mut this: TexFormat) -> i32 {
    match this {
        33321 => return 1_i32,
        33322 | 33325 | 33323 | 33189 => return 2_i32,
        32849 | 33190 => return 3_i32,
        33326 | 33324 | 33327 | 32856 | 36012 => return 4_i32,
        33328 | 32859 | 34842 => return 8_i32,
        34836 => return 16_i32,
        _ => {}
    }
    return 0_i32;
}

#[no_mangle]
pub unsafe extern "C" fn TexFormat_IsColor(mut this: TexFormat) -> bool {
    match this {
        33189 | 33190 | 36012 => return 0_i32 != 0,
        _ => {}
    }
    return 1_i32 != 0;
}

#[no_mangle]
pub unsafe extern "C" fn TexFormat_IsDepth(mut this: TexFormat) -> bool {
    match this {
        33189 | 33190 | 36012 => return 1_i32 != 0,
        _ => {}
    }
    return 0_i32 != 0;
}

#[no_mangle]
pub unsafe extern "C" fn TexFormat_IsValid(mut this: TexFormat) -> bool {
    match this {
        33321 | 33322 | 33325 | 33326 | 33323 | 33324 | 33327 | 33328 | 32849 | 32856 | 32859
        | 34842 | 34836 | 33189 | 33190 | 36012 => return 1_i32 != 0,
        _ => {}
    }
    return 0_i32 != 0;
}
