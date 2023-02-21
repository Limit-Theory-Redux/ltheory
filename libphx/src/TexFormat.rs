use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
pub type int32_t = libc::c_int;
pub type int32 = int32_t;
pub type TexFormat = int32;
#[no_mangle]
pub static mut TexFormat_R8: TexFormat = 0x8229 as libc::c_int;
#[no_mangle]
pub static mut TexFormat_R16: TexFormat = 0x822a as libc::c_int;
#[no_mangle]
pub static mut TexFormat_R16F: TexFormat = 0x822d as libc::c_int;
#[no_mangle]
pub static mut TexFormat_R32F: TexFormat = 0x822e as libc::c_int;
#[no_mangle]
pub static mut TexFormat_RG8: TexFormat = 0x822b as libc::c_int;
#[no_mangle]
pub static mut TexFormat_RG16: TexFormat = 0x822c as libc::c_int;
#[no_mangle]
pub static mut TexFormat_RG16F: TexFormat = 0x822f as libc::c_int;
#[no_mangle]
pub static mut TexFormat_RG32F: TexFormat = 0x8230 as libc::c_int;
#[no_mangle]
pub static mut TexFormat_RGB8: TexFormat = 0x8051 as libc::c_int;
#[no_mangle]
pub static mut TexFormat_RGBA8: TexFormat = 0x8058 as libc::c_int;
#[no_mangle]
pub static mut TexFormat_RGBA16: TexFormat = 0x805b as libc::c_int;
#[no_mangle]
pub static mut TexFormat_RGBA16F: TexFormat = 0x881a as libc::c_int;
#[no_mangle]
pub static mut TexFormat_RGBA32F: TexFormat = 0x8814 as libc::c_int;
#[no_mangle]
pub static mut TexFormat_Depth16: TexFormat = 0x81a5 as libc::c_int;
#[no_mangle]
pub static mut TexFormat_Depth24: TexFormat = 0x81a6 as libc::c_int;
#[no_mangle]
pub static mut TexFormat_Depth32F: TexFormat = 0x8cac as libc::c_int;
#[no_mangle]
pub unsafe extern "C" fn TexFormat_Components(mut this: TexFormat) -> libc::c_int {
    match this {
        33321 | 33322 | 33325 | 33326 | 33189 | 33190 | 36012 => return 1 as libc::c_int,
        33323 | 33324 | 33327 | 33328 => return 2 as libc::c_int,
        32849 => return 3 as libc::c_int,
        32856 | 32859 | 34842 | 34836 => return 4 as libc::c_int,
        _ => {}
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn TexFormat_GetSize(mut this: TexFormat) -> libc::c_int {
    match this {
        33321 => return 1 as libc::c_int,
        33322 | 33325 | 33323 | 33189 => return 2 as libc::c_int,
        32849 | 33190 => return 3 as libc::c_int,
        33326 | 33324 | 33327 | 32856 | 36012 => return 4 as libc::c_int,
        33328 | 32859 | 34842 => return 8 as libc::c_int,
        34836 => return 16 as libc::c_int,
        _ => {}
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn TexFormat_IsColor(mut this: TexFormat) -> bool {
    match this {
        33189 | 33190 | 36012 => return 0 as libc::c_int != 0,
        _ => {}
    }
    return 1 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn TexFormat_IsDepth(mut this: TexFormat) -> bool {
    match this {
        33189 | 33190 | 36012 => return 1 as libc::c_int != 0,
        _ => {}
    }
    return 0 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn TexFormat_IsValid(mut this: TexFormat) -> bool {
    match this {
        33321 | 33322 | 33325 | 33326 | 33323 | 33324 | 33327 | 33328 | 32849 | 32856
        | 32859 | 34842 | 34836 | 33189 | 33190 | 36012 => return 1 as libc::c_int != 0,
        _ => {}
    }
    return 0 as libc::c_int != 0;
}
