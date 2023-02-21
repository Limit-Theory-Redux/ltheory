use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
pub type HatDir = i32;
#[no_mangle]
pub static mut HatDir_Centered: HatDir = 0 as libc::c_int;
#[no_mangle]
pub static mut HatDir_Up: HatDir = 0x1 as libc::c_int;
#[no_mangle]
pub static mut HatDir_Right: HatDir = 0x2 as libc::c_int;
#[no_mangle]
pub static mut HatDir_Down: HatDir = 0x4 as libc::c_int;
#[no_mangle]
pub static mut HatDir_Left: HatDir = 0x8 as libc::c_int;
#[no_mangle]
pub static mut HatDir_RightUp: HatDir = 0x2 as libc::c_int | 0x1 as libc::c_int;
#[no_mangle]
pub static mut HatDir_RightDown: HatDir = 0x2 as libc::c_int | 0x4 as libc::c_int;
#[no_mangle]
pub static mut HatDir_LeftUp: HatDir = 0x8 as libc::c_int | 0x1 as libc::c_int;
#[no_mangle]
pub static mut HatDir_LeftDown: HatDir = 0x8 as libc::c_int | 0x4 as libc::c_int;
