use crate::internal::Memory::*;
use crate::GamepadAxis::*;
use crate::GamepadButton::*;
use glam::Vec3;
use libc;
use sdl2_sys::*;

extern "C" {
    fn TimeStamp_Get() -> TimeStamp;
    fn TimeStamp_GetElapsed(start: TimeStamp) -> f64;
}
pub type __i64_t = i64;
pub type __darwin_off_t = __i64_t;
pub type cstr = *const libc::c_char;
pub type TimeStamp = u64;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Gamepad {
    pub gamepadList_prev: *mut *mut Gamepad,
    pub gamepadList_next: *mut Gamepad,
    pub handle: *mut SDL_GameController,
    pub lastActive: TimeStamp,
    pub axisState: [f64; 6],
    pub axisLast: [f64; 6],
    pub deadzone: [f64; 6],
    pub buttonState: [bool; 15],
    pub buttonLast: [bool; 15],
}

static mut gamepadList: *mut Gamepad = 0 as *const Gamepad as *mut Gamepad;
unsafe extern "C" fn Gamepad_UpdateState(mut this: *mut Gamepad) {
    let now: TimeStamp = TimeStamp_Get();
    let mut i = GamepadAxis_BEGIN as i32;
    while i <= (GamepadAxis_END as i32) {
        let mut state: f64 = Gamepad_GetAxis(this, std::mem::transmute(i));
        if (*this).axisState[i as usize] != state {
            (*this).lastActive = now;
        }
        (*this).axisLast[i as usize] = (*this).axisState[i as usize];
        (*this).axisState[i as usize] = state;
        i += 1;
    }
    i = GamepadButton_BEGIN as i32;
    while i <= (GamepadButton_END as i32) {
        let mut state_0: bool = Gamepad_GetButton(this, std::mem::transmute(i));
        if (*this).buttonState[i as usize] as i32 != state_0 as i32 {
            (*this).lastActive = now;
        }
        (*this).buttonLast[i as usize] = (*this).buttonState[i as usize];
        (*this).buttonState[i as usize] = state_0;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_CanOpen(mut index: i32) -> bool {
    return SDL_IsGameController(index) == SDL_bool::SDL_TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_Open(mut index: i32) -> *mut Gamepad {
    let mut handle: *mut SDL_GameController = SDL_GameControllerOpen(index);
    if handle.is_null() {
        return 0 as *mut Gamepad;
    }
    let mut this: *mut Gamepad =
        MemAllocZero(::core::mem::size_of::<Gamepad>() as usize) as *mut Gamepad;
    (*this).handle = handle;
    (*this).lastActive = TimeStamp_Get();
    (*this).gamepadList_prev = &mut gamepadList;
    (*this).gamepadList_next = gamepadList;
    if !gamepadList.is_null() {
        (*gamepadList).gamepadList_prev = &mut (*this).gamepadList_next;
    }
    gamepadList = this;
    Gamepad_UpdateState(this);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_Close(mut this: *mut Gamepad) {
    *(*this).gamepadList_prev = (*this).gamepadList_next;
    if !((*this).gamepadList_next).is_null() {
        (*(*this).gamepadList_next).gamepadList_prev = (*this).gamepadList_prev;
    }
    SDL_GameControllerClose((*this).handle);
    MemFree(this as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_AddMappings(mut file: cstr) -> i32 {
    return SDL_GameControllerAddMappingsFromRW(
        SDL_RWFromFile(file, b"rb\0" as *const u8 as *const libc::c_char),
        1 as i32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetAxis(mut this: *mut Gamepad, mut axis: GamepadAxis) -> f64 {
    let mut value: f64 = SDL_GameControllerGetAxis((*this).handle, axis as SDL_GameControllerAxis)
        as f64
        / 32767.0f64;
    let mut deadzone: f64 = (*this).deadzone[axis as usize];
    if value > deadzone {
        return (value - deadzone) / (1.0f64 - deadzone);
    }
    if value < -deadzone {
        return (value + deadzone) / (1.0f64 - deadzone);
    }
    return 0.0f64;
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetAxisDelta(
    mut this: *mut Gamepad,
    mut axis: GamepadAxis,
) -> f64 {
    return (*this).axisState[axis as usize] - (*this).axisLast[axis as usize];
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetButton(
    mut this: *mut Gamepad,
    mut button: GamepadButton,
) -> bool {
    return SDL_GameControllerGetButton((*this).handle, button as SDL_GameControllerButton) as i32
        == 1 as i32;
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetButtonPressed(
    mut this: *mut Gamepad,
    mut button: GamepadButton,
) -> f64 {
    return if (*this).buttonState[button as usize] as i32 != 0
        && !(*this).buttonLast[button as usize]
    {
        1.0f64
    } else {
        0.0f64
    };
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetButtonReleased(
    mut this: *mut Gamepad,
    mut button: GamepadButton,
) -> f64 {
    return if !(*this).buttonState[button as usize]
        && (*this).buttonLast[button as usize] as i32 != 0
    {
        1.0f64
    } else {
        0.0f64
    };
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetIdleTime(mut this: *mut Gamepad) -> f64 {
    return TimeStamp_GetElapsed((*this).lastActive);
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetID(mut this: *mut Gamepad) -> i32 {
    let mut joystick: *mut SDL_Joystick = SDL_GameControllerGetJoystick((*this).handle);
    if joystick.is_null() {
        return -(1 as i32);
    }
    return SDL_JoystickInstanceID(joystick);
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetName(mut this: *mut Gamepad) -> cstr {
    return SDL_GameControllerName((*this).handle);
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_IsConnected(mut this: *mut Gamepad) -> bool {
    return SDL_GameControllerGetAttached((*this).handle) == SDL_bool::SDL_TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_SetDeadzone(
    mut this: *mut Gamepad,
    mut axis: GamepadAxis,
    mut deadzone: f64,
) {
    (*this).deadzone[axis as usize] = deadzone;
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_Update() {
    let mut this: *mut Gamepad = gamepadList;
    while !this.is_null() {
        Gamepad_UpdateState(this);
        this = (*this).gamepadList_next;
    }
}
