use crate::common::*;
use crate::internal::*;
use crate::math::Vec3;
use crate::mem_pool::*;
use crate::sound::*;
use crate::sound_desc::*;
use crate::str_map::*;
use crate::MemNewZero;
use crate::*;
use fmod_sys::*;

const AUDIO_CHANNELS: i32 = 1024;
const SOUNDPOOL_BLOCK_SIZE: u32 = 128;

#[derive(Clone)]
#[repr(C)]
pub struct Audio {
    pub handle: *mut FMOD_SYSTEM,
    pub descMap: *mut StrMap,
    pub soundPool: *mut MemPool,
    pub playingSounds: Vec<*mut Sound>,
    pub freeingSounds: Vec<*mut Sound>,
    pub autoPos: *const Vec3,
    pub autoVel: *const Vec3,
    pub autoFwd: *const Vec3,
    pub autoUp: *const Vec3,
}

static mut this: Audio = Audio {
    handle: std::ptr::null_mut(),
    descMap: std::ptr::null_mut(),
    soundPool: std::ptr::null_mut(),
    playingSounds: Vec::new(),
    freeingSounds: Vec::new(),
    autoPos: std::ptr::null(),
    autoVel: std::ptr::null(),
    autoFwd: std::ptr::null(),
    autoUp: std::ptr::null(),
};

#[no_mangle]
pub unsafe extern "C" fn Audio_Init() {
    /* Initialize Debugging. */
    let mut flags: FMOD_DEBUG_FLAGS = 0;
    flags |= FMOD_DEBUG_LEVEL_NONE;
    // CHECK1(flags |= FMOD_DEBUG_LEVEL_ERROR);
    // CHECK1(flags |= FMOD_DEBUG_LEVEL_WARNING);
    // CHECK2(flags |= FMOD_DEBUG_LEVEL_LOG);

    let res: FMOD_RESULT = FMOD_Debug_Initialize(
        flags,
        FMOD_DEBUG_MODE::FMOD_DEBUG_MODE_FILE,
        None,
        c_str!("log/fmod.txt"),
    );
    if res != FMOD_RESULT::FMOD_OK && res != FMOD_RESULT::FMOD_ERR_UNSUPPORTED {
        FMODCALL(res);
    }

    /* Initialize FMOD. */
    FMODCALL(FMOD_System_Create(&mut this.handle, FMOD_VERSION));

    let mut version: u32 = 0;
    FMODCALL(FMOD_System_GetVersion(this.handle, &mut version));
    if version < FMOD_VERSION {
        CFatal!("Audio_Create: FMOD library link/compile version mismatch");
    }

    /* NOTE : The fake HRTF mentioned in FMOD_INIT_CHANNEL_LOWPASS and
     *        FMOD_ADVANCEDSETTINGS has been removed from FMOD.
     *        http://www.fmod.org/questions/question/hrtf-does-not-appear-to-work/ */
    let mut flags: FMOD_INITFLAGS = 0;
    flags |= FMOD_INIT_NORMAL;
    flags |= FMOD_INIT_3D_RIGHTHANDED;
    flags |= FMOD_INIT_CHANNEL_DISTANCEFILTER;
    // CHECK2(flags |= FMOD_INIT_PROFILE_ENABLE);
    FMODCALL(FMOD_System_Init(
        this.handle,
        AUDIO_CHANNELS,
        flags,
        std::ptr::null_mut(),
    ));

    /* Initialize audio instance data. */
    this.descMap = StrMap_Create(SOUNDPOOL_BLOCK_SIZE);
    this.soundPool = MemPool_Create(std::mem::size_of::<Sound>() as u32, SOUNDPOOL_BLOCK_SIZE);
}

#[no_mangle]
pub unsafe extern "C" fn Audio_Free() {
    FMODCALL(FMOD_System_Release(this.handle));
    StrMap_Free(&mut *this.descMap);
    MemPool_Free(&mut *this.soundPool);
    this.playingSounds.clear();
    this.freeingSounds.clear();
}

#[no_mangle]
pub unsafe extern "C" fn Audio_AttachListenerPos(
    pos: *const Vec3,
    vel: *const Vec3,
    fwd: *const Vec3,
    up: *const Vec3,
) {
    this.autoPos = pos;
    this.autoVel = vel;
    this.autoFwd = fwd;
    this.autoUp = up;
    Audio_SetListenerPos(pos, vel, fwd, up);
}

