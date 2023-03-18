use crate::internal::Memory::*;
use crate::Math::Vec3;
use libc;
pub type TexFilter = i32;

#[no_mangle]
pub static TexFilter_Point: TexFilter = 0x2600_i32;

#[no_mangle]
pub static TexFilter_PointMipPoint: TexFilter = 0x2700_i32;

#[no_mangle]
pub static TexFilter_PointMipLinear: TexFilter = 0x2702_i32;

#[no_mangle]
pub static TexFilter_Linear: TexFilter = 0x2601_i32;

#[no_mangle]
pub static TexFilter_LinearMipPoint: TexFilter = 0x2701_i32;

#[no_mangle]
pub static TexFilter_LinearMipLinear: TexFilter = 0x2703_i32;
