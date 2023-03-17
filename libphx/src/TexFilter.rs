use crate::internal::Memory::*;
use glam::Vec3;
use libc;
pub type TexFilter = i32;

#[no_mangle]
pub static TexFilter_Point: TexFilter = 0x2600 as i32;

#[no_mangle]
pub static TexFilter_PointMipPoint: TexFilter = 0x2700 as i32;

#[no_mangle]
pub static TexFilter_PointMipLinear: TexFilter = 0x2702 as i32;

#[no_mangle]
pub static TexFilter_Linear: TexFilter = 0x2601 as i32;

#[no_mangle]
pub static TexFilter_LinearMipPoint: TexFilter = 0x2701 as i32;

#[no_mangle]
pub static TexFilter_LinearMipLinear: TexFilter = 0x2703 as i32;
