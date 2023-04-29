use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
pub type MouseButton = i32;

#[no_mangle]
pub static MouseButton_Left: MouseButton = 1;

#[no_mangle]
pub static MouseButton_Middle: MouseButton = 2;

#[no_mangle]
pub static MouseButton_Right: MouseButton = 3;

#[no_mangle]
pub static MouseButton_X1: MouseButton = 4;

#[no_mangle]
pub static MouseButton_X2: MouseButton = 5;