#[no_mangle]
pub unsafe extern "C" fn Audio_Set3DSettings(doppler: f32, scale: f32, rolloff: f32) {
    FMODCALL(FMOD_System_Set3DSettings(
        this.handle,
        doppler,
        scale,
        rolloff,
    ));
}

#[no_mangle]
pub unsafe extern "C" fn Audio_SetListenerPos(
    pos: *const Vec3,
    vel: *const Vec3,
    fwd: *const Vec3,
    up: *const Vec3,
) {
    //   Assert(sizeof(*pos) == sizeof(FMOD_VECTOR));
    //   Assert(!fwd || Approx(Vec3f_Length(*fwd), 1));
    //   Assert(!up || Approx(Vec3f_Length(*up), 1));
    //   Assert(!fwd || !up || Approx(Vec3f_Dot(*fwd, *up), 0));

    FMODCALL(FMOD_System_Set3DListenerAttributes(
        this.handle,
        0,
        pos as *const FMOD_VECTOR,
        vel as *const FMOD_VECTOR,
        fwd as *const FMOD_VECTOR,
        up as *const FMOD_VECTOR,
    ));
}

#[no_mangle]
pub unsafe extern "C" fn Audio_Update() {
    FMODCALL(FMOD_System_Update(this.handle));
    Audio_SetListenerPos(this.autoPos, this.autoVel, this.autoFwd, this.autoUp);

    let mut soundsToRemove: Vec<usize> = Vec::new();
    for (i, sound) in this.playingSounds.iter().enumerate() {
        /* TODO : Refine the API to make this less awkward */
        if !Sound_IsFreed(&mut **sound) && Sound_IsPlaying(&mut **sound) as i32 != 0 {
            Sound_Update(&mut **sound);
        } else {
            soundsToRemove.push(i);
        }
    }
    for i in soundsToRemove.iter().rev() {
        this.playingSounds.swap_remove(*i);
    }

    for sound in this.freeingSounds.iter() {
        Audio_DeallocSound(*sound);
    }
    this.freeingSounds.clear();
}

#[no_mangle]
pub unsafe extern "C" fn Audio_GetLoadedCount() -> i32 {
    let size: u32 = StrMap_GetSize(&mut *this.descMap);
    // Assert(size <= INT32_MAX);
    return size as i32;
}

#[no_mangle]
pub unsafe extern "C" fn Audio_GetPlayingCount() -> i32 {
    this.playingSounds.len() as i32
}

#[no_mangle]
pub unsafe extern "C" fn Audio_GetTotalCount() -> i32 {
    let size: u32 = MemPool_GetSize(&mut *this.soundPool);
    // Assert(size <= INT32_MAX);
    size as i32
}

#[no_mangle]
pub unsafe extern "C" fn Audio_GetHandle() -> *mut libc::c_void {
    this.handle as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn Audio_AllocSoundDesc(name: *const libc::c_char) -> *mut SoundDesc {
    let mut desc: *mut SoundDesc = StrMap_Get(&mut *this.descMap, name) as *mut SoundDesc;
    if desc.is_null() {
        desc = MemNewZero!(SoundDesc);
        StrMap_Set(&mut *this.descMap, name, desc as *mut _);
    }
    desc
}

#[no_mangle]
pub unsafe extern "C" fn Audio_DeallocSoundDesc(desc: *mut SoundDesc) {
    StrMap_Remove(&mut *this.descMap, (*desc).name);
    MemFree(desc as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn Audio_AllocSound() -> *mut Sound {
    MemPool_Alloc(&mut *this.soundPool) as *mut Sound
}

#[no_mangle]
pub unsafe extern "C" fn Audio_DeallocSound(sound: *mut Sound) {
    MemPool_Dealloc(&mut *this.soundPool, sound as *mut _);
}

#[no_mangle]
pub unsafe extern "C" fn Audio_SoundStateChanged(sound: &mut Sound) {
    if Sound_IsFreed(sound) {
        this.freeingSounds.push(sound);
    } else if Sound_IsPlaying(sound) {
        this.playingSounds.push(sound);
        // CHECK1(
        //   if (ArrayList_GetSize(self.playingSounds) == AUDIO_CHANNELS + 1)
        //     Warn("Audio: Exceeded the number of available sound channels (%i)", AUDIO_CHANNELS);
        // )
    }
}
