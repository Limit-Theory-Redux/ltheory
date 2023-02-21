use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
pub type TexWrapMode = i32;
#[no_mangle]
pub static mut TexWrapMode_Clamp: TexWrapMode = 0x812f as libc::c_int;
#[no_mangle]
pub static mut TexWrapMode_MirrorClamp: TexWrapMode = 0x8743 as libc::c_int;
#[no_mangle]
pub static mut TexWrapMode_MirrorRepeat: TexWrapMode = 0x8370 as libc::c_int;
#[no_mangle]
pub static mut TexWrapMode_Repeat: TexWrapMode = 0x2901 as libc::c_int;
