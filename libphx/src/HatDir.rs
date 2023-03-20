use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;
pub type HatDir = i32;

#[no_mangle]
pub static HatDir_Centered: HatDir = 0;

#[no_mangle]
pub static HatDir_Up: HatDir = 0x1;

#[no_mangle]
pub static HatDir_Right: HatDir = 0x2;

#[no_mangle]
pub static HatDir_Down: HatDir = 0x4;

#[no_mangle]
pub static HatDir_Left: HatDir = 0x8;

#[no_mangle]
pub static HatDir_RightUp: HatDir = 0x2 | 0x1;

#[no_mangle]
pub static HatDir_RightDown: HatDir = 0x2 | 0x4;

#[no_mangle]
pub static HatDir_LeftUp: HatDir = 0x8 | 0x1;

#[no_mangle]
pub static HatDir_LeftDown: HatDir = 0x8 | 0x4;
