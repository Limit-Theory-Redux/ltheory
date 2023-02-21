use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
pub type MouseButton = i32;
#[no_mangle]
pub static mut MouseButton_Left: MouseButton = 1 as libc::c_int;
#[no_mangle]
pub static mut MouseButton_Middle: MouseButton = 2 as libc::c_int;
#[no_mangle]
pub static mut MouseButton_Right: MouseButton = 3 as libc::c_int;
#[no_mangle]
pub static mut MouseButton_X1: MouseButton = 4 as libc::c_int;
#[no_mangle]
pub static mut MouseButton_X2: MouseButton = 5 as libc::c_int;
