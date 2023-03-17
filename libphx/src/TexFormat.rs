use crate::internal::Memory::*;
use glam::Vec3;
use libc;
pub type TexFormat = i32;

#[no_mangle]
pub static mut TexFormat_R8: TexFormat = 0x8229 as i32;

#[no_mangle]
pub static mut TexFormat_R16: TexFormat = 0x822a as i32;

#[no_mangle]
pub static mut TexFormat_R16F: TexFormat = 0x822d as i32;

#[no_mangle]
pub static mut TexFormat_R32F: TexFormat = 0x822e as i32;

#[no_mangle]
pub static mut TexFormat_RG8: TexFormat = 0x822b as i32;

#[no_mangle]
pub static mut TexFormat_RG16: TexFormat = 0x822c as i32;

#[no_mangle]
pub static mut TexFormat_RG16F: TexFormat = 0x822f as i32;

#[no_mangle]
pub static mut TexFormat_RG32F: TexFormat = 0x8230 as i32;

#[no_mangle]
pub static mut TexFormat_RGB8: TexFormat = 0x8051 as i32;

#[no_mangle]
pub static mut TexFormat_RGBA8: TexFormat = 0x8058 as i32;

#[no_mangle]
pub static mut TexFormat_RGBA16: TexFormat = 0x805b as i32;

#[no_mangle]
pub static mut TexFormat_RGBA16F: TexFormat = 0x881a as i32;

#[no_mangle]
pub static mut TexFormat_RGBA32F: TexFormat = 0x8814 as i32;

#[no_mangle]
pub static mut TexFormat_Depth16: TexFormat = 0x81a5 as i32;

#[no_mangle]
pub static mut TexFormat_Depth24: TexFormat = 0x81a6 as i32;

#[no_mangle]
pub static mut TexFormat_Depth32F: TexFormat = 0x8cac as i32;

#[no_mangle]
pub unsafe extern "C" fn TexFormat_Components(mut this: TexFormat) -> i32 {
    match this {
        33321 | 33322 | 33325 | 33326 | 33189 | 33190 | 36012 => return 1 as i32,
        33323 | 33324 | 33327 | 33328 => return 2 as i32,
        32849 => return 3 as i32,
        32856 | 32859 | 34842 | 34836 => return 4 as i32,
        _ => {}
    }
    return 0 as i32;
}

#[no_mangle]
pub unsafe extern "C" fn TexFormat_GetSize(mut this: TexFormat) -> i32 {
    match this {
        33321 => return 1 as i32,
        33322 | 33325 | 33323 | 33189 => return 2 as i32,
        32849 | 33190 => return 3 as i32,
        33326 | 33324 | 33327 | 32856 | 36012 => return 4 as i32,
        33328 | 32859 | 34842 => return 8 as i32,
        34836 => return 16 as i32,
        _ => {}
    }
    return 0 as i32;
}

#[no_mangle]
pub unsafe extern "C" fn TexFormat_IsColor(mut this: TexFormat) -> bool {
    match this {
        33189 | 33190 | 36012 => return 0 as i32 != 0,
        _ => {}
    }
    return 1 as i32 != 0;
}

#[no_mangle]
pub unsafe extern "C" fn TexFormat_IsDepth(mut this: TexFormat) -> bool {
    match this {
        33189 | 33190 | 36012 => return 1 as i32 != 0,
        _ => {}
    }
    return 0 as i32 != 0;
}

#[no_mangle]
pub unsafe extern "C" fn TexFormat_IsValid(mut this: TexFormat) -> bool {
    match this {
        33321 | 33322 | 33325 | 33326 | 33323 | 33324 | 33327 | 33328 | 32849 | 32856 | 32859
        | 34842 | 34836 | 33189 | 33190 | 36012 => return 1 as i32 != 0,
        _ => {}
    }
    return 0 as i32 != 0;
}
