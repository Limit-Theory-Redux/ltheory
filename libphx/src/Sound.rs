use crate::Audio::*;
use crate::Common::*;
use crate::Math::Vec3;
use crate::SoundDesc::*;
use fmod_sys::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Sound {
    pub desc: *mut SoundDesc,
    pub handle: *mut FMOD_CHANNEL,
    pub state: SoundState,
    pub autoPos: *const Vec3,
    pub autoVel: *const Vec3,
    pub freeOnFinish: bool,
}
pub type SoundState = u8;

#[inline]
pub fn FMODCALL(result: FMOD_RESULT) {
    if result != FMOD_RESULT::FMOD_OK {
        panic!(
            "FMOD operation failed with {:?} ({})",
            result,
            result.error_string()
        );
    }
}

unsafe extern "C" fn Sound_Callback(
    channel: *mut FMOD_CHANNELCONTROL,
    _controlType: FMOD_CHANNELCONTROL_TYPE,
    callbackType: FMOD_CHANNELCONTROL_CALLBACK_TYPE,
    _a: *mut libc::c_void,
    _b: *mut libc::c_void,
) -> FMOD_RESULT {
    if callbackType == FMOD_CHANNELCONTROL_CALLBACK_TYPE::FMOD_CHANNELCONTROL_CALLBACK_END {
        let mut this: *mut Sound = std::ptr::null_mut();
        FMODCALL(FMOD_Channel_GetUserData(
            channel as *mut FMOD_CHANNEL,
            &mut this as *mut *mut Sound as _,
        ));
        Sound_SetState(&mut *this, 4 as SoundState);
    }
    FMOD_RESULT::FMOD_OK
}

#[inline]
unsafe extern "C" fn Sound_EnsureLoadedImpl(this: &mut Sound, func: *const libc::c_char) {
    if this.state as i32 == 1 {
        SoundDesc_FinishLoad(&mut *this.desc, func);
        FMODCALL(FMOD_System_PlaySound(
            Audio_GetHandle() as *mut FMOD_SYSTEM,
            (*this.desc).handle,
            std::ptr::null_mut(),
            1,
            &mut this.handle,
        ));
        FMODCALL(FMOD_Channel_SetUserData(
            this.handle,
            this as *mut Sound as *mut _,
        ));
        FMODCALL(FMOD_Channel_SetCallback(
            this.handle,
            Some(
                Sound_Callback
                    as unsafe extern "C" fn(
                        *mut FMOD_CHANNELCONTROL,
                        FMOD_CHANNELCONTROL_TYPE,
                        FMOD_CHANNELCONTROL_CALLBACK_TYPE,
                        *mut libc::c_void,
                        *mut libc::c_void,
                    ) -> FMOD_RESULT,
            ),
        ));
        Sound_SetState(this, 2 as SoundState);
        if Sound_Get3D(this) {
            let mut zero: Vec3 = Vec3::ZERO;
            Sound_Set3DPos(this, &mut zero, &mut zero);
        }
    }
}

#[inline]
unsafe extern "C" fn Sound_EnsureNotFreedImpl(this: &mut Sound, func: *const libc::c_char) {
    if this.state as i32 == 5 {
        let name: *const libc::c_char = if (*this.desc)._refCount > 0 {
            (*this.desc).name
        } else {
            c_str!("<SoundDesc has been freed>")
        };
        CFatal!("%s: Sound has been freed.\n  Name: %s", func, name);
    }
}

#[inline]
unsafe extern "C" fn Sound_EnsureStateImpl(this: &mut Sound, func: *const libc::c_char) {
    Sound_EnsureLoadedImpl(this, func);
    Sound_EnsureNotFreedImpl(this, func);
}

unsafe extern "C" fn Sound_SetState(this: &mut Sound, nextState: SoundState) {
    if nextState as i32 == this.state as i32 {}
    match nextState as i32 {
        3 => {
            FMODCALL(FMOD_Channel_SetPaused(this.handle, 0));
        }
        2 => {
            FMODCALL(FMOD_Channel_SetPaused(this.handle, 1));
        }
        4 => {
            FMODCALL(FMOD_Channel_Stop(this.handle));
        }
        1 | 5 => {}
        _ => {
            CFatal!("Sound_SetState: Unhandled case: %i", nextState as i32,);
        }
    }
    this.state = nextState;
    Audio_SoundStateChanged(this);
    if this.freeOnFinish as i32 != 0 && this.state as i32 == 4 as i32 {
        Sound_Free(this);
    }
}

