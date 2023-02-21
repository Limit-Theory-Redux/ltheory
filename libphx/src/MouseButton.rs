use crate::internal::Memory::*;
use glam::Vec3;
use libc;
pub type MouseButton = i32;
#[no_mangle]
pub static mut MouseButton_Left: MouseButton = 1 as i32;
#[no_mangle]
pub static mut MouseButton_Middle: MouseButton = 2 as i32;
#[no_mangle]
pub static mut MouseButton_Right: MouseButton = 3 as i32;
#[no_mangle]
pub static mut MouseButton_X1: MouseButton = 4 as i32;
#[no_mangle]
pub static mut MouseButton_X2: MouseButton = 5 as i32;
