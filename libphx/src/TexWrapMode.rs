use crate::internal::Memory::*;
use glam::Vec3;
use libc;
pub type TexWrapMode = i32;

#[no_mangle]
pub static mut TexWrapMode_Clamp: TexWrapMode = 0x812f as i32;

#[no_mangle]
pub static mut TexWrapMode_MirrorClamp: TexWrapMode = 0x8743 as i32;

#[no_mangle]
pub static mut TexWrapMode_MirrorRepeat: TexWrapMode = 0x8370 as i32;

#[no_mangle]
pub static mut TexWrapMode_Repeat: TexWrapMode = 0x2901 as i32;
