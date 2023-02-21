use crate::internal::Memory::*;
use glam::Vec3;
use libc;
extern "C" {
    pub type FMOD_CHANNEL;
    pub type FMOD_SOUND;
    pub type FMOD_SYSTEM;
    pub type FMOD_CHANNELCONTROL;
    pub type FMOD_CHANNELGROUP;
    fn Audio_AllocSound() -> *mut Sound;
    fn Audio_GetHandle() -> *mut libc::c_void;
    fn Fatal(_: cstr, _: ...);
    fn Audio_SoundStateChanged(_: *mut Sound);
    fn FMOD_System_PlaySound(
        system: *mut FMOD_SYSTEM,
        sound: *mut FMOD_SOUND,
        channelgroup: *mut FMOD_CHANNELGROUP,
        paused: FMOD_BOOL,
        channel: *mut *mut FMOD_CHANNEL,
    ) -> FMOD_RESULT;
    fn FMOD_Channel_Stop(channel: *mut FMOD_CHANNEL) -> FMOD_RESULT;
    fn FMOD_Channel_SetPaused(channel: *mut FMOD_CHANNEL, paused: FMOD_BOOL) -> FMOD_RESULT;
    fn FMOD_Channel_SetVolume(channel: *mut FMOD_CHANNEL, volume: f32) -> FMOD_RESULT;
    fn FMOD_Channel_SetPitch(channel: *mut FMOD_CHANNEL, pitch: f32) -> FMOD_RESULT;
    fn FMOD_Channel_GetMode(channel: *mut FMOD_CHANNEL, mode: *mut FMOD_MODE) -> FMOD_RESULT;
    fn FMOD_Channel_SetCallback(
        channel: *mut FMOD_CHANNEL,
        callback: FMOD_CHANNELCONTROL_CALLBACK,
    ) -> FMOD_RESULT;
    fn FMOD_Channel_SetPan(channel: *mut FMOD_CHANNEL, pan: f32) -> FMOD_RESULT;
    fn FMOD_Channel_Set3DAttributes(
        channel: *mut FMOD_CHANNEL,
        pos: *const FMOD_VECTOR,
        vel: *const FMOD_VECTOR,
    ) -> FMOD_RESULT;
    fn FMOD_Channel_Set3DLevel(channel: *mut FMOD_CHANNEL, level: f32) -> FMOD_RESULT;
    fn FMOD_Channel_SetUserData(
        channel: *mut FMOD_CHANNEL,
        userdata: *mut libc::c_void,
    ) -> FMOD_RESULT;
    fn FMOD_Channel_GetUserData(
        channel: *mut FMOD_CHANNEL,
        userdata: *mut *mut libc::c_void,
    ) -> FMOD_RESULT;
    fn FMOD_Channel_SetPosition(
        channel: *mut FMOD_CHANNEL,
        position: u32,
        postype: FMOD_TIMEUNIT,
    ) -> FMOD_RESULT;
    fn floor(_: f64) -> f64;
    fn SoundDesc_Load(name: cstr, immediate: bool, isLooped: bool, is3D: bool) -> *mut SoundDesc;
    fn SoundDesc_FinishLoad(_: *mut SoundDesc, func: cstr);
    fn SoundDesc_Acquire(_: *mut SoundDesc);
    fn SoundDesc_Free(_: *mut SoundDesc);
    fn SoundDesc_GetDuration(_: *mut SoundDesc) -> f32;
    fn SoundDesc_GetName(_: *mut SoundDesc) -> cstr;
    fn SoundDesc_GetPath(_: *mut SoundDesc) -> cstr;
    fn SoundDesc_ToFile(_: *mut SoundDesc, _: cstr);
}
pub type cstr = *const libc::c_char;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SoundDesc {
    pub _refCount: u32,
    pub handle: *mut FMOD_SOUND,
    pub name: cstr,
    pub path: cstr,
}
pub type FMOD_BOOL = i32;
pub type FMOD_RESULT = u32;
pub const FMOD_RESULT_FORCEINT: FMOD_RESULT = 65536;
pub const FMOD_ERR_TOOMANYSAMPLES: FMOD_RESULT = 81;
pub const FMOD_ERR_RECORD_DISCONNECTED: FMOD_RESULT = 80;
pub const FMOD_ERR_NOT_LOCKED: FMOD_RESULT = 79;
pub const FMOD_ERR_ALREADY_LOCKED: FMOD_RESULT = 78;
pub const FMOD_ERR_INVALID_STRING: FMOD_RESULT = 77;
pub const FMOD_ERR_STUDIO_NOT_LOADED: FMOD_RESULT = 76;
pub const FMOD_ERR_STUDIO_UNINITIALIZED: FMOD_RESULT = 75;
pub const FMOD_ERR_EVENT_NOTFOUND: FMOD_RESULT = 74;
pub const FMOD_ERR_EVENT_LIVEUPDATE_TIMEOUT: FMOD_RESULT = 73;
pub const FMOD_ERR_EVENT_LIVEUPDATE_MISMATCH: FMOD_RESULT = 72;
pub const FMOD_ERR_EVENT_LIVEUPDATE_BUSY: FMOD_RESULT = 71;
pub const FMOD_ERR_EVENT_ALREADY_LOADED: FMOD_RESULT = 70;
pub const FMOD_ERR_VERSION: FMOD_RESULT = 69;
pub const FMOD_ERR_UNSUPPORTED: FMOD_RESULT = 68;
pub const FMOD_ERR_UNINITIALIZED: FMOD_RESULT = 67;
pub const FMOD_ERR_UNIMPLEMENTED: FMOD_RESULT = 66;
pub const FMOD_ERR_TRUNCATED: FMOD_RESULT = 65;
pub const FMOD_ERR_TOOMANYCHANNELS: FMOD_RESULT = 64;
pub const FMOD_ERR_TAGNOTFOUND: FMOD_RESULT = 63;
pub const FMOD_ERR_SUBSOUND_CANTMOVE: FMOD_RESULT = 62;
pub const FMOD_ERR_SUBSOUND_ALLOCATED: FMOD_RESULT = 61;
pub const FMOD_ERR_SUBSOUNDS: FMOD_RESULT = 60;
pub const FMOD_ERR_REVERB_INSTANCE: FMOD_RESULT = 59;
pub const FMOD_ERR_REVERB_CHANNELGROUP: FMOD_RESULT = 58;
pub const FMOD_ERR_RECORD: FMOD_RESULT = 57;
pub const FMOD_ERR_PLUGIN_VERSION: FMOD_RESULT = 56;
pub const FMOD_ERR_PLUGIN_RESOURCE: FMOD_RESULT = 55;
pub const FMOD_ERR_PLUGIN_MISSING: FMOD_RESULT = 54;
pub const FMOD_ERR_PLUGIN: FMOD_RESULT = 53;
pub const FMOD_ERR_OUTPUT_NODRIVERS: FMOD_RESULT = 52;
pub const FMOD_ERR_OUTPUT_INIT: FMOD_RESULT = 51;
pub const FMOD_ERR_OUTPUT_FORMAT: FMOD_RESULT = 50;
pub const FMOD_ERR_OUTPUT_DRIVERCALL: FMOD_RESULT = 49;
pub const FMOD_ERR_OUTPUT_CREATEBUFFER: FMOD_RESULT = 48;
pub const FMOD_ERR_OUTPUT_ALLOCATED: FMOD_RESULT = 47;
pub const FMOD_ERR_NOTREADY: FMOD_RESULT = 46;
pub const FMOD_ERR_NET_WOULD_BLOCK: FMOD_RESULT = 45;
pub const FMOD_ERR_NET_URL: FMOD_RESULT = 44;
pub const FMOD_ERR_NET_SOCKET_ERROR: FMOD_RESULT = 43;
pub const FMOD_ERR_NET_CONNECT: FMOD_RESULT = 42;
pub const FMOD_ERR_NEEDSHARDWARE: FMOD_RESULT = 41;
pub const FMOD_ERR_NEEDS3D: FMOD_RESULT = 40;
pub const FMOD_ERR_MEMORY_CANTPOINT: FMOD_RESULT = 39;
pub const FMOD_ERR_MEMORY: FMOD_RESULT = 38;
pub const FMOD_ERR_MAXAUDIBLE: FMOD_RESULT = 37;
pub const FMOD_ERR_INVALID_VECTOR: FMOD_RESULT = 36;
pub const FMOD_ERR_INVALID_THREAD: FMOD_RESULT = 35;
pub const FMOD_ERR_INVALID_SYNCPOINT: FMOD_RESULT = 34;
pub const FMOD_ERR_INVALID_SPEAKER: FMOD_RESULT = 33;
pub const FMOD_ERR_INVALID_POSITION: FMOD_RESULT = 32;
pub const FMOD_ERR_INVALID_PARAM: FMOD_RESULT = 31;
pub const FMOD_ERR_INVALID_HANDLE: FMOD_RESULT = 30;
pub const FMOD_ERR_INVALID_FLOAT: FMOD_RESULT = 29;
pub const FMOD_ERR_INTERNAL: FMOD_RESULT = 28;
pub const FMOD_ERR_INITIALIZED: FMOD_RESULT = 27;
pub const FMOD_ERR_INITIALIZATION: FMOD_RESULT = 26;
pub const FMOD_ERR_HTTP_TIMEOUT: FMOD_RESULT = 25;
pub const FMOD_ERR_HTTP_SERVER_ERROR: FMOD_RESULT = 24;
pub const FMOD_ERR_HTTP_PROXY_AUTH: FMOD_RESULT = 23;
pub const FMOD_ERR_HTTP_ACCESS: FMOD_RESULT = 22;
pub const FMOD_ERR_HTTP: FMOD_RESULT = 21;
pub const FMOD_ERR_HEADER_MISMATCH: FMOD_RESULT = 20;
pub const FMOD_ERR_FORMAT: FMOD_RESULT = 19;
pub const FMOD_ERR_FILE_NOTFOUND: FMOD_RESULT = 18;
pub const FMOD_ERR_FILE_ENDOFDATA: FMOD_RESULT = 17;
pub const FMOD_ERR_FILE_EOF: FMOD_RESULT = 16;
pub const FMOD_ERR_FILE_DISKEJECTED: FMOD_RESULT = 15;
pub const FMOD_ERR_FILE_COULDNOTSEEK: FMOD_RESULT = 14;
pub const FMOD_ERR_FILE_BAD: FMOD_RESULT = 13;
pub const FMOD_ERR_DSP_TYPE: FMOD_RESULT = 12;
pub const FMOD_ERR_DSP_SILENCE: FMOD_RESULT = 11;
pub const FMOD_ERR_DSP_RESERVED: FMOD_RESULT = 10;
pub const FMOD_ERR_DSP_NOTFOUND: FMOD_RESULT = 9;
pub const FMOD_ERR_DSP_INUSE: FMOD_RESULT = 8;
pub const FMOD_ERR_DSP_FORMAT: FMOD_RESULT = 7;
pub const FMOD_ERR_DSP_DONTPROCESS: FMOD_RESULT = 6;
pub const FMOD_ERR_DSP_CONNECTION: FMOD_RESULT = 5;
pub const FMOD_ERR_DMA: FMOD_RESULT = 4;
pub const FMOD_ERR_CHANNEL_STOLEN: FMOD_RESULT = 3;
pub const FMOD_ERR_CHANNEL_ALLOC: FMOD_RESULT = 2;
pub const FMOD_ERR_BADCOMMAND: FMOD_RESULT = 1;
pub const FMOD_OK: FMOD_RESULT = 0;
pub type FMOD_TIMEUNIT = u32;
pub type FMOD_MODE = u32;
pub type FMOD_CHANNELCONTROL_TYPE = u32;
pub const FMOD_CHANNELCONTROL_FORCEINT: FMOD_CHANNELCONTROL_TYPE = 65536;
pub const FMOD_CHANNELCONTROL_MAX: FMOD_CHANNELCONTROL_TYPE = 2;
pub const FMOD_CHANNELCONTROL_CHANNELGROUP: FMOD_CHANNELCONTROL_TYPE = 1;
pub const FMOD_CHANNELCONTROL_CHANNEL: FMOD_CHANNELCONTROL_TYPE = 0;
pub type FMOD_CHANNELCONTROL_CALLBACK_TYPE = u32;
pub const FMOD_CHANNELCONTROL_CALLBACK_FORCEINT: FMOD_CHANNELCONTROL_CALLBACK_TYPE = 65536;
pub const FMOD_CHANNELCONTROL_CALLBACK_MAX: FMOD_CHANNELCONTROL_CALLBACK_TYPE = 4;
pub const FMOD_CHANNELCONTROL_CALLBACK_OCCLUSION: FMOD_CHANNELCONTROL_CALLBACK_TYPE = 3;
pub const FMOD_CHANNELCONTROL_CALLBACK_SYNCPOINT: FMOD_CHANNELCONTROL_CALLBACK_TYPE = 2;
pub const FMOD_CHANNELCONTROL_CALLBACK_VIRTUALVOICE: FMOD_CHANNELCONTROL_CALLBACK_TYPE = 1;
pub const FMOD_CHANNELCONTROL_CALLBACK_END: FMOD_CHANNELCONTROL_CALLBACK_TYPE = 0;
pub type FMOD_CHANNELCONTROL_CALLBACK = Option<
    unsafe extern "C" fn(
        *mut FMOD_CHANNELCONTROL,
        FMOD_CHANNELCONTROL_TYPE,
        FMOD_CHANNELCONTROL_CALLBACK_TYPE,
        *mut libc::c_void,
        *mut libc::c_void,
    ) -> FMOD_RESULT,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FMOD_VECTOR {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
#[inline]
unsafe extern "C" fn FMOD_CheckError(
    mut result: FMOD_RESULT,
    mut file: cstr,
    mut line: i32,
    mut func: cstr,
) {
    if result as u32 != FMOD_OK as i32 as u32 {
        Fatal(
            b"%s: %s\n%s\n  [%s @ Line %d]\0" as *const u8 as *const libc::c_char,
            func,
            FMODError_ToString(result),
            FMOD_ErrorString(result),
            file,
            line,
        );
    }
}
#[inline]
unsafe extern "C" fn FMODError_ToString(mut this: FMOD_RESULT) -> cstr {
    match this as u32 {
        0 => return b"FMOD_OK\0" as *const u8 as *const libc::c_char,
        1 => return b"FMOD_ERR_BADCOMMAND\0" as *const u8 as *const libc::c_char,
        2 => return b"FMOD_ERR_CHANNEL_ALLOC\0" as *const u8 as *const libc::c_char,
        3 => return b"FMOD_ERR_CHANNEL_STOLEN\0" as *const u8 as *const libc::c_char,
        4 => return b"FMOD_ERR_DMA\0" as *const u8 as *const libc::c_char,
        5 => return b"FMOD_ERR_DSP_CONNECTION\0" as *const u8 as *const libc::c_char,
        6 => return b"FMOD_ERR_DSP_DONTPROCESS\0" as *const u8 as *const libc::c_char,
        7 => return b"FMOD_ERR_DSP_FORMAT\0" as *const u8 as *const libc::c_char,
        8 => return b"FMOD_ERR_DSP_INUSE\0" as *const u8 as *const libc::c_char,
        9 => return b"FMOD_ERR_DSP_NOTFOUND\0" as *const u8 as *const libc::c_char,
        10 => return b"FMOD_ERR_DSP_RESERVED\0" as *const u8 as *const libc::c_char,
        11 => return b"FMOD_ERR_DSP_SILENCE\0" as *const u8 as *const libc::c_char,
        12 => return b"FMOD_ERR_DSP_TYPE\0" as *const u8 as *const libc::c_char,
        13 => return b"FMOD_ERR_FILE_BAD\0" as *const u8 as *const libc::c_char,
        14 => return b"FMOD_ERR_FILE_COULDNOTSEEK\0" as *const u8 as *const libc::c_char,
        15 => return b"FMOD_ERR_FILE_DISKEJECTED\0" as *const u8 as *const libc::c_char,
        16 => return b"FMOD_ERR_FILE_EOF\0" as *const u8 as *const libc::c_char,
        17 => return b"FMOD_ERR_FILE_ENDOFDATA\0" as *const u8 as *const libc::c_char,
        18 => return b"FMOD_ERR_FILE_NOTFOUND\0" as *const u8 as *const libc::c_char,
        19 => return b"FMOD_ERR_FORMAT\0" as *const u8 as *const libc::c_char,
        20 => return b"FMOD_ERR_HEADER_MISMATCH\0" as *const u8 as *const libc::c_char,
        21 => return b"FMOD_ERR_HTTP\0" as *const u8 as *const libc::c_char,
        22 => return b"FMOD_ERR_HTTP_ACCESS\0" as *const u8 as *const libc::c_char,
        23 => return b"FMOD_ERR_HTTP_PROXY_AUTH\0" as *const u8 as *const libc::c_char,
        24 => return b"FMOD_ERR_HTTP_SERVER_ERROR\0" as *const u8 as *const libc::c_char,
        25 => return b"FMOD_ERR_HTTP_TIMEOUT\0" as *const u8 as *const libc::c_char,
        26 => return b"FMOD_ERR_INITIALIZATION\0" as *const u8 as *const libc::c_char,
        27 => return b"FMOD_ERR_INITIALIZED\0" as *const u8 as *const libc::c_char,
        28 => return b"FMOD_ERR_INTERNAL\0" as *const u8 as *const libc::c_char,
        29 => return b"FMOD_ERR_INVALID_FLOAT\0" as *const u8 as *const libc::c_char,
        30 => return b"FMOD_ERR_INVALID_HANDLE\0" as *const u8 as *const libc::c_char,
        31 => return b"FMOD_ERR_INVALID_PARAM\0" as *const u8 as *const libc::c_char,
        32 => return b"FMOD_ERR_INVALID_POSITION\0" as *const u8 as *const libc::c_char,
        33 => return b"FMOD_ERR_INVALID_SPEAKER\0" as *const u8 as *const libc::c_char,
        34 => return b"FMOD_ERR_INVALID_SYNCPOINT\0" as *const u8 as *const libc::c_char,
        35 => return b"FMOD_ERR_INVALID_THREAD\0" as *const u8 as *const libc::c_char,
        36 => return b"FMOD_ERR_INVALID_VECTOR\0" as *const u8 as *const libc::c_char,
        37 => return b"FMOD_ERR_MAXAUDIBLE\0" as *const u8 as *const libc::c_char,
        38 => return b"FMOD_ERR_MEMORY\0" as *const u8 as *const libc::c_char,
        39 => return b"FMOD_ERR_MEMORY_CANTPOINT\0" as *const u8 as *const libc::c_char,
        40 => return b"FMOD_ERR_NEEDS3D\0" as *const u8 as *const libc::c_char,
        41 => return b"FMOD_ERR_NEEDSHARDWARE\0" as *const u8 as *const libc::c_char,
        42 => return b"FMOD_ERR_NET_CONNECT\0" as *const u8 as *const libc::c_char,
        43 => return b"FMOD_ERR_NET_SOCKET_ERROR\0" as *const u8 as *const libc::c_char,
        44 => return b"FMOD_ERR_NET_URL\0" as *const u8 as *const libc::c_char,
        45 => return b"FMOD_ERR_NET_WOULD_BLOCK\0" as *const u8 as *const libc::c_char,
        46 => return b"FMOD_ERR_NOTREADY\0" as *const u8 as *const libc::c_char,
        47 => return b"FMOD_ERR_OUTPUT_ALLOCATED\0" as *const u8 as *const libc::c_char,
        48 => {
            return b"FMOD_ERR_OUTPUT_CREATEBUFFER\0" as *const u8 as *const libc::c_char;
        }
        49 => return b"FMOD_ERR_OUTPUT_DRIVERCALL\0" as *const u8 as *const libc::c_char,
        50 => return b"FMOD_ERR_OUTPUT_FORMAT\0" as *const u8 as *const libc::c_char,
        51 => return b"FMOD_ERR_OUTPUT_INIT\0" as *const u8 as *const libc::c_char,
        52 => return b"FMOD_ERR_OUTPUT_NODRIVERS\0" as *const u8 as *const libc::c_char,
        53 => return b"FMOD_ERR_PLUGIN\0" as *const u8 as *const libc::c_char,
        54 => return b"FMOD_ERR_PLUGIN_MISSING\0" as *const u8 as *const libc::c_char,
        55 => return b"FMOD_ERR_PLUGIN_RESOURCE\0" as *const u8 as *const libc::c_char,
        56 => return b"FMOD_ERR_PLUGIN_VERSION\0" as *const u8 as *const libc::c_char,
        57 => return b"FMOD_ERR_RECORD\0" as *const u8 as *const libc::c_char,
        58 => {
            return b"FMOD_ERR_REVERB_CHANNELGROUP\0" as *const u8 as *const libc::c_char;
        }
        59 => return b"FMOD_ERR_REVERB_INSTANCE\0" as *const u8 as *const libc::c_char,
        60 => return b"FMOD_ERR_SUBSOUNDS\0" as *const u8 as *const libc::c_char,
        61 => return b"FMOD_ERR_SUBSOUND_ALLOCATED\0" as *const u8 as *const libc::c_char,
        62 => return b"FMOD_ERR_SUBSOUND_CANTMOVE\0" as *const u8 as *const libc::c_char,
        63 => return b"FMOD_ERR_TAGNOTFOUND\0" as *const u8 as *const libc::c_char,
        64 => return b"FMOD_ERR_TOOMANYCHANNELS\0" as *const u8 as *const libc::c_char,
        65 => return b"FMOD_ERR_TRUNCATED\0" as *const u8 as *const libc::c_char,
        66 => return b"FMOD_ERR_UNIMPLEMENTED\0" as *const u8 as *const libc::c_char,
        67 => return b"FMOD_ERR_UNINITIALIZED\0" as *const u8 as *const libc::c_char,
        68 => return b"FMOD_ERR_UNSUPPORTED\0" as *const u8 as *const libc::c_char,
        69 => return b"FMOD_ERR_VERSION\0" as *const u8 as *const libc::c_char,
        70 => {
            return b"FMOD_ERR_EVENT_ALREADY_LOADED\0" as *const u8 as *const libc::c_char;
        }
        71 => {
            return b"FMOD_ERR_EVENT_LIVEUPDATE_BUSY\0" as *const u8 as *const libc::c_char;
        }
        72 => {
            return b"FMOD_ERR_EVENT_LIVEUPDATE_MISMATCH\0" as *const u8 as *const libc::c_char;
        }
        73 => {
            return b"FMOD_ERR_EVENT_LIVEUPDATE_TIMEOUT\0" as *const u8 as *const libc::c_char;
        }
        74 => return b"FMOD_ERR_EVENT_NOTFOUND\0" as *const u8 as *const libc::c_char,
        75 => {
            return b"FMOD_ERR_STUDIO_UNINITIALIZED\0" as *const u8 as *const libc::c_char;
        }
        76 => return b"FMOD_ERR_STUDIO_NOT_LOADED\0" as *const u8 as *const libc::c_char,
        77 => return b"FMOD_ERR_INVALID_STRING\0" as *const u8 as *const libc::c_char,
        78 => return b"FMOD_ERR_ALREADY_LOCKED\0" as *const u8 as *const libc::c_char,
        79 => return b"FMOD_ERR_NOT_LOCKED\0" as *const u8 as *const libc::c_char,
        80 => {
            return b"FMOD_ERR_RECORD_DISCONNECTED\0" as *const u8 as *const libc::c_char;
        }
        81 => return b"FMOD_ERR_TOOMANYSAMPLES\0" as *const u8 as *const libc::c_char,
        65536 => return b"FMOD_RESULT_FORCEINT\0" as *const u8 as *const libc::c_char,
        _ => {}
    }
    return b"Unknown Error\0" as *const u8 as *const libc::c_char;
}
unsafe extern "C" fn FMOD_ErrorString(mut errcode: FMOD_RESULT) -> *const libc::c_char {
    match errcode as u32 {
        0 => return b"No errors.\0" as *const u8 as *const libc::c_char,
        1 => {
            return b"Tried to call a function on a data type that does not allow this type of functionality (ie calling Sound::lock on a streaming sound).\0"
                as *const u8 as *const libc::c_char;
        }
        2 => {
            return b"Error trying to allocate a channel.\0" as *const u8 as *const libc::c_char;
        }
        3 => {
            return b"The specified channel has been reused to play another sound.\0" as *const u8
                as *const libc::c_char;
        }
        4 => {
            return b"DMA Failure.  See debug output for more information.\0" as *const u8
                as *const libc::c_char;
        }
        5 => {
            return b"DSP connection error.  Connection possibly caused a cyclic dependency or connected dsps with incompatible buffer counts.\0"
                as *const u8 as *const libc::c_char;
        }
        6 => {
            return b"DSP return code from a DSP process query callback.  Tells mixer not to call the process callback and therefore not consume CPU.  Use this to optimize the DSP graph.\0"
                as *const u8 as *const libc::c_char;
        }
        7 => {
            return b"DSP Format error.  A DSP unit may have attempted to connect to this network with the wrong format, or a matrix may have been set with the wrong size if the target unit has a specified channel map.\0"
                as *const u8 as *const libc::c_char;
        }
        8 => {
            return b"DSP is already in the mixer's DSP network. It must be removed before being reinserted or released.\0"
                as *const u8 as *const libc::c_char;
        }
        9 => {
            return b"DSP connection error.  Couldn't find the DSP unit specified.\0" as *const u8
                as *const libc::c_char;
        }
        10 => {
            return b"DSP operation error.  Cannot perform operation on this DSP as it is reserved by the system.\0"
                as *const u8 as *const libc::c_char;
        }
        11 => {
            return b"DSP return code from a DSP process query callback.  Tells mixer silence would be produced from read, so go idle and not consume CPU.  Use this to optimize the DSP graph.\0"
                as *const u8 as *const libc::c_char;
        }
        12 => {
            return b"DSP operation cannot be performed on a DSP of this type.\0" as *const u8
                as *const libc::c_char;
        }
        13 => return b"Error loading file.\0" as *const u8 as *const libc::c_char,
        14 => {
            return b"Couldn't perform seek operation.  This is a limitation of the medium (ie netstreams) or the file format.\0"
                as *const u8 as *const libc::c_char;
        }
        15 => {
            return b"Media was ejected while reading.\0" as *const u8 as *const libc::c_char;
        }
        16 => {
            return b"End of file unexpectedly reached while trying to read essential data (truncated?).\0"
                as *const u8 as *const libc::c_char;
        }
        17 => {
            return b"End of current chunk reached while trying to read data.\0" as *const u8
                as *const libc::c_char;
        }
        18 => return b"File not found.\0" as *const u8 as *const libc::c_char,
        19 => {
            return b"Unsupported file or audio format.\0" as *const u8 as *const libc::c_char;
        }
        20 => {
            return b"There is a version mismatch between the FMOD header and either the FMOD Studio library or the FMOD Low Level library.\0"
                as *const u8 as *const libc::c_char;
        }
        21 => {
            return b"A HTTP error occurred. This is a catch-all for HTTP errors not listed elsewhere.\0"
                as *const u8 as *const libc::c_char;
        }
        22 => {
            return b"The specified resource requires authentication or is forbidden.\0" as *const u8
                as *const libc::c_char;
        }
        23 => {
            return b"Proxy authentication is required to access the specified resource.\0"
                as *const u8 as *const libc::c_char;
        }
        24 => {
            return b"A HTTP server error occurred.\0" as *const u8 as *const libc::c_char;
        }
        25 => return b"The HTTP request timed out.\0" as *const u8 as *const libc::c_char,
        26 => {
            return b"FMOD was not initialized correctly to support this function.\0" as *const u8
                as *const libc::c_char;
        }
        27 => {
            return b"Cannot call this command after System::init.\0" as *const u8
                as *const libc::c_char;
        }
        28 => {
            return b"An error occurred that wasn't supposed to.  Contact support.\0" as *const u8
                as *const libc::c_char;
        }
        29 => {
            return b"Value passed in was a NaN, Inf or denormalized float.\0" as *const u8
                as *const libc::c_char;
        }
        30 => {
            return b"An invalid object handle was used.\0" as *const u8 as *const libc::c_char;
        }
        31 => {
            return b"An invalid parameter was passed to this function.\0" as *const u8
                as *const libc::c_char;
        }
        32 => {
            return b"An invalid seek position was passed to this function.\0" as *const u8
                as *const libc::c_char;
        }
        33 => {
            return b"An invalid speaker was passed to this function based on the current speaker mode.\0"
                as *const u8 as *const libc::c_char;
        }
        34 => {
            return b"The syncpoint did not come from this sound handle.\0" as *const u8
                as *const libc::c_char;
        }
        35 => {
            return b"Tried to call a function on a thread that is not supported.\0" as *const u8
                as *const libc::c_char;
        }
        36 => {
            return b"The vectors passed in are not unit length, or perpendicular.\0" as *const u8
                as *const libc::c_char;
        }
        37 => {
            return b"Reached maximum audible playback count for this sound's soundgroup.\0"
                as *const u8 as *const libc::c_char;
        }
        38 => {
            return b"Not enough memory or resources.\0" as *const u8 as *const libc::c_char;
        }
        39 => {
            return b"Can't use FMOD_OPENMEMORY_POINT on non PCM source data, or non mp3/xma/adpcm data if FMOD_CREATECOMPRESSEDSAMPLE was used.\0"
                as *const u8 as *const libc::c_char;
        }
        40 => {
            return b"Tried to call a command on a 2d sound when the command was meant for 3d sound.\0"
                as *const u8 as *const libc::c_char;
        }
        41 => {
            return b"Tried to use a feature that requires hardware support.\0" as *const u8
                as *const libc::c_char;
        }
        42 => {
            return b"Couldn't connect to the specified host.\0" as *const u8
                as *const libc::c_char;
        }
        43 => {
            return b"A socket error occurred.  This is a catch-all for socket-related errors not listed elsewhere.\0"
                as *const u8 as *const libc::c_char;
        }
        44 => {
            return b"The specified URL couldn't be resolved.\0" as *const u8
                as *const libc::c_char;
        }
        45 => {
            return b"Operation on a non-blocking socket could not complete immediately.\0"
                as *const u8 as *const libc::c_char;
        }
        46 => {
            return b"Operation could not be performed because specified sound/DSP connection is not ready.\0"
                as *const u8 as *const libc::c_char;
        }
        47 => {
            return b"Error initializing output device, but more specifically, the output device is already in use and cannot be reused.\0"
                as *const u8 as *const libc::c_char;
        }
        48 => {
            return b"Error creating hardware sound buffer.\0" as *const u8 as *const libc::c_char;
        }
        49 => {
            return b"A call to a standard soundcard driver failed, which could possibly mean a bug in the driver or resources were missing or exhausted.\0"
                as *const u8 as *const libc::c_char;
        }
        50 => {
            return b"Soundcard does not support the specified format.\0" as *const u8
                as *const libc::c_char;
        }
        51 => {
            return b"Error initializing output device.\0" as *const u8 as *const libc::c_char;
        }
        52 => {
            return b"The output device has no drivers installed.  If pre-init, FMOD_OUTPUT_NOSOUND is selected as the output mode.  If post-init, the function just fails.\0"
                as *const u8 as *const libc::c_char;
        }
        53 => {
            return b"An unspecified error has been returned from a plugin.\0" as *const u8
                as *const libc::c_char;
        }
        54 => {
            return b"A requested output, dsp unit type or codec was not available.\0" as *const u8
                as *const libc::c_char;
        }
        55 => {
            return b"A resource that the plugin requires cannot be allocated or found. (ie the DLS file for MIDI playback)\0"
                as *const u8 as *const libc::c_char;
        }
        56 => {
            return b"A plugin was built with an unsupported SDK version.\0" as *const u8
                as *const libc::c_char;
        }
        57 => {
            return b"An error occurred trying to initialize the recording device.\0" as *const u8
                as *const libc::c_char;
        }
        58 => {
            return b"Reverb properties cannot be set on this channel because a parent channelgroup owns the reverb connection.\0"
                as *const u8 as *const libc::c_char;
        }
        59 => {
            return b"Specified instance in FMOD_REVERB_PROPERTIES couldn't be set. Most likely because it is an invalid instance number or the reverb doesn't exist.\0"
                as *const u8 as *const libc::c_char;
        }
        60 => {
            return b"The error occurred because the sound referenced contains subsounds when it shouldn't have, or it doesn't contain subsounds when it should have.  The operation may also not be able to be performed on a parent sound.\0"
                as *const u8 as *const libc::c_char;
        }
        61 => {
            return b"This subsound is already being used by another sound, you cannot have more than one parent to a sound.  Null out the other parent's entry first.\0"
                as *const u8 as *const libc::c_char;
        }
        62 => {
            return b"Shared subsounds cannot be replaced or moved from their parent stream, such as when the parent stream is an FSB file.\0"
                as *const u8 as *const libc::c_char;
        }
        63 => {
            return b"The specified tag could not be found or there are no tags.\0" as *const u8
                as *const libc::c_char;
        }
        64 => {
            return b"The sound created exceeds the allowable input channel count.  This can be increased using the 'maxinputchannels' parameter in System::setSoftwareFormat.\0"
                as *const u8 as *const libc::c_char;
        }
        65 => {
            return b"The retrieved string is too long to fit in the supplied buffer and has been truncated.\0"
                as *const u8 as *const libc::c_char;
        }
        66 => {
            return b"Something in FMOD hasn't been implemented when it should be! contact support!\0"
                as *const u8 as *const libc::c_char;
        }
        67 => {
            return b"This command failed because System::init or System::setDriver was not called.\0"
                as *const u8 as *const libc::c_char;
        }
        68 => {
            return b"A command issued was not supported by this object.  Possibly a plugin without certain callbacks specified.\0"
                as *const u8 as *const libc::c_char;
        }
        69 => {
            return b"The version number of this file format is not supported.\0" as *const u8
                as *const libc::c_char;
        }
        70 => {
            return b"The specified bank has already been loaded.\0" as *const u8
                as *const libc::c_char;
        }
        71 => {
            return b"The live update connection failed due to the game already being connected.\0"
                as *const u8 as *const libc::c_char;
        }
        72 => {
            return b"The live update connection failed due to the game data being out of sync with the tool.\0"
                as *const u8 as *const libc::c_char;
        }
        73 => {
            return b"The live update connection timed out.\0" as *const u8 as *const libc::c_char;
        }
        74 => {
            return b"The requested event, parameter, bus or vca could not be found.\0" as *const u8
                as *const libc::c_char;
        }
        75 => {
            return b"The Studio::System object is not yet initialized.\0" as *const u8
                as *const libc::c_char;
        }
        76 => {
            return b"The specified resource is not loaded, so it can't be unloaded.\0" as *const u8
                as *const libc::c_char;
        }
        77 => {
            return b"An invalid string was passed to this function.\0" as *const u8
                as *const libc::c_char;
        }
        78 => {
            return b"The specified resource is already locked.\0" as *const u8
                as *const libc::c_char;
        }
        79 => {
            return b"The specified resource is not locked, so it can't be unlocked.\0" as *const u8
                as *const libc::c_char;
        }
        80 => {
            return b"The specified recording driver has been disconnected.\0" as *const u8
                as *const libc::c_char;
        }
        81 => {
            return b"The length provided exceeds the allowable limit.\0" as *const u8
                as *const libc::c_char;
        }
        _ => return b"Unknown error.\0" as *const u8 as *const libc::c_char,
    };
}
#[inline]
unsafe extern "C" fn Floor(mut t: f64) -> f64 {
    return floor(t);
}
#[inline]
unsafe extern "C" fn Round(mut t: f64) -> f64 {
    return Floor(t + 0.5f64);
}
unsafe extern "C" fn Sound_Callback(
    mut channel: *mut FMOD_CHANNELCONTROL,
    mut controlType: FMOD_CHANNELCONTROL_TYPE,
    mut callbackType: FMOD_CHANNELCONTROL_CALLBACK_TYPE,
    mut a: *mut libc::c_void,
    mut b: *mut libc::c_void,
) -> FMOD_RESULT {
    if callbackType as u32 == FMOD_CHANNELCONTROL_CALLBACK_END as i32 as u32 {
        let mut this: *mut Sound = 0 as *mut Sound;
        FMOD_CheckError(
            FMOD_Channel_GetUserData(
                channel as *mut FMOD_CHANNEL,
                &mut this as *mut *mut Sound as *mut *mut libc::c_void,
            ),
            b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
                as *const libc::c_char,
            23 as i32,
            (*::core::mem::transmute::<&[u8; 15], &[libc::c_char; 15]>(b"Sound_Callback\0"))
                .as_ptr(),
        );
        Sound_SetState(this, 4 as i32 as SoundState);
    }
    return FMOD_OK;
}
#[inline]
unsafe extern "C" fn Sound_EnsureLoadedImpl(mut this: *mut Sound, mut func: cstr) {
    if (*this).state as i32 == 1 as i32 {
        SoundDesc_FinishLoad((*this).desc, func);
        // FMOD_CheckError(
        //     FMOD_System_PlaySound(
        //         Audio_GetHandle() as *mut FMOD_SYSTEM,
        //         (*(*this).desc).handle,
        //         0 as *mut FMOD_CHANNELGROUP,
        //         1 as i32,
        //         &mut (*this).handle,
        //     ),
        //     b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
        //         as *const libc::c_char,
        //     33 as i32,
        //     (*::core::mem::transmute::<
        //         &[u8; 23],
        //         &[libc::c_char; 23],
        //     >(b"Sound_EnsureLoadedImpl\0"))
        //         .as_ptr(),
        // );
        // FMOD_CheckError(
        //     FMOD_Channel_SetUserData((*this).handle, this as *mut libc::c_void),
        //     b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
        //         as *const libc::c_char,
        //     34 as i32,
        //     (*::core::mem::transmute::<
        //         &[u8; 23],
        //         &[libc::c_char; 23],
        //     >(b"Sound_EnsureLoadedImpl\0"))
        //         .as_ptr(),
        // );
        // FMOD_CheckError(
        //     FMOD_Channel_SetCallback(
        //         (*this).handle,
        //         Some(
        //             Sound_Callback
        //                 as unsafe extern "C" fn(
        //                     *mut FMOD_CHANNELCONTROL,
        //                     FMOD_CHANNELCONTROL_TYPE,
        //                     FMOD_CHANNELCONTROL_CALLBACK_TYPE,
        //                     *mut libc::c_void,
        //                     *mut libc::c_void,
        //                 ) -> FMOD_RESULT,
        //         ),
        //     ),
        //     b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
        //         as *const libc::c_char,
        //     35 as i32,
        //     (*::core::mem::transmute::<
        //         &[u8; 23],
        //         &[libc::c_char; 23],
        //     >(b"Sound_EnsureLoadedImpl\0"))
        //         .as_ptr(),
        // );
        Sound_SetState(this, 2 as i32 as SoundState);
        if Sound_Get3D(this) {
            let mut zero: Vec3 = Vec3 {
                x: 0.0f32,
                y: 0.0f32,
                z: 0.0f32,
            };
            Sound_Set3DPos(this, &mut zero, &mut zero);
        }
    }
}
#[inline]
unsafe extern "C" fn Sound_EnsureNotFreedImpl(mut this: *mut Sound, mut func: cstr) {
    if (*this).state as i32 == 5 as i32 {
        let mut name: cstr = if (*(*this).desc)._refCount > 0 as i32 as u32 {
            (*(*this).desc).name
        } else {
            b"<SoundDesc has been freed>\0" as *const u8 as *const libc::c_char
        };
        Fatal(
            b"%s: Sound has been freed.\n  Name: %s\0" as *const u8 as *const libc::c_char,
            func,
            name,
        );
    }
}
#[inline]
unsafe extern "C" fn Sound_EnsureStateImpl(mut this: *mut Sound, mut func: cstr) {
    Sound_EnsureLoadedImpl(this, func);
    Sound_EnsureNotFreedImpl(this, func);
}
unsafe extern "C" fn Sound_SetState(mut this: *mut Sound, mut nextState: SoundState) {
    if nextState as i32 == (*this).state as i32 {
        return;
    }
    // match nextState as i32 {
    //     3 => {
    //         FMOD_CheckError(
    //             FMOD_Channel_SetPaused((*this).handle, 0 as i32),
    //             b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //                 as *const libc::c_char,
    //             71 as i32,
    //             (*::core::mem::transmute::<
    //                 &[u8; 15],
    //                 &[libc::c_char; 15],
    //             >(b"Sound_SetState\0"))
    //                 .as_ptr(),
    //         );
    //     }
    //     2 => {
    //         FMOD_CheckError(
    //             FMOD_Channel_SetPaused((*this).handle, 1 as i32),
    //             b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //                 as *const libc::c_char,
    //             75 as i32,
    //             (*::core::mem::transmute::<
    //                 &[u8; 15],
    //                 &[libc::c_char; 15],
    //             >(b"Sound_SetState\0"))
    //                 .as_ptr(),
    //         );
    //     }
    //     4 => {
    //         FMOD_CheckError(
    //             FMOD_Channel_Stop((*this).handle),
    //             b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //                 as *const libc::c_char,
    //             79 as i32,
    //             (*::core::mem::transmute::<
    //                 &[u8; 15],
    //                 &[libc::c_char; 15],
    //             >(b"Sound_SetState\0"))
    //                 .as_ptr(),
    //         );
    //     }
    //     1 | 5 => {}
    //     _ => {
    //         Fatal(
    //             b"Sound_SetState: Unhandled case: %i\0" as *const u8
    //                 as *const libc::c_char,
    //             nextState as i32,
    //         );
    //     }
    // }
    // (*this).state = nextState;
    // Audio_SoundStateChanged(this);
    // if (*this).freeOnFinish as i32 != 0
    //     && (*this).state as i32 == 4 as i32
    // {
    //     Sound_Free(this);
    // }
}
unsafe extern "C" fn Sound_Create(
    mut name: cstr,
    mut immediate: bool,
    mut isLooped: bool,
    mut is3D: bool,
) -> *mut Sound {
    let mut desc: *mut SoundDesc = SoundDesc_Load(name, immediate, isLooped, is3D);
    let mut this: *mut Sound = Audio_AllocSound();
    (*this).desc = desc;
    Sound_SetState(this, 1 as i32 as SoundState);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Sound_Load(
    mut name: cstr,
    mut isLooped: bool,
    mut is3D: bool,
) -> *mut Sound {
    let mut this: *mut Sound = Sound_Create(name, 1 as i32 != 0, isLooped, is3D);
    Sound_EnsureLoadedImpl(
        this,
        (*::core::mem::transmute::<&[u8; 11], &[libc::c_char; 11]>(b"Sound_Load\0")).as_ptr(),
    );
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Sound_LoadAsync(
    mut name: cstr,
    mut isLooped: bool,
    mut is3D: bool,
) -> *mut Sound {
    let mut this: *mut Sound = Sound_Create(name, 0 as i32 != 0, isLooped, is3D);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Sound_Clone(mut this: *mut Sound) -> *mut Sound {
    Sound_EnsureStateImpl(
        this,
        (*::core::mem::transmute::<&[u8; 12], &[libc::c_char; 12]>(b"Sound_Clone\0")).as_ptr(),
    );
    let mut clone: *mut Sound = Audio_AllocSound();
    *clone = *this;
    SoundDesc_Acquire((*this).desc);
    (*clone).handle = 0 as *mut FMOD_CHANNEL;
    (*clone).state = 0 as i32 as SoundState;
    Sound_SetState(clone, 1 as i32 as SoundState);
    return clone;
}
#[no_mangle]
pub unsafe extern "C" fn Sound_ToFile(mut this: *mut Sound, mut name: cstr) {
    Sound_EnsureStateImpl(
        this,
        (*::core::mem::transmute::<&[u8; 13], &[libc::c_char; 13]>(b"Sound_ToFile\0")).as_ptr(),
    );
    SoundDesc_ToFile((*this).desc, name);
}
#[no_mangle]
pub unsafe extern "C" fn Sound_Acquire(mut this: *mut Sound) {
    Sound_EnsureStateImpl(
        this,
        (*::core::mem::transmute::<&[u8; 14], &[libc::c_char; 14]>(b"Sound_Acquire\0")).as_ptr(),
    );
    (*(*this).desc)._refCount = ((*(*this).desc)._refCount).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn Sound_Free(mut this: *mut Sound) {
    Sound_EnsureStateImpl(
        this,
        (*::core::mem::transmute::<&[u8; 11], &[libc::c_char; 11]>(b"Sound_Free\0")).as_ptr(),
    );
    Sound_SetState(this, 4 as i32 as SoundState);
    Sound_SetState(this, 5 as i32 as SoundState);
    SoundDesc_Free((*this).desc);
}
#[no_mangle]
pub unsafe extern "C" fn Sound_Play(mut this: *mut Sound) {
    Sound_EnsureStateImpl(
        this,
        (*::core::mem::transmute::<&[u8; 11], &[libc::c_char; 11]>(b"Sound_Play\0")).as_ptr(),
    );
    Sound_SetState(this, 3 as i32 as SoundState);
}
#[no_mangle]
pub unsafe extern "C" fn Sound_Pause(mut this: *mut Sound) {
    Sound_EnsureStateImpl(
        this,
        (*::core::mem::transmute::<&[u8; 12], &[libc::c_char; 12]>(b"Sound_Pause\0")).as_ptr(),
    );
    Sound_SetState(this, 2 as i32 as SoundState);
}
#[no_mangle]
pub unsafe extern "C" fn Sound_Rewind(mut this: *mut Sound) {
    Sound_EnsureStateImpl(
        this,
        (*::core::mem::transmute::<&[u8; 13], &[libc::c_char; 13]>(b"Sound_Rewind\0")).as_ptr(),
    );
    // FMOD_CheckError(
    //     FMOD_Channel_SetPosition(
    //         (*this).handle,
    //         0 as i32 as u32,
    //         0x2 as i32 as FMOD_TIMEUNIT,
    //     ),
    //     b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     153 as i32,
    //     (*::core::mem::transmute::<&[u8; 13], &[libc::c_char; 13]>(b"Sound_Rewind\0"))
    //         .as_ptr(),
    // );
}
#[no_mangle]
pub unsafe extern "C" fn Sound_Get3D(mut this: *mut Sound) -> bool {
    Sound_EnsureStateImpl(
        this,
        (*::core::mem::transmute::<&[u8; 12], &[libc::c_char; 12]>(b"Sound_Get3D\0")).as_ptr(),
    );
    let mut mode: FMOD_MODE = 0;
    // FMOD_CheckError(
    //     FMOD_Channel_GetMode((*this).handle, &mut mode),
    //     b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     159 as i32,
    //     (*::core::mem::transmute::<&[u8; 12], &[libc::c_char; 12]>(b"Sound_Get3D\0"))
    //         .as_ptr(),
    // );
    return mode & 0x10 as i32 as u32 == 0x10 as i32 as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Sound_GetDuration(mut this: *mut Sound) -> f32 {
    Sound_EnsureStateImpl(
        this,
        (*::core::mem::transmute::<&[u8; 18], &[libc::c_char; 18]>(b"Sound_GetDuration\0"))
            .as_ptr(),
    );
    return SoundDesc_GetDuration((*this).desc);
}
#[no_mangle]
pub unsafe extern "C" fn Sound_GetLooped(mut this: *mut Sound) -> bool {
    // Sound_EnsureStateImpl(
    //     this,
    //     (*::core::mem::transmute::<&[u8; 16], &[libc::c_char; 16]>(b"Sound_GetLooped\0"))
    //         .as_ptr(),
    // );
    let mut mode: FMOD_MODE = 0;
    // FMOD_CheckError(
    //     FMOD_Channel_GetMode((*this).handle, &mut mode),
    //     b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     171 as i32,
    //     (*::core::mem::transmute::<&[u8; 16], &[libc::c_char; 16]>(b"Sound_GetLooped\0"))
    //         .as_ptr(),
    // );
    return mode & 0x2 as i32 as u32 == 0x2 as i32 as u32;
}
#[no_mangle]
pub unsafe extern "C" fn Sound_GetName(mut this: *mut Sound) -> cstr {
    Sound_EnsureNotFreedImpl(
        this,
        (*::core::mem::transmute::<&[u8; 14], &[libc::c_char; 14]>(b"Sound_GetName\0")).as_ptr(),
    );
    return SoundDesc_GetName((*this).desc);
}
#[no_mangle]
pub unsafe extern "C" fn Sound_GetPath(mut this: *mut Sound) -> cstr {
    Sound_EnsureNotFreedImpl(
        this,
        (*::core::mem::transmute::<&[u8; 14], &[libc::c_char; 14]>(b"Sound_GetPath\0")).as_ptr(),
    );
    return SoundDesc_GetPath((*this).desc);
}
#[no_mangle]
pub unsafe extern "C" fn Sound_IsFinished(mut this: *mut Sound) -> bool {
    return (*this).state as i32 == 4 as i32;
}
#[no_mangle]
pub unsafe extern "C" fn Sound_IsPlaying(mut this: *mut Sound) -> bool {
    return (*this).state as i32 == 3 as i32;
}
#[no_mangle]
pub unsafe extern "C" fn Sound_Attach3DPos(
    mut this: *mut Sound,
    mut pos: *const Vec3,
    mut vel: *const Vec3,
) {
    Sound_Set3DPos(this, pos, vel);
    (*this).autoPos = pos;
    (*this).autoVel = vel;
}
#[no_mangle]
pub unsafe extern "C" fn Sound_Set3DLevel(mut this: *mut Sound, mut level: f32) {
    Sound_EnsureStateImpl(
        this,
        (*::core::mem::transmute::<&[u8; 17], &[libc::c_char; 17]>(b"Sound_Set3DLevel\0")).as_ptr(),
    );
    // FMOD_CheckError(
    //     FMOD_Channel_Set3DLevel((*this).handle, level),
    //     b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     202 as i32,
    //     (*::core::mem::transmute::<
    //         &[u8; 17],
    //         &[libc::c_char; 17],
    //     >(b"Sound_Set3DLevel\0"))
    //         .as_ptr(),
    // );
}
#[no_mangle]
pub unsafe extern "C" fn Sound_Set3DPos(
    mut this: *mut Sound,
    mut pos: *const Vec3,
    mut vel: *const Vec3,
) {
    Sound_EnsureStateImpl(
        this,
        (*::core::mem::transmute::<&[u8; 15], &[libc::c_char; 15]>(b"Sound_Set3DPos\0")).as_ptr(),
    );
    // FMOD_CheckError(
    //     FMOD_Channel_Set3DAttributes(
    //         (*this).handle,
    //         pos as *mut FMOD_VECTOR,
    //         vel as *mut FMOD_VECTOR,
    //     ),
    //     b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     210 as i32,
    //     (*::core::mem::transmute::<&[u8; 15], &[libc::c_char; 15]>(b"Sound_Set3DPos\0"))
    //         .as_ptr(),
    // );
}
#[no_mangle]
pub unsafe extern "C" fn Sound_SetFreeOnFinish(mut this: *mut Sound, mut freeOnFinish: bool) {
    (*this).freeOnFinish = freeOnFinish;
}
#[no_mangle]
pub unsafe extern "C" fn Sound_SetPan(mut this: *mut Sound, mut pan: f32) {
    Sound_EnsureStateImpl(
        this,
        (*::core::mem::transmute::<&[u8; 13], &[libc::c_char; 13]>(b"Sound_SetPan\0")).as_ptr(),
    );
    // FMOD_CheckError(
    //     FMOD_Channel_SetPan((*this).handle, pan),
    //     b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     219 as i32,
    //     (*::core::mem::transmute::<&[u8; 13], &[libc::c_char; 13]>(b"Sound_SetPan\0"))
    //         .as_ptr(),
    // );
}
#[no_mangle]
pub unsafe extern "C" fn Sound_SetPitch(mut this: *mut Sound, mut pitch: f32) {
    Sound_EnsureStateImpl(
        this,
        (*::core::mem::transmute::<&[u8; 15], &[libc::c_char; 15]>(b"Sound_SetPitch\0")).as_ptr(),
    );
    // FMOD_CheckError(
    //     FMOD_Channel_SetPitch((*this).handle, pitch),
    //     b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     224 as i32,
    //     (*::core::mem::transmute::<&[u8; 15], &[libc::c_char; 15]>(b"Sound_SetPitch\0"))
    //         .as_ptr(),
    // );
}
#[no_mangle]
pub unsafe extern "C" fn Sound_SetPlayPos(mut this: *mut Sound, mut seconds: f32) {
    Sound_EnsureStateImpl(
        this,
        (*::core::mem::transmute::<&[u8; 17], &[libc::c_char; 17]>(b"Sound_SetPlayPos\0")).as_ptr(),
    );
    let mut ms: u32 = Round((seconds * 1000.0f32) as f64) as u32;
    // FMOD_CheckError(
    //     FMOD_Channel_SetPosition(
    //         (*this).handle,
    //         ms,
    //         0x1 as i32 as FMOD_TIMEUNIT,
    //     ),
    //     b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     232 as i32,
    //     (*::core::mem::transmute::<
    //         &[u8; 17],
    //         &[libc::c_char; 17],
    //     >(b"Sound_SetPlayPos\0"))
    //         .as_ptr(),
    // );
}
#[no_mangle]
pub unsafe extern "C" fn Sound_SetVolume(mut this: *mut Sound, mut volume: f32) {
    Sound_EnsureStateImpl(
        this,
        (*::core::mem::transmute::<&[u8; 16], &[libc::c_char; 16]>(b"Sound_SetVolume\0")).as_ptr(),
    );
    // FMOD_CheckError(
    //     FMOD_Channel_SetVolume((*this).handle, volume),
    //     b"/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     237 as i32,
    //     (*::core::mem::transmute::<&[u8; 16], &[libc::c_char; 16]>(b"Sound_SetVolume\0"))
    //         .as_ptr(),
    // );
}
#[no_mangle]
pub unsafe extern "C" fn Sound_LoadPlay(
    mut name: cstr,
    mut isLooped: bool,
    mut is3D: bool,
) -> *mut Sound {
    let mut this: *mut Sound = Sound_Load(name, isLooped, is3D);
    Sound_Play(this);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Sound_LoadPlayAttached(
    mut name: cstr,
    mut isLooped: bool,
    mut is3D: bool,
    mut pos: *const Vec3,
    mut vel: *const Vec3,
) -> *mut Sound {
    let mut this: *mut Sound = Sound_Load(name, isLooped, is3D);
    Sound_Attach3DPos(this, pos, vel);
    Sound_Play(this);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn Sound_LoadPlayFree(mut name: cstr, mut isLooped: bool, mut is3D: bool) {
    let mut this: *mut Sound = Sound_Load(name, isLooped, is3D);
    Sound_SetFreeOnFinish(this, 1 as i32 != 0);
    Sound_Play(this);
}
#[no_mangle]
pub unsafe extern "C" fn Sound_LoadPlayFreeAttached(
    mut name: cstr,
    mut isLooped: bool,
    mut is3D: bool,
    mut pos: *const Vec3,
    mut vel: *const Vec3,
) {
    let mut this: *mut Sound = Sound_Load(name, isLooped, is3D);
    Sound_Attach3DPos(this, pos, vel);
    Sound_SetFreeOnFinish(this, 1 as i32 != 0);
    Sound_Play(this);
}
#[no_mangle]
pub unsafe extern "C" fn Sound_ClonePlay(mut this: *mut Sound) -> *mut Sound {
    let mut clone: *mut Sound = Sound_Clone(this);
    Sound_Play(clone);
    return clone;
}
#[no_mangle]
pub unsafe extern "C" fn Sound_ClonePlayAttached(
    mut this: *mut Sound,
    mut pos: *const Vec3,
    mut vel: *const Vec3,
) -> *mut Sound {
    let mut clone: *mut Sound = Sound_Clone(this);
    Sound_Attach3DPos(clone, pos, vel);
    Sound_Play(clone);
    return clone;
}
#[no_mangle]
pub unsafe extern "C" fn Sound_ClonePlayFree(mut this: *mut Sound) {
    let mut clone: *mut Sound = Sound_Clone(this);
    Sound_SetFreeOnFinish(clone, 1 as i32 != 0);
    Sound_Play(clone);
}
#[no_mangle]
pub unsafe extern "C" fn Sound_ClonePlayFreeAttached(
    mut this: *mut Sound,
    mut pos: *const Vec3,
    mut vel: *const Vec3,
) {
    let mut clone: *mut Sound = Sound_Clone(this);
    Sound_Attach3DPos(clone, pos, vel);
    Sound_SetFreeOnFinish(clone, 1 as i32 != 0);
    Sound_Play(clone);
}
#[no_mangle]
pub unsafe extern "C" fn Sound_Update(mut this: *mut Sound) {
    if (*this).state as i32 == 1 as i32 {
        return;
    }
    if Sound_Get3D(this) {
        Sound_Set3DPos(this, (*this).autoPos, (*this).autoVel);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Sound_IsFreed(mut this: *mut Sound) -> bool {
    return (*this).state as i32 == 5 as i32;
}
