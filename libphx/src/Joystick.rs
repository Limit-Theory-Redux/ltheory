use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    pub type _SDL_Joystick;
    fn Fatal(_: cstr, _: ...);
    fn fabs(_: f64) -> f64;
    fn SDL_JoystickGetGUID(joystick: *mut SDL_Joystick) -> SDL_JoystickGUID;
    fn SDL_JoystickClose(joystick: *mut SDL_Joystick);
    fn SDL_NumJoysticks() -> libc::c_int;
    fn SDL_JoystickGetHat(joystick: *mut SDL_Joystick, hat: libc::c_int) -> Uint8;
    fn SDL_JoystickGetButton(joystick: *mut SDL_Joystick, button: libc::c_int) -> Uint8;
    fn SDL_JoystickGetAxis(joystick: *mut SDL_Joystick, axis: libc::c_int) -> Sint16;
    fn SDL_JoystickNumHats(joystick: *mut SDL_Joystick) -> libc::c_int;
    fn SDL_JoystickNumButtons(joystick: *mut SDL_Joystick) -> libc::c_int;
    fn SDL_JoystickNumBalls(joystick: *mut SDL_Joystick) -> libc::c_int;
    fn SDL_JoystickNumAxes(joystick: *mut SDL_Joystick) -> libc::c_int;
    fn SDL_JoystickNameForIndex(device_index: libc::c_int) -> *const libc::c_char;
    fn SDL_JoystickName(joystick: *mut SDL_Joystick) -> *const libc::c_char;
    fn SDL_JoystickGetDeviceGUID(device_index: libc::c_int) -> SDL_JoystickGUID;
    fn SDL_JoystickOpen(device_index: libc::c_int) -> *mut SDL_Joystick;
    fn SDL_JoystickGetGUIDString(
        guid: SDL_JoystickGUID,
        pszGUID: *mut libc::c_char,
        cbGUID: libc::c_int,
    );
    fn TimeStamp_Get() -> TimeStamp;
    fn TimeStamp_GetElapsed(start: TimeStamp) -> f64;
}
pub type int16_t = libc::c_short;
pub type int32_t = libc::c_int;
pub type uint8_t = libc::c_uchar;
pub type uint64_t = libc::c_ulonglong;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint64 = uint64_t;
pub type TimeStamp = uint64;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Joystick {
    pub handle: *mut SDL_Joystick,
    pub guid: cstr,
    pub axes: libc::c_int,
    pub balls: libc::c_int,
    pub buttons: libc::c_int,
    pub hats: libc::c_int,
    pub buttonStates: *mut bool,
    pub axisAlive: *mut bool,
    pub axisStates: *mut f64,
    pub lastUsed: TimeStamp,
}
pub type SDL_Joystick = _SDL_Joystick;
pub type HatDir = int32;
pub type Uint8 = uint8_t;
pub type Sint16 = int16_t;
pub type SDL_GUID = SDL_JoystickGUID;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_JoystickGUID {
    pub data: [Uint8; 16],
}
#[inline]
unsafe extern "C" fn Abs(mut t: f64) -> f64 {
    return fabs(t);
}


