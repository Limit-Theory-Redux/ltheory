use crate::phx::internal::Memory::*;
use crate::phx::Common::*;
use crate::phx::GamepadAxis::*;
use crate::phx::GamepadButton::*;

use crate::phx::TimeStamp::*;
use sdl2_sys::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Gamepad {
    pub gamepadList_prev: *mut *mut Gamepad,
    pub gamepadList_next: *mut Gamepad,
    pub handle: *mut SDL_GameController,
    pub lastActive: TimeStamp,
    pub axisState: [f64; GamepadAxis_SIZE],
    pub axisLast: [f64; GamepadAxis_SIZE],
    pub deadzone: [f64; GamepadAxis_SIZE],
    pub buttonState: [bool; GamepadButton_SIZE],
    pub buttonLast: [bool; GamepadButton_SIZE],
}

static mut gamepadList: *mut Gamepad = std::ptr::null_mut();

unsafe extern "C" fn Gamepad_UpdateState(this: &mut Gamepad) {
    let now: TimeStamp = TimeStamp_Get();
    for i in (GamepadAxis_BEGIN as usize)..=(GamepadAxis_END as usize) {
        let state: f64 = Gamepad_GetAxis(this, std::mem::transmute(i as u32));
        if this.axisState[i] != state {
            this.lastActive = now;
        }
        this.axisLast[i] = this.axisState[i];
        this.axisState[i] = state;
    }

    for i in (GamepadButton_BEGIN as usize)..=(GamepadButton_END as usize) {
        let state: bool = Gamepad_GetButton(this, std::mem::transmute(i as u32));
        if this.buttonState[i] as i32 != state as i32 {
            this.lastActive = now;
        }
        this.buttonLast[i] = this.buttonState[i];
        this.buttonState[i] = state;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Gamepad_CanOpen(index: i32) -> bool {
    SDL_IsGameController(index) == SDL_bool::SDL_TRUE
}

#[no_mangle]
pub unsafe extern "C" fn Gamepad_Open(index: i32) -> *mut Gamepad {
    let handle: *mut SDL_GameController = SDL_GameControllerOpen(index);
    if handle.is_null() {
        return std::ptr::null_mut();
    }
    let this = MemNewZero!(Gamepad);
    (*this).handle = handle;
    (*this).lastActive = TimeStamp_Get();
    (*this).gamepadList_prev = &mut gamepadList;
    (*this).gamepadList_next = gamepadList;
    if !gamepadList.is_null() {
        (*gamepadList).gamepadList_prev = &mut (*this).gamepadList_next;
    }
    gamepadList = this;
    Gamepad_UpdateState(&mut *this);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Gamepad_Close(this: *mut Gamepad) {
    *(*this).gamepadList_prev = (*this).gamepadList_next;
    if !((*this).gamepadList_next).is_null() {
        (*(*this).gamepadList_next).gamepadList_prev = (*this).gamepadList_prev;
    }
    SDL_GameControllerClose((*this).handle);
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn Gamepad_AddMappings(file: *const libc::c_char) -> i32 {
    SDL_GameControllerAddMappingsFromRW(SDL_RWFromFile(file, c_str!("rb")), 1)
}

#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetAxis(this: &mut Gamepad, axis: GamepadAxis) -> f64 {
    let value: f64 =
        SDL_GameControllerGetAxis(this.handle, axis as SDL_GameControllerAxis) as f64 / 32767.0f64;
    let deadzone: f64 = this.deadzone[axis as usize];
    if value > deadzone {
        return (value - deadzone) / (1.0f64 - deadzone);
    }
    if value < -deadzone {
        return (value + deadzone) / (1.0f64 - deadzone);
    }
    0.0f64
}

#[no_mangle]
pub extern "C" fn Gamepad_GetAxisDelta(this: &mut Gamepad, axis: GamepadAxis) -> f64 {
    this.axisState[axis as usize] - this.axisLast[axis as usize]
}

#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetButton(this: &mut Gamepad, button: GamepadButton) -> bool {
    SDL_GameControllerGetButton(this.handle, button as SDL_GameControllerButton) as i32 == 1
}

#[no_mangle]
pub extern "C" fn Gamepad_GetButtonPressed(
    this: &mut Gamepad,
    button: GamepadButton,
) -> f64 {
    if this.buttonState[button as usize] as i32 != 0 && !this.buttonLast[button as usize] {
        1.0f64
    } else {
        0.0f64
    }
}

#[no_mangle]
pub extern "C" fn Gamepad_GetButtonReleased(
    this: &mut Gamepad,
    button: GamepadButton,
) -> f64 {
    if !this.buttonState[button as usize] && this.buttonLast[button as usize] as i32 != 0 {
        1.0f64
    } else {
        0.0f64
    }
}

#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetIdleTime(this: &mut Gamepad) -> f64 {
    TimeStamp_GetElapsed(this.lastActive)
}

#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetID(this: &mut Gamepad) -> i32 {
    let joystick: *mut SDL_Joystick = SDL_GameControllerGetJoystick(this.handle);
    if joystick.is_null() {
        return -1;
    }
    SDL_JoystickInstanceID(joystick)
}

#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetName(this: &mut Gamepad) -> *const libc::c_char {
    SDL_GameControllerName(this.handle)
}

#[no_mangle]
pub unsafe extern "C" fn Gamepad_IsConnected(this: &mut Gamepad) -> bool {
    SDL_GameControllerGetAttached(this.handle) == SDL_bool::SDL_TRUE
}

#[no_mangle]
pub extern "C" fn Gamepad_SetDeadzone(this: &mut Gamepad, axis: GamepadAxis, deadzone: f64) {
    this.deadzone[axis as usize] = deadzone;
}

#[no_mangle]
pub unsafe extern "C" fn Gamepad_Update() {
    let mut this: *mut Gamepad = gamepadList;
    while !this.is_null() {
        Gamepad_UpdateState(&mut *this);
        this = (*this).gamepadList_next;
    }
}
