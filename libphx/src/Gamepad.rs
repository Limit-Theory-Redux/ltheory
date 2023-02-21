use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
use crate::GamepadAxis::*;
use crate::GamepadButton::*;
extern "C" {
    pub type _SDL_GameController;
    pub type __sFILEX;
    pub type _SDL_Joystick;
    fn SDL_GameControllerGetAttached(
        gamecontroller: *mut SDL_GameController,
    ) -> SDL_bool;
    fn SDL_GameControllerName(
        gamecontroller: *mut SDL_GameController,
    ) -> *const libc::c_char;
    fn SDL_IsGameController(joystick_index: libc::c_int) -> SDL_bool;
    fn SDL_JoystickInstanceID(joystick: *mut SDL_Joystick) -> SDL_JoystickID;
    fn SDL_GameControllerGetJoystick(
        gamecontroller: *mut SDL_GameController,
    ) -> *mut SDL_Joystick;
    fn SDL_GameControllerGetButton(
        gamecontroller: *mut SDL_GameController,
        button: SDL_GameControllerButton,
    ) -> Uint8;
    fn SDL_GameControllerAddMappingsFromRW(
        rw: *mut SDL_RWops,
        freerw: libc::c_int,
    ) -> libc::c_int;
    fn SDL_RWFromFile(
        file: *const libc::c_char,
        mode: *const libc::c_char,
    ) -> *mut SDL_RWops;
    fn SDL_GameControllerClose(gamecontroller: *mut SDL_GameController);
    fn SDL_GameControllerOpen(joystick_index: libc::c_int) -> *mut SDL_GameController;
    fn SDL_GameControllerGetAxis(
        gamecontroller: *mut SDL_GameController,
        axis: SDL_GameControllerAxis,
    ) -> Sint16;
    fn TimeStamp_Get() -> TimeStamp;
    fn TimeStamp_GetElapsed(start: TimeStamp) -> f64;
}
pub type int16_t = libc::c_short;
pub type int32_t = libc::c_int;
pub type int64_t = libc::c_longlong;
pub type uint8_t = libc::c_uchar;
pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type __int64_t = libc::c_longlong;
pub type __darwin_off_t = __int64_t;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint64 = uint64_t;
pub type TimeStamp = uint64;
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
pub type SDL_GameController = _SDL_GameController;
pub type GamepadAxis = int32;
pub type GamepadButton = int32;
pub const SDL_TRUE: SDL_bool = 1;
pub type SDL_bool = libc::c_uint;
pub const SDL_FALSE: SDL_bool = 0;
pub type Uint8 = uint8_t;
pub type SDL_GameControllerButton = libc::c_int;
pub const SDL_CONTROLLER_BUTTON_MAX: SDL_GameControllerButton = 21;
pub const SDL_CONTROLLER_BUTTON_TOUCHPAD: SDL_GameControllerButton = 20;
pub const SDL_CONTROLLER_BUTTON_PADDLE4: SDL_GameControllerButton = 19;
pub const SDL_CONTROLLER_BUTTON_PADDLE3: SDL_GameControllerButton = 18;
pub const SDL_CONTROLLER_BUTTON_PADDLE2: SDL_GameControllerButton = 17;
pub const SDL_CONTROLLER_BUTTON_PADDLE1: SDL_GameControllerButton = 16;
pub const SDL_CONTROLLER_BUTTON_MISC1: SDL_GameControllerButton = 15;
pub const SDL_CONTROLLER_BUTTON_DPAD_RIGHT: SDL_GameControllerButton = 14;
pub const SDL_CONTROLLER_BUTTON_DPAD_LEFT: SDL_GameControllerButton = 13;
pub const SDL_CONTROLLER_BUTTON_DPAD_DOWN: SDL_GameControllerButton = 12;
pub const SDL_CONTROLLER_BUTTON_DPAD_UP: SDL_GameControllerButton = 11;
pub const SDL_CONTROLLER_BUTTON_RIGHTSHOULDER: SDL_GameControllerButton = 10;
pub const SDL_CONTROLLER_BUTTON_LEFTSHOULDER: SDL_GameControllerButton = 9;
pub const SDL_CONTROLLER_BUTTON_RIGHTSTICK: SDL_GameControllerButton = 8;
pub const SDL_CONTROLLER_BUTTON_LEFTSTICK: SDL_GameControllerButton = 7;
pub const SDL_CONTROLLER_BUTTON_START: SDL_GameControllerButton = 6;
pub const SDL_CONTROLLER_BUTTON_GUIDE: SDL_GameControllerButton = 5;
pub const SDL_CONTROLLER_BUTTON_BACK: SDL_GameControllerButton = 4;
pub const SDL_CONTROLLER_BUTTON_Y: SDL_GameControllerButton = 3;
pub const SDL_CONTROLLER_BUTTON_X: SDL_GameControllerButton = 2;
pub const SDL_CONTROLLER_BUTTON_B: SDL_GameControllerButton = 1;
pub const SDL_CONTROLLER_BUTTON_A: SDL_GameControllerButton = 0;
pub const SDL_CONTROLLER_BUTTON_INVALID: SDL_GameControllerButton = -1;
pub type Sint16 = int16_t;
pub type SDL_GameControllerAxis = libc::c_int;
pub const SDL_CONTROLLER_AXIS_MAX: SDL_GameControllerAxis = 6;
pub const SDL_CONTROLLER_AXIS_TRIGGERRIGHT: SDL_GameControllerAxis = 5;
pub const SDL_CONTROLLER_AXIS_TRIGGERLEFT: SDL_GameControllerAxis = 4;
pub const SDL_CONTROLLER_AXIS_RIGHTY: SDL_GameControllerAxis = 3;
pub const SDL_CONTROLLER_AXIS_RIGHTX: SDL_GameControllerAxis = 2;
pub const SDL_CONTROLLER_AXIS_LEFTY: SDL_GameControllerAxis = 1;
pub const SDL_CONTROLLER_AXIS_LEFTX: SDL_GameControllerAxis = 0;
pub const SDL_CONTROLLER_AXIS_INVALID: SDL_GameControllerAxis = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_RWops {
    pub size: Option::<unsafe extern "C" fn(*mut SDL_RWops) -> Sint64>,
    pub seek: Option::<
        unsafe extern "C" fn(*mut SDL_RWops, Sint64, libc::c_int) -> Sint64,
    >,
    pub read: Option::<
        unsafe extern "C" fn(*mut SDL_RWops, *mut libc::c_void, libc::size_t, libc::size_t) -> libc::size_t,
    >,
    pub write: Option::<
        unsafe extern "C" fn(
            *mut SDL_RWops,
            *const libc::c_void,
            libc::size_t,
            libc::size_t,
        ) -> libc::size_t,
    >,
    pub close: Option::<unsafe extern "C" fn(*mut SDL_RWops) -> libc::c_int>,
    pub type_0: Uint32,
    pub hidden: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub stdio: C2RustUnnamed_2,
    pub mem: C2RustUnnamed_1,
    pub unknown: C2RustUnnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub data1: *mut libc::c_void,
    pub data2: *mut libc::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_1 {
    pub base: *mut Uint8,
    pub here: *mut Uint8,
    pub stop: *mut Uint8,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_2 {
    pub autoclose: SDL_bool,
    pub fp: *mut FILE,
}
pub type FILE = __sFILE;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sFILE {
    pub _p: *mut libc::c_uchar,
    pub _r: libc::c_int,
    pub _w: libc::c_int,
    pub _flags: libc::c_short,
    pub _file: libc::c_short,
    pub _bf: __sbuf,
    pub _lbfsize: libc::c_int,
    pub _cookie: *mut libc::c_void,
    pub _close: Option::<unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int>,
    pub _read: Option::<
        unsafe extern "C" fn(
            *mut libc::c_void,
            *mut libc::c_char,
            libc::c_int,
        ) -> libc::c_int,
    >,
    pub _seek: Option::<
        unsafe extern "C" fn(*mut libc::c_void, fpos_t, libc::c_int) -> fpos_t,
    >,
    pub _write: Option::<
        unsafe extern "C" fn(
            *mut libc::c_void,
            *const libc::c_char,
            libc::c_int,
        ) -> libc::c_int,
    >,
    pub _ub: __sbuf,
    pub _extra: *mut __sFILEX,
    pub _ur: libc::c_int,
    pub _ubuf: [libc::c_uchar; 3],
    pub _nbuf: [libc::c_uchar; 1],
    pub _lb: __sbuf,
    pub _blksize: libc::c_int,
    pub _offset: fpos_t,
}
pub type fpos_t = __darwin_off_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sbuf {
    pub _base: *mut libc::c_uchar,
    pub _size: libc::c_int,
}
pub type Uint32 = uint32_t;
pub type Sint64 = int64_t;
pub type SDL_JoystickID = Sint32;
pub type Sint32 = int32_t;
pub type SDL_Joystick = _SDL_Joystick;


static mut gamepadList: *mut Gamepad = 0 as *const Gamepad as *mut Gamepad;
unsafe extern "C" fn Gamepad_UpdateState(mut this: *mut Gamepad) {
    let mut now: TimeStamp = TimeStamp_Get();
    let mut i: GamepadAxis = GamepadAxis_BEGIN;
    while i <= GamepadAxis_END {
        let mut state: f64 = Gamepad_GetAxis(this, i);
        if (*this).axisState[i as usize] != state {
            (*this).lastActive = now;
        }
        (*this).axisLast[i as usize] = (*this).axisState[i as usize];
        (*this).axisState[i as usize] = state;
        i += 1;
    }
    let mut i_0: GamepadButton = GamepadButton_BEGIN;
    while i_0 <= GamepadButton_END {
        let mut state_0: bool = Gamepad_GetButton(this, i_0);
        if (*this).buttonState[i_0 as usize] as libc::c_int != state_0 as libc::c_int {
            (*this).lastActive = now;
        }
        (*this).buttonLast[i_0 as usize] = (*this).buttonState[i_0 as usize];
        (*this).buttonState[i_0 as usize] = state_0;
        i_0 += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_CanOpen(mut index: libc::c_int) -> bool {
    return SDL_IsGameController(index) as libc::c_uint
        == SDL_TRUE as libc::c_int as libc::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_Open(mut index: libc::c_int) -> *mut Gamepad {
    let mut handle: *mut SDL_GameController = SDL_GameControllerOpen(index);
    if handle.is_null() {
        return 0 as *mut Gamepad;
    }
    let mut this: *mut Gamepad = MemAllocZero(
        ::core::mem::size_of::<Gamepad>() as usize,
    ) as *mut Gamepad;
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
pub unsafe extern "C" fn Gamepad_AddMappings(mut file: cstr) -> libc::c_int {
    return SDL_GameControllerAddMappingsFromRW(
        SDL_RWFromFile(file, b"rb\0" as *const u8 as *const libc::c_char),
        1 as libc::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetAxis(
    mut this: *mut Gamepad,
    mut axis: GamepadAxis,
) -> f64 {
    let mut value: f64 = SDL_GameControllerGetAxis(
        (*this).handle,
        axis as SDL_GameControllerAxis,
    ) as f64 / 32767.0f64;
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
    return SDL_GameControllerGetButton(
        (*this).handle,
        button as SDL_GameControllerButton,
    ) as libc::c_int == 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetButtonPressed(
    mut this: *mut Gamepad,
    mut button: GamepadButton,
) -> f64 {
    return if (*this).buttonState[button as usize] as libc::c_int != 0
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
        && (*this).buttonLast[button as usize] as libc::c_int != 0
    {
        1.0f64
    } else {
        0.0f64
    };
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetIdleTime(
    mut this: *mut Gamepad,
) -> f64 {
    return TimeStamp_GetElapsed((*this).lastActive);
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetID(mut this: *mut Gamepad) -> libc::c_int {
    let mut joystick: *mut SDL_Joystick = SDL_GameControllerGetJoystick(
        (*this).handle,
    );
    if joystick.is_null() {
        return -(1 as libc::c_int);
    }
    return SDL_JoystickInstanceID(joystick);
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_GetName(mut this: *mut Gamepad) -> cstr {
    return SDL_GameControllerName((*this).handle);
}
#[no_mangle]
pub unsafe extern "C" fn Gamepad_IsConnected(mut this: *mut Gamepad) -> bool {
    return SDL_GameControllerGetAttached((*this).handle) as libc::c_uint
        == SDL_TRUE as libc::c_int as libc::c_uint;
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
