use crate::audio::*;
use crate::common::*;
use crate::file::*;
use crate::internal::*;
use crate::*;

use crate::resource::*;
use crate::resource_type::*;
use crate::sound::FMODCALL;

use fmod_sys::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SoundDesc {
    pub _refCount: u32,
    pub handle: *mut FMOD_SOUND,
    pub name: *const libc::c_char,
    pub path: *const libc::c_char,
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_FinishLoad(this: &mut SoundDesc, func: *const libc::c_char) {
    let mut warned: bool = false;
    let mut openState: FMOD_OPENSTATE = FMOD_OPENSTATE::FMOD_OPENSTATE_READY;
    loop {
        FMODCALL(FMOD_Sound_GetOpenState(
            this.handle,
            &mut openState,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ));
        if openState == FMOD_OPENSTATE::FMOD_OPENSTATE_ERROR {
            CFatal!(
                "%s: Background file load has failed.\n  Path: %s",
                func,
                this.path,
            );
        }
        if openState == FMOD_OPENSTATE::FMOD_OPENSTATE_READY
            || openState == FMOD_OPENSTATE::FMOD_OPENSTATE_PLAYING
        {
            break;
        }
        if !warned {
            warned = true;
            CWarn!(
                "%s: Background file load hasn't finished. Blocking the main thread.\n  Path: %s",
                func,
                this.path,
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_Load(
    name: *const libc::c_char,
    immediate: bool,
    isLooped: bool,
    is3D: bool,
) -> *mut SoundDesc {
    let looped_str = if isLooped as i32 != 0 {
        "LOOPED:"
    } else {
        "UNLOOPED:"
    };
    let mapKey = static_string!(format!("{looped_str}{}", name.convert()));
    let this: *mut SoundDesc = Audio_AllocSoundDesc(mapKey);

    if ((*this).name).is_null() {
        let path: *const libc::c_char = Resource_GetPath(ResourceType_Sound, name);
        let mut mode: FMOD_MODE = 0 as FMOD_MODE;
        mode |= 0x100;
        mode |= 0x2000000;
        mode |= 0x4000;
        mode |= (if isLooped as i32 != 0 { 0x2 } else { 0x1 }) as u32;
        mode |= (if is3D as i32 != 0 {
            0x10 | 0x80000
        } else {
            0x8
        }) as u32;
        if !immediate {
            mode |= 0x10000;
        }
        FMODCALL(FMOD_System_CreateSound(
            Audio_GetHandle() as *mut FMOD_SYSTEM,
            path,
            mode,
            std::ptr::null_mut(),
            &mut (*this).handle,
        ));
        FMODCALL(FMOD_Sound_SetUserData((*this).handle, this as *mut _));
        (*this).name = StrDup(name);
        (*this).path = StrDup(path);
        (*this)._refCount = 1;
    } else {
        (*this)._refCount = ((*this)._refCount).wrapping_add(1);
        if immediate {
            SoundDesc_FinishLoad(&mut *this, c_str!("SoundDesc_Load"));
        }
    }
    this
}

#[no_mangle]
pub extern "C" fn SoundDesc_Acquire(this: &mut SoundDesc) {
    this._refCount = (this._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_Free(this: *mut SoundDesc) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0
    } {
        let name: *const libc::c_char = (*this).name;
        let path: *const libc::c_char = (*this).path;

        FMODCALL(FMOD_Sound_Release((*this).handle));

        Audio_DeallocSoundDesc(this);
        StrFree(name);
        StrFree(path);
        MemZero(this as *mut _, std::mem::size_of::<SoundDesc>());
    }
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_GetDuration(this: &mut SoundDesc) -> f32 {
    SoundDesc_FinishLoad(this, c_str!("SoundDesc_GetDuration"));
    let mut duration: u32 = 0;
    FMODCALL(FMOD_Sound_GetLength(
        this.handle,
        &mut duration,
        0x1 as i32 as FMOD_TIMEUNIT,
    ));
    duration as f32 / 1000.0f32
}

#[no_mangle]
pub extern "C" fn SoundDesc_GetName(this: &mut SoundDesc) -> *const libc::c_char {
    this.name
}

#[no_mangle]
pub extern "C" fn SoundDesc_GetPath(this: &mut SoundDesc) -> *const libc::c_char {
    this.path
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_ToFile(this: &mut SoundDesc, name: *const libc::c_char) {
    /* TODO : Finish this.
     *        There's some sort of signed/unsigned issue with the current
     *        implementation. 8-bit PCM is unsigned according to the spec.
     *        However, inspecting thybidding.wav in a hex editor sure looks like
     *        signed data to me. The data read from FMOD is 'correct' but appears
     *        to be unsigned (e.g. offset by a constant 0x80 / 0d128).
     *
     *        I'm extremely confused hy this as the FMOD data seems to be correct
     *        while the actual file appears to be incorrect, yet VLC and Audacity
     *        both play the original file correctly and the file output here
     *        sounds like ass. Further, exporting the original wav to a new file
     *        looks like the original file, further supporting that it is correct
     *        and well formed.
     *
     *        Possible solutions:
     *        1) Post a question on the FMOD site to gather more information.
     *        2) Use Sound::readData instead (probably going to get the same result)
     *        3) Create a second System and use System::setOutput to set
     *           FMOD_OUTPUTTYPE_WAVWRITER_NRT
     *
     *        I've already spent too much time on this so I'm tabling it until
     *        more of the audio API is fleshed out (namely, FMOD Studio is
     *        integrated, Rigidbody updates are processed, and HRTF solutions are
     *        explored).
     */
    SoundDesc_FinishLoad(this, c_str!("SoundDesc_ToFile"));

    let mut length: u32 = 0;
    let mut channels: i32 = 0;
    let mut bitsPerSample: i32 = 0;
    FMODCALL(FMOD_Sound_GetLength(
        this.handle,
        &mut length,
        FMOD_TIMEUNIT_RAWBYTES,
    ));
    FMODCALL(FMOD_Sound_GetFormat(
        this.handle,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut channels,
        &mut bitsPerSample,
    ));
    let bytesPerSample: i32 = bitsPerSample / 8;
    let mut sampleRate: f32 = 0.;
    FMOD_Sound_GetDefaults(this.handle, &mut sampleRate, std::ptr::null_mut());
    let mut ptr1: *mut libc::c_void = std::ptr::null_mut();
    let mut len1: u32 = 0;
    let mut ptr2: *mut libc::c_void = std::ptr::null_mut();
    let mut len2: u32 = 0;
    FMODCALL(FMOD_Sound_Lock(
        this.handle,
        0 as u32,
        length,
        &mut ptr1,
        &mut ptr2,
        &mut len1,
        &mut len2,
    ));

    /* Write the file */
    {
        let mut file: Box<File> = File_Create(name).unwrap_or_else(|| {
            CFatal!("SoundDesc_ToFile: Failed to create file.\nPath: %s", name,)
        });

        File_Write(file.as_mut(), c_str!("RIFF") as *const _, 4);
        File_WriteI32(file.as_mut(), (36_u32).wrapping_add(length) as i32);
        File_Write(file.as_mut(), c_str!("WAVE") as *const _, 4);
        File_Write(file.as_mut(), c_str!("fmt ") as *const _, 4);
        File_WriteI32(file.as_mut(), 16);
        File_WriteI16(file.as_mut(), 1);
        File_WriteI16(file.as_mut(), channels as i16);
        File_WriteI32(file.as_mut(), sampleRate as i32);
        File_WriteI32(
            file.as_mut(),
            ((bytesPerSample * channels) as f32 * sampleRate) as i32,
        );
        File_WriteI16(file.as_mut(), (bytesPerSample * channels) as i16);
        File_WriteI16(file.as_mut(), bitsPerSample as i16);
        File_Write(file.as_mut(), c_str!("data") as *const _, 4);
        File_WriteI32(file.as_mut(), length as i32);
        File_Write(file.as_mut(), ptr1, length);
    }

    FMODCALL(FMOD_Sound_Unlock(this.handle, ptr1, ptr2, len1, len2));
}
