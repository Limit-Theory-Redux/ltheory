use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
pub type GamepadAxis = i32;
pub const SDL_CONTROLLER_AXIS_LEFTX: C2RustUnnamed = 0;
pub const SDL_CONTROLLER_AXIS_LEFTY: C2RustUnnamed = 1;
pub const SDL_CONTROLLER_AXIS_RIGHTX: C2RustUnnamed = 2;
pub const SDL_CONTROLLER_AXIS_RIGHTY: C2RustUnnamed = 3;
pub const SDL_CONTROLLER_AXIS_TRIGGERLEFT: C2RustUnnamed = 4;
pub const SDL_CONTROLLER_AXIS_TRIGGERRIGHT: C2RustUnnamed = 5;
pub type C2RustUnnamed = i32;
pub const SDL_CONTROLLER_AXIS_MAX: C2RustUnnamed = 6;
pub const SDL_CONTROLLER_AXIS_INVALID: C2RustUnnamed = -1;
#[no_mangle]
pub static mut GamepadAxis_BEGIN: GamepadAxis = SDL_CONTROLLER_AXIS_LEFTX as i32;
#[no_mangle]
pub static mut GamepadAxis_LeftX: GamepadAxis = SDL_CONTROLLER_AXIS_LEFTX as i32;
#[no_mangle]
pub static mut GamepadAxis_LeftY: GamepadAxis = SDL_CONTROLLER_AXIS_LEFTY as i32;
#[no_mangle]
pub static mut GamepadAxis_RightX: GamepadAxis = SDL_CONTROLLER_AXIS_RIGHTX
    as i32;
#[no_mangle]
pub static mut GamepadAxis_RightY: GamepadAxis = SDL_CONTROLLER_AXIS_RIGHTY
    as i32;
#[no_mangle]
pub static mut GamepadAxis_LTrigger: GamepadAxis = SDL_CONTROLLER_AXIS_TRIGGERLEFT
    as i32;
#[no_mangle]
pub static mut GamepadAxis_RTrigger: GamepadAxis = SDL_CONTROLLER_AXIS_TRIGGERRIGHT
    as i32;
#[no_mangle]
pub static mut GamepadAxis_END: GamepadAxis = SDL_CONTROLLER_AXIS_TRIGGERRIGHT
    as i32;
