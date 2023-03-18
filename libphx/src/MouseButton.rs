use crate::internal::Memory::*;
use crate::Math::Vec3;
use libc;
pub type MouseButton = i32;

#[no_mangle]
pub static MouseButton_Left: MouseButton = 1_i32;

#[no_mangle]
pub static MouseButton_Middle: MouseButton = 2_i32;

#[no_mangle]
pub static MouseButton_Right: MouseButton = 3_i32;

#[no_mangle]
pub static MouseButton_X1: MouseButton = 4_i32;

#[no_mangle]
pub static MouseButton_X2: MouseButton = 5_i32;
