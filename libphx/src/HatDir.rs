use crate::internal::Memory::*;
use glam::Vec3;
use libc;
pub type HatDir = i32;

#[no_mangle]
pub static HatDir_Centered: HatDir = 0 as i32;

#[no_mangle]
pub static HatDir_Up: HatDir = 0x1 as i32;

#[no_mangle]
pub static HatDir_Right: HatDir = 0x2 as i32;

#[no_mangle]
pub static HatDir_Down: HatDir = 0x4 as i32;

#[no_mangle]
pub static HatDir_Left: HatDir = 0x8 as i32;

#[no_mangle]
pub static HatDir_RightUp: HatDir = 0x2 as i32 | 0x1 as i32;

#[no_mangle]
pub static HatDir_RightDown: HatDir = 0x2 as i32 | 0x4 as i32;

#[no_mangle]
pub static HatDir_LeftUp: HatDir = 0x8 as i32 | 0x1 as i32;

#[no_mangle]
pub static HatDir_LeftDown: HatDir = 0x8 as i32 | 0x4 as i32;
