use ::libc;
pub type int32_t = libc::c_int;
pub type int32 = int32_t;
pub type TexWrapMode = int32;
#[no_mangle]
pub static mut TexWrapMode_Clamp: TexWrapMode = 0x812f as libc::c_int;
#[no_mangle]
pub static mut TexWrapMode_MirrorClamp: TexWrapMode = 0x8743 as libc::c_int;
#[no_mangle]
pub static mut TexWrapMode_MirrorRepeat: TexWrapMode = 0x8370 as libc::c_int;
#[no_mangle]
pub static mut TexWrapMode_Repeat: TexWrapMode = 0x2901 as libc::c_int;
