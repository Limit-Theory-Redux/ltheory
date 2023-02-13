use ::libc;
use super::internal::Memory::*;
pub type int32_t = libc::c_int;
pub type int32 = int32_t;
pub type GamepadAxis = int32;
pub const SDL_CONTROLLER_AXIS_LEFTX: C2RustUnnamed = 0;
pub const SDL_CONTROLLER_AXIS_LEFTY: C2RustUnnamed = 1;
pub const SDL_CONTROLLER_AXIS_RIGHTX: C2RustUnnamed = 2;
pub const SDL_CONTROLLER_AXIS_RIGHTY: C2RustUnnamed = 3;
pub const SDL_CONTROLLER_AXIS_TRIGGERLEFT: C2RustUnnamed = 4;
pub const SDL_CONTROLLER_AXIS_TRIGGERRIGHT: C2RustUnnamed = 5;
pub type C2RustUnnamed = libc::c_int;
pub const SDL_CONTROLLER_AXIS_MAX: C2RustUnnamed = 6;
pub const SDL_CONTROLLER_AXIS_INVALID: C2RustUnnamed = -1;
#[no_mangle]
pub static mut GamepadAxis_BEGIN: GamepadAxis = SDL_CONTROLLER_AXIS_LEFTX as libc::c_int;
#[no_mangle]
pub static mut GamepadAxis_LeftX: GamepadAxis = SDL_CONTROLLER_AXIS_LEFTX as libc::c_int;
#[no_mangle]
pub static mut GamepadAxis_LeftY: GamepadAxis = SDL_CONTROLLER_AXIS_LEFTY as libc::c_int;
#[no_mangle]
pub static mut GamepadAxis_RightX: GamepadAxis = SDL_CONTROLLER_AXIS_RIGHTX
    as libc::c_int;
#[no_mangle]
pub static mut GamepadAxis_RightY: GamepadAxis = SDL_CONTROLLER_AXIS_RIGHTY
    as libc::c_int;
#[no_mangle]
pub static mut GamepadAxis_LTrigger: GamepadAxis = SDL_CONTROLLER_AXIS_TRIGGERLEFT
    as libc::c_int;
#[no_mangle]
pub static mut GamepadAxis_RTrigger: GamepadAxis = SDL_CONTROLLER_AXIS_TRIGGERRIGHT
    as libc::c_int;
#[no_mangle]
pub static mut GamepadAxis_END: GamepadAxis = SDL_CONTROLLER_AXIS_TRIGGERRIGHT
    as libc::c_int;
