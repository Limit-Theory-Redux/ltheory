use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;
pub type HatDir = i32;

#[no_mangle]
pub static HatDir_Centered: HatDir = 0_i32;

#[no_mangle]
pub static HatDir_Up: HatDir = 0x1_i32;

#[no_mangle]
pub static HatDir_Right: HatDir = 0x2_i32;

#[no_mangle]
pub static HatDir_Down: HatDir = 0x4_i32;

#[no_mangle]
pub static HatDir_Left: HatDir = 0x8_i32;

#[no_mangle]
pub static HatDir_RightUp: HatDir = 0x2_i32 | 0x1_i32;

#[no_mangle]
pub static HatDir_RightDown: HatDir = 0x2_i32 | 0x4_i32;

#[no_mangle]
pub static HatDir_LeftUp: HatDir = 0x8_i32 | 0x1_i32;

#[no_mangle]
pub static HatDir_LeftDown: HatDir = 0x8_i32 | 0x4_i32;
