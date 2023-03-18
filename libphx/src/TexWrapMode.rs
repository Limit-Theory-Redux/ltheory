use crate::internal::Memory::*;
use crate::Math::Vec3;
use libc;
pub type TexWrapMode = i32;

#[no_mangle]
pub static TexWrapMode_Clamp: TexWrapMode = 0x812f_i32;

#[no_mangle]
pub static TexWrapMode_MirrorClamp: TexWrapMode = 0x8743_i32;

#[no_mangle]
pub static TexWrapMode_MirrorRepeat: TexWrapMode = 0x8370_i32;

#[no_mangle]
pub static TexWrapMode_Repeat: TexWrapMode = 0x2901_i32;