static mut kMaxOpen: libc::c_int = 64 as libc::c_int;
static mut kOpen: libc::c_int = 0 as libc::c_int;
static mut freeList: [*mut Joystick; 64] = [
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
    0 as *const Joystick as *mut Joystick,
];
unsafe extern "C" fn ConvertGUID(mut id: SDL_JoystickGUID) -> *mut libc::c_char {
    static mut buf: [libc::c_char; 64] = [0; 64];
    SDL_JoystickGetGUIDString(
        id,
        buf.as_mut_ptr(),
        ::core::mem::size_of::<[libc::c_char; 64]>() as libc::c_ulong as libc::c_int,
    );
    return buf.as_mut_ptr();
}
unsafe extern "C" fn Joystick_UpdateSingle(mut this: *mut Joystick) {
    let mut changed: bool = 0 as libc::c_int != 0;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*this).axes {
        let mut state: f64 = Joystick_GetAxis(this, i);
        let mut delta: f64 = Abs(
            state - *((*this).axisStates).offset(i as isize),
        );
        if delta > 0.1f64 {
            changed = 1 as libc::c_int != 0;
            *((*this).axisAlive).offset(i as isize) = 1 as libc::c_int != 0;
        }
        *((*this).axisStates).offset(i as isize) = state;
        i += 1;
    }
    let mut i_0: libc::c_int = 0 as libc::c_int;
    while i_0 < (*this).buttons {
        let mut state_0: bool = Joystick_ButtonDown(this, i_0);
        if *((*this).buttonStates).offset(i_0 as isize) as libc::c_int
            != state_0 as libc::c_int
        {
            changed = 1 as libc::c_int != 0;
        }
        *((*this).buttonStates).offset(i_0 as isize) = state_0;
        i_0 += 1;
    }
    if changed {
        (*this).lastUsed = TimeStamp_Get();
    }
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetCount() -> libc::c_int {
    return SDL_NumJoysticks();
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_Open(mut index: libc::c_int) -> *mut Joystick {
    let mut this: *mut Joystick = MemAlloc(
        ::core::mem::size_of::<Joystick>() as usize,
    ) as *mut Joystick;
    if kOpen == kMaxOpen {
        Fatal(
            b"Cannot open any more gamepad connections.\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < kMaxOpen {
        if (freeList[i as usize]).is_null() {
            freeList[i as usize] = this;
            kOpen += 1;
            break;
        } else {
            i += 1;
        }
    }
    (*this).handle = SDL_JoystickOpen(index);
    (*this).guid = StrDup(ConvertGUID(SDL_JoystickGetGUID((*this).handle)) as cstr);
    (*this).axes = SDL_JoystickNumAxes((*this).handle);
    (*this).balls = SDL_JoystickNumBalls((*this).handle);
    (*this).buttons = SDL_JoystickNumButtons((*this).handle);
    (*this).hats = SDL_JoystickNumHats((*this).handle);
    (*this)
        .buttonStates = MemAlloc(
        (::core::mem::size_of::<bool>())
            .wrapping_mul((*this).buttons as usize),
    ) as *mut bool;
    (*this)
        .axisAlive = MemAlloc(
        (::core::mem::size_of::<bool>())
            .wrapping_mul((*this).axes as usize),
    ) as *mut bool;
    MemZero(
        (*this).axisAlive as *mut libc::c_void,
        (::core::mem::size_of::<bool>())
            .wrapping_mul((*this).axes as usize),
    );
    (*this)
        .axisStates = MemAlloc(
        (::core::mem::size_of::<f64>())
            .wrapping_mul((*this).axes as usize),
    ) as *mut f64;
    (*this).lastUsed = TimeStamp_Get();
    Joystick_UpdateSingle(this);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_Close(mut this: *mut Joystick) {
    kOpen -= 1;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < kMaxOpen {
        if freeList[i as usize] == this {
            freeList[i as usize] = 0 as *mut Joystick;
            break;
        } else {
            i += 1;
        }
    }
    SDL_JoystickClose((*this).handle);
    MemFree((*this).guid as *const libc::c_void);
    MemFree((*this).buttonStates as *const libc::c_void);
    MemFree((*this).axisStates as *const libc::c_void);
    MemFree(this as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetGUID(mut this: *mut Joystick) -> cstr {
    return (*this).guid;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetGUIDByIndex(mut index: libc::c_int) -> cstr {
    return ConvertGUID(SDL_JoystickGetDeviceGUID(index)) as cstr;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetName(mut this: *mut Joystick) -> cstr {
    return SDL_JoystickName((*this).handle);
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetNameByIndex(mut index: libc::c_int) -> cstr {
    return SDL_JoystickNameForIndex(index);
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetAxisCount(
    mut this: *mut Joystick,
) -> libc::c_int {
    return (*this).axes;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetBallCount(
    mut this: *mut Joystick,
) -> libc::c_int {
    return (*this).balls;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetButtonCount(
    mut this: *mut Joystick,
) -> libc::c_int {
    return (*this).buttons;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetHatCount(mut this: *mut Joystick) -> libc::c_int {
    return (*this).hats;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetIdleTime(
    mut this: *mut Joystick,
) -> f64 {
    return TimeStamp_GetElapsed((*this).lastUsed);
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetAxis(
    mut this: *mut Joystick,
    mut index: libc::c_int,
) -> f64 {
    return SDL_JoystickGetAxis((*this).handle, index) as libc::c_int as f64
        / 32768.0f64;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetAxisAlive(
    mut this: *mut Joystick,
    mut index: libc::c_int,
) -> bool {
    return *((*this).axisAlive).offset(index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetAxisDelta(
    mut this: *mut Joystick,
    mut index: libc::c_int,
) -> f64 {
    return SDL_JoystickGetAxis((*this).handle, index) as libc::c_int as f64
        / 32768.0f64 - *((*this).axisStates).offset(index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetHat(
    mut this: *mut Joystick,
    mut index: libc::c_int,
) -> HatDir {
    return SDL_JoystickGetHat((*this).handle, index) as HatDir;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_ButtonDown(
    mut this: *mut Joystick,
    mut index: libc::c_int,
) -> bool {
    return SDL_JoystickGetButton((*this).handle, index) as libc::c_int
        > 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_ButtonPressed(
    mut this: *mut Joystick,
    mut index: libc::c_int,
) -> bool {
    return SDL_JoystickGetButton((*this).handle, index) as libc::c_int
        > 0 as libc::c_int && !*((*this).buttonStates).offset(index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_ButtonReleased(
    mut this: *mut Joystick,
    mut index: libc::c_int,
) -> bool {
    return SDL_JoystickGetButton((*this).handle, index) as libc::c_int
        == 0 as libc::c_int
        && *((*this).buttonStates).offset(index as isize) as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_Update() {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < kMaxOpen {
        if !(freeList[i as usize]).is_null() {
            Joystick_UpdateSingle(freeList[i as usize]);
        }
        i += 1;
    }
}
