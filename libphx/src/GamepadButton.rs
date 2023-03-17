use crate::internal::Memory::*;
use glam::Vec3;
use libc;
use sdl2_sys::*;

pub type GamepadButton = SDL_GameControllerButton;

#[no_mangle]
pub static mut GamepadButton_BEGIN: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_A;

#[no_mangle]
pub static mut GamepadButton_A: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_A;

#[no_mangle]
pub static mut GamepadButton_B: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_B;

#[no_mangle]
pub static mut GamepadButton_X: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_X;

#[no_mangle]
pub static mut GamepadButton_Y: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_Y;

#[no_mangle]
pub static mut GamepadButton_Back: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_BACK;

#[no_mangle]
pub static mut GamepadButton_Guide: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_GUIDE;

#[no_mangle]
pub static mut GamepadButton_Start: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_START;

#[no_mangle]
pub static mut GamepadButton_LStick: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_LEFTSTICK;

#[no_mangle]
pub static mut GamepadButton_RStick: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_RIGHTSTICK;

#[no_mangle]
pub static mut GamepadButton_LBumper: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_LEFTSHOULDER;

#[no_mangle]
pub static mut GamepadButton_RBumper: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_RIGHTSHOULDER;

#[no_mangle]
pub static mut GamepadButton_Up: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_UP;

#[no_mangle]
pub static mut GamepadButton_Down: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_DOWN;

#[no_mangle]
pub static mut GamepadButton_Left: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_LEFT;

#[no_mangle]
pub static mut GamepadButton_Right: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_RIGHT;

#[no_mangle]
pub static mut GamepadButton_END: GamepadButton = SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_RIGHT;
