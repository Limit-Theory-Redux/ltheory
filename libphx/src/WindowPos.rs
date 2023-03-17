use crate::internal::Memory::*;
use glam::Vec3;
use libc;
pub type WindowPos = i32;

#[no_mangle]
pub static WindowPos_Centered: WindowPos = (0x2fff0000 as u32 | 0 as i32 as u32) as WindowPos;

#[no_mangle]
pub static WindowPos_Default: WindowPos = (0x1fff0000 as u32 | 0 as i32 as u32) as WindowPos;
