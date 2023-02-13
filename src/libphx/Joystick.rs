use ::libc;
use super::internal::Memory::*;
extern "C" {
    pub type _SDL_Joystick;
    fn memcpy(
        _: *mut libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn Fatal(_: cstr, _: ...);
    fn memset(
        _: *mut libc::c_void,
        _: libc::c_int,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn fabs(_: libc::c_double) -> libc::c_double;
    fn free(_: *mut libc::c_void);
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
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
    fn TimeStamp_GetElapsed(start: TimeStamp) -> libc::c_double;
}
pub type int16_t = libc::c_short;
pub type int32_t = libc::c_int;
pub type uint8_t = libc::c_uchar;
pub type uint64_t = libc::c_ulonglong;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
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
    pub axisStates: *mut libc::c_double,
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
unsafe extern "C" fn Abs(mut t: libc::c_double) -> libc::c_double {
    return fabs(t);
}

#[inline]
unsafe extern "C" fn StrAlloc(mut len: size_t) -> *mut libc::c_char {
    return malloc(len) as *mut libc::c_char;
}
#[inline]
unsafe extern "C" fn StrLen(mut s: cstr) -> size_t {
    if s.is_null() {
        return 0 as libc::c_int as size_t;
    }
    let mut begin: cstr = s;
    while *s != 0 {
        s = s.offset(1);
    }
    return s.offset_from(begin) as libc::c_long as size_t;
}
#[inline]
unsafe extern "C" fn StrDup(mut s: cstr) -> cstr {
    if s.is_null() {
        return 0 as cstr;
    }
    let mut len: size_t = (StrLen(s)).wrapping_add(1 as libc::c_int as libc::c_ulong);
    let mut buf: *mut libc::c_char = StrAlloc(len);
    memcpy(buf as *mut libc::c_void, s as *const libc::c_void, len);
    return buf as cstr;
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
unsafe extern "C" fn Joystick_UpdateSingle(mut self_0: *mut Joystick) {
    let mut changed: bool = 0 as libc::c_int != 0;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*self_0).axes {
        let mut state: libc::c_double = Joystick_GetAxis(self_0, i);
        let mut delta: libc::c_double = Abs(
            state - *((*self_0).axisStates).offset(i as isize),
        );
        if delta > 0.1f64 {
            changed = 1 as libc::c_int != 0;
            *((*self_0).axisAlive).offset(i as isize) = 1 as libc::c_int != 0;
        }
        *((*self_0).axisStates).offset(i as isize) = state;
        i += 1;
    }
    let mut i_0: libc::c_int = 0 as libc::c_int;
    while i_0 < (*self_0).buttons {
        let mut state_0: bool = Joystick_ButtonDown(self_0, i_0);
        if *((*self_0).buttonStates).offset(i_0 as isize) as libc::c_int
            != state_0 as libc::c_int
        {
            changed = 1 as libc::c_int != 0;
        }
        *((*self_0).buttonStates).offset(i_0 as isize) = state_0;
        i_0 += 1;
    }
    if changed {
        (*self_0).lastUsed = TimeStamp_Get();
    }
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetCount() -> libc::c_int {
    return SDL_NumJoysticks();
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_Open(mut index: libc::c_int) -> *mut Joystick {
    let mut self_0: *mut Joystick = MemAlloc(
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
            freeList[i as usize] = self_0;
            kOpen += 1;
            break;
        } else {
            i += 1;
        }
    }
    (*self_0).handle = SDL_JoystickOpen(index);
    (*self_0).guid = StrDup(ConvertGUID(SDL_JoystickGetGUID((*self_0).handle)) as cstr);
    (*self_0).axes = SDL_JoystickNumAxes((*self_0).handle);
    (*self_0).balls = SDL_JoystickNumBalls((*self_0).handle);
    (*self_0).buttons = SDL_JoystickNumButtons((*self_0).handle);
    (*self_0).hats = SDL_JoystickNumHats((*self_0).handle);
    (*self_0)
        .buttonStates = MemAlloc(
        (::core::mem::size_of::<bool>())
            .wrapping_mul((*self_0).buttons as libc::c_ulong),
    ) as *mut bool;
    (*self_0)
        .axisAlive = MemAlloc(
        (::core::mem::size_of::<bool>())
            .wrapping_mul((*self_0).axes as libc::c_ulong),
    ) as *mut bool;
    MemZero(
        (*self_0).axisAlive as *mut libc::c_void,
        (::core::mem::size_of::<bool>())
            .wrapping_mul((*self_0).axes as libc::c_ulong),
    );
    (*self_0)
        .axisStates = MemAlloc(
        (::core::mem::size_of::<libc::c_double>())
            .wrapping_mul((*self_0).axes as libc::c_ulong),
    ) as *mut libc::c_double;
    (*self_0).lastUsed = TimeStamp_Get();
    Joystick_UpdateSingle(self_0);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_Close(mut self_0: *mut Joystick) {
    kOpen -= 1;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < kMaxOpen {
        if freeList[i as usize] == self_0 {
            freeList[i as usize] = 0 as *mut Joystick;
            break;
        } else {
            i += 1;
        }
    }
    SDL_JoystickClose((*self_0).handle);
    MemFree((*self_0).guid as *const libc::c_void);
    MemFree((*self_0).buttonStates as *const libc::c_void);
    MemFree((*self_0).axisStates as *const libc::c_void);
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetGUID(mut self_0: *mut Joystick) -> cstr {
    return (*self_0).guid;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetGUIDByIndex(mut index: libc::c_int) -> cstr {
    return ConvertGUID(SDL_JoystickGetDeviceGUID(index)) as cstr;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetName(mut self_0: *mut Joystick) -> cstr {
    return SDL_JoystickName((*self_0).handle);
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetNameByIndex(mut index: libc::c_int) -> cstr {
    return SDL_JoystickNameForIndex(index);
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetAxisCount(
    mut self_0: *mut Joystick,
) -> libc::c_int {
    return (*self_0).axes;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetBallCount(
    mut self_0: *mut Joystick,
) -> libc::c_int {
    return (*self_0).balls;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetButtonCount(
    mut self_0: *mut Joystick,
) -> libc::c_int {
    return (*self_0).buttons;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetHatCount(mut self_0: *mut Joystick) -> libc::c_int {
    return (*self_0).hats;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetIdleTime(
    mut self_0: *mut Joystick,
) -> libc::c_double {
    return TimeStamp_GetElapsed((*self_0).lastUsed);
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetAxis(
    mut self_0: *mut Joystick,
    mut index: libc::c_int,
) -> libc::c_double {
    return SDL_JoystickGetAxis((*self_0).handle, index) as libc::c_int as libc::c_double
        / 32768.0f64;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetAxisAlive(
    mut self_0: *mut Joystick,
    mut index: libc::c_int,
) -> bool {
    return *((*self_0).axisAlive).offset(index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetAxisDelta(
    mut self_0: *mut Joystick,
    mut index: libc::c_int,
) -> libc::c_double {
    return SDL_JoystickGetAxis((*self_0).handle, index) as libc::c_int as libc::c_double
        / 32768.0f64 - *((*self_0).axisStates).offset(index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_GetHat(
    mut self_0: *mut Joystick,
    mut index: libc::c_int,
) -> HatDir {
    return SDL_JoystickGetHat((*self_0).handle, index) as HatDir;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_ButtonDown(
    mut self_0: *mut Joystick,
    mut index: libc::c_int,
) -> bool {
    return SDL_JoystickGetButton((*self_0).handle, index) as libc::c_int
        > 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_ButtonPressed(
    mut self_0: *mut Joystick,
    mut index: libc::c_int,
) -> bool {
    return SDL_JoystickGetButton((*self_0).handle, index) as libc::c_int
        > 0 as libc::c_int && !*((*self_0).buttonStates).offset(index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Joystick_ButtonReleased(
    mut self_0: *mut Joystick,
    mut index: libc::c_int,
) -> bool {
    return SDL_JoystickGetButton((*self_0).handle, index) as libc::c_int
        == 0 as libc::c_int
        && *((*self_0).buttonStates).offset(index as isize) as libc::c_int != 0;
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