unsafe extern "C" fn Sound_Create(
    name: *const libc::c_char,
    immediate: bool,
    isLooped: bool,
    is3D: bool,
) -> *mut Sound {
    let desc: *mut SoundDesc = SoundDesc_Load(name, immediate, isLooped, is3D);
    let this: *mut Sound = Audio_AllocSound();
    (*this).desc = desc;
    Sound_SetState(&mut *this, 1 as SoundState);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Load(
    name: *const libc::c_char,
    isLooped: bool,
    is3D: bool,
) -> *mut Sound {
    let this: *mut Sound = Sound_Create(name, true, isLooped, is3D);
    Sound_EnsureLoadedImpl(&mut *this, c_str!("Sound_Load"));
    this
}

#[no_mangle]
pub unsafe extern "C" fn Sound_LoadAsync(
    name: *const libc::c_char,
    isLooped: bool,
    is3D: bool,
) -> *mut Sound {
    Sound_Create(name, false, isLooped, is3D)
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Clone(this: &mut Sound) -> *mut Sound {
    Sound_EnsureStateImpl(this, c_str!("Sound_Clone"));
    let clone: *mut Sound = Audio_AllocSound();
    *clone = *this;
    SoundDesc_Acquire(&mut *this.desc);
    (*clone).handle = std::ptr::null_mut();
    (*clone).state = 0 as SoundState;
    Sound_SetState(&mut *clone, 1 as SoundState);
    clone
}

#[no_mangle]
pub unsafe extern "C" fn Sound_ToFile(this: &mut Sound, name: *const libc::c_char) {
    Sound_EnsureStateImpl(this, c_str!("Sound_ToFile"));
    SoundDesc_ToFile(&mut *this.desc, name);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Acquire(this: &mut Sound) {
    Sound_EnsureStateImpl(this, c_str!("Sound_Acquire"));
    (*this.desc)._refCount = ((*this.desc)._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Free(this: &mut Sound) {
    Sound_EnsureStateImpl(this, c_str!("Sound_Free"));
    Sound_SetState(this, 4 as SoundState);
    Sound_SetState(this, 5 as SoundState);
    SoundDesc_Free(&mut *this.desc);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Play(this: &mut Sound) {
    Sound_EnsureStateImpl(this, c_str!("Sound_Play"));
    Sound_SetState(this, 3 as SoundState);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Pause(this: &mut Sound) {
    Sound_EnsureStateImpl(this, c_str!("Sound_Pause"));
    Sound_SetState(this, 2 as SoundState);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Rewind(this: &mut Sound) {
    Sound_EnsureStateImpl(this, c_str!("Sound_Rewind"));
    FMODCALL(FMOD_Channel_SetPosition(
        this.handle,
        0 as u32,
        0x2 as i32 as FMOD_TIMEUNIT,
    ));
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Get3D(this: &mut Sound) -> bool {
    Sound_EnsureStateImpl(this, c_str!("Sound_Get3D"));
    let mut mode: FMOD_MODE = 0;
    FMODCALL(FMOD_Channel_GetMode(this.handle, &mut mode));
    mode & 0x10 == 0x10
}

#[no_mangle]
pub unsafe extern "C" fn Sound_GetDuration(this: &mut Sound) -> f32 {
    Sound_EnsureStateImpl(this, c_str!("Sound_GetDuration"));
    SoundDesc_GetDuration(&mut *this.desc)
}

#[no_mangle]
pub unsafe extern "C" fn Sound_GetLooped(this: &mut Sound) -> bool {
    Sound_EnsureStateImpl(this, c_str!("Sound_GetLooped"));
    let mut mode: FMOD_MODE = 0;
    FMODCALL(FMOD_Channel_GetMode(this.handle, &mut mode));
    mode & 0x2 == 0x2
}

#[no_mangle]
pub unsafe extern "C" fn Sound_GetName(this: &mut Sound) -> *const libc::c_char {
    Sound_EnsureNotFreedImpl(this, c_str!("Sound_GetName"));
    SoundDesc_GetName(&mut *this.desc)
}

#[no_mangle]
pub unsafe extern "C" fn Sound_GetPath(this: &mut Sound) -> *const libc::c_char {
    Sound_EnsureNotFreedImpl(this, c_str!("Sound_GetPath"));
    SoundDesc_GetPath(&mut *this.desc)
}

#[no_mangle]
pub extern "C" fn Sound_IsFinished(this: &mut Sound) -> bool {
    this.state as i32 == 4
}

#[no_mangle]
pub extern "C" fn Sound_IsPlaying(this: &mut Sound) -> bool {
    this.state as i32 == 3
}

#[no_mangle]
pub unsafe extern "C" fn Sound_IsAudible(this: &mut Sound) -> bool {
    Sound_EnsureStateImpl(this, c_str!("Sound_Set3DLevel"));

    let mut audibility = 0.0f32;
    FMOD_Channel_GetAudibility(this.handle, &mut audibility);
    return audibility > 0.0f32;
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Attach3DPos(this: &mut Sound, pos: *const Vec3, vel: *const Vec3) {
    Sound_Set3DPos(this, pos, vel);
    this.autoPos = pos;
    this.autoVel = vel;
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Set3DLevel(this: &mut Sound, level: f32) {
    Sound_EnsureStateImpl(this, c_str!("Sound_Set3DLevel"));
    FMODCALL(FMOD_Channel_Set3DLevel(this.handle, level));
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Set3DPos(this: &mut Sound, pos: *const Vec3, vel: *const Vec3) {
    Sound_EnsureStateImpl(this, c_str!("Sound_Set3DPos"));
    FMODCALL(FMOD_Channel_Set3DAttributes(
        this.handle,
        pos as *mut FMOD_VECTOR,
        vel as *mut FMOD_VECTOR,
    ));
}

#[no_mangle]
pub extern "C" fn Sound_SetFreeOnFinish(this: &mut Sound, freeOnFinish: bool) {
    this.freeOnFinish = freeOnFinish;
}

#[no_mangle]
pub unsafe extern "C" fn Sound_SetPan(this: &mut Sound, pan: f32) {
    Sound_EnsureStateImpl(this, c_str!("Sound_SetPan"));
    FMODCALL(FMOD_Channel_SetPan(this.handle, pan));
}

#[no_mangle]
pub unsafe extern "C" fn Sound_SetPitch(this: &mut Sound, pitch: f32) {
    Sound_EnsureStateImpl(this, c_str!("Sound_SetPitch"));
    FMODCALL(FMOD_Channel_SetPitch(this.handle, pitch));
}

#[no_mangle]
pub unsafe extern "C" fn Sound_SetPlayPos(this: &mut Sound, seconds: f32) {
    Sound_EnsureStateImpl(this, c_str!("Sound_SetPlayPos"));
    let ms: u32 = f64::round((seconds * 1000.0f32) as f64) as u32;
    FMODCALL(FMOD_Channel_SetPosition(
        this.handle,
        ms,
        0x1 as FMOD_TIMEUNIT,
    ));
}

#[no_mangle]
pub unsafe extern "C" fn Sound_SetVolume(this: &mut Sound, volume: f32) {
    Sound_EnsureStateImpl(this, c_str!("Sound_SetVolume"));
    FMODCALL(FMOD_Channel_SetVolume(this.handle, volume));
}

#[no_mangle]
pub unsafe extern "C" fn Sound_FadeIn(this: &mut Sound, seconds: f32) {
    Sound_EnsureStateImpl(this, c_str!("Sound_FadeIn"));
    // Assert(seconds >= 0.0f);

    if !Sound_IsPlaying(this) {
        Sound_Play(this);
    }

    // Already fading in/out?
    let mut numpoints = 0;
    FMOD_Channel_GetFadePoints(
        this.handle,
        &mut numpoints,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if numpoints > 0 {
        return;
    }

    let mut rate = 0;
    FMOD_System_GetSoftwareFormat(
        Audio_GetHandle() as *mut _,
        &mut rate,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    let fadeTime = (rate as f32 * seconds) as u64;

    let mut volume = 1.0f32;
    FMOD_Channel_GetVolume(this.handle, &mut volume);

    let mut dspClock = 0;
    FMOD_Channel_GetDSPClock(this.handle, std::ptr::null_mut(), &mut dspClock);

    FMOD_Channel_SetDelay(this.handle, dspClock, 0, 0);

    FMOD_Channel_AddFadePoint(this.handle, dspClock, 0.0f32);
    FMOD_Channel_AddFadePoint(this.handle, dspClock + fadeTime, volume);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_FadeOut(this: &mut Sound, seconds: f32) {
    Sound_EnsureStateImpl(this, c_str!("Sound_FadeOut"));
    // Assert(seconds >= 0.0f);

    // Already fading in/out?
    let mut numpoints = 0;
    FMOD_Channel_GetFadePoints(
        this.handle,
        &mut numpoints,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if numpoints > 0 {
        return;
    }

    let mut rate = 0;
    FMOD_System_GetSoftwareFormat(
        Audio_GetHandle() as *mut _,
        &mut rate,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    let fadeTime = (rate as f32 * seconds) as u64;

    let mut volume = 1.0f32;
    FMOD_Channel_GetVolume(this.handle, &mut volume);

    let mut dspClock = 0;
    FMOD_Channel_GetDSPClock(this.handle, std::ptr::null_mut(), &mut dspClock);

    FMOD_Channel_AddFadePoint(this.handle, dspClock, volume);
    FMOD_Channel_AddFadePoint(this.handle, dspClock + fadeTime, 0.0f32);

    FMOD_Channel_SetDelay(this.handle, dspClock + fadeTime, 0, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_LoadPlay(
    name: *const libc::c_char,
    isLooped: bool,
    is3D: bool,
) -> *mut Sound {
    let this: *mut Sound = Sound_Load(name, isLooped, is3D);
    Sound_Play(&mut *this);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Sound_LoadPlayAttached(
    name: *const libc::c_char,
    isLooped: bool,
    is3D: bool,
    pos: *const Vec3,
    vel: *const Vec3,
) -> *mut Sound {
    let this: *mut Sound = Sound_Load(name, isLooped, is3D);
    Sound_Attach3DPos(&mut *this, pos, vel);
    Sound_Play(&mut *this);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Sound_LoadPlayFree(name: *const libc::c_char, isLooped: bool, is3D: bool) {
    let this: *mut Sound = Sound_Load(name, isLooped, is3D);
    Sound_SetFreeOnFinish(&mut *this, true);
    Sound_Play(&mut *this);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_LoadPlayFreeAttached(
    name: *const libc::c_char,
    isLooped: bool,
    is3D: bool,
    pos: *const Vec3,
    vel: *const Vec3,
) {
    let this: *mut Sound = Sound_Load(name, isLooped, is3D);
    Sound_Attach3DPos(&mut *this, pos, vel);
    Sound_SetFreeOnFinish(&mut *this, true);
    Sound_Play(&mut *this);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_ClonePlay(this: &mut Sound) -> *mut Sound {
    let clone: *mut Sound = Sound_Clone(this);
    Sound_Play(&mut *clone);
    clone
}

#[no_mangle]
pub unsafe extern "C" fn Sound_ClonePlayAttached(
    this: &mut Sound,
    pos: *const Vec3,
    vel: *const Vec3,
) -> *mut Sound {
    let clone: *mut Sound = Sound_Clone(this);
    Sound_Attach3DPos(&mut *clone, pos, vel);
    Sound_Play(&mut *clone);
    clone
}

#[no_mangle]
pub unsafe extern "C" fn Sound_ClonePlayFree(this: &mut Sound) {
    let clone: *mut Sound = Sound_Clone(this);
    Sound_SetFreeOnFinish(&mut *clone, true);
    Sound_Play(&mut *clone);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_ClonePlayFreeAttached(
    this: &mut Sound,
    pos: *const Vec3,
    vel: *const Vec3,
) {
    let clone: *mut Sound = Sound_Clone(this);
    Sound_Attach3DPos(&mut *clone, pos, vel);
    Sound_SetFreeOnFinish(&mut *clone, true);
    Sound_Play(&mut *clone);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Update(this: &mut Sound) {
    if this.state as i32 == 1 {
        return;
    }
    if Sound_Get3D(this) {
        Sound_Set3DPos(this, this.autoPos, this.autoVel);
    }
}

#[no_mangle]
pub extern "C" fn Sound_IsFreed(this: &mut Sound) -> bool {
    this.state as i32 == 5
}
