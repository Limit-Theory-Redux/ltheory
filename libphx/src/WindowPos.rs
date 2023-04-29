use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
pub type WindowPos = i32;

#[no_mangle]
pub static WindowPos_Centered: WindowPos = (0x2fff0000 | 0) as WindowPos;

#[no_mangle]
pub static WindowPos_Default: WindowPos = (0x1fff0000 | 0) as WindowPos;
