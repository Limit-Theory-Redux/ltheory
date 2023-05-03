use crate::phx::internal::Memory::*;
use crate::phx::Common::*;
use crate::phx::Math::Vec3;
use sdl2_sys::*;

pub type GamepadAxis = SDL_GameControllerAxis;

#[no_mangle]
pub static GamepadAxis_BEGIN: GamepadAxis = SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTX;

#[no_mangle]
pub static GamepadAxis_LeftX: GamepadAxis = SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTX;

#[no_mangle]
pub static GamepadAxis_LeftY: GamepadAxis = SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTY;

#[no_mangle]
pub static GamepadAxis_RightX: GamepadAxis = SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTX;

#[no_mangle]
pub static GamepadAxis_RightY: GamepadAxis = SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTY;

#[no_mangle]
pub static GamepadAxis_LTrigger: GamepadAxis =
    SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERLEFT;

#[no_mangle]
pub static GamepadAxis_RTrigger: GamepadAxis =
    SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERRIGHT;

#[no_mangle]
pub static GamepadAxis_END: GamepadAxis = SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERRIGHT;

pub const GamepadAxis_SIZE: usize = 6;
