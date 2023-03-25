use crate::internal::Memory::*;
use crate::Audio::*;
use crate::Common::*;
use crate::File::*;
use crate::Math::Vec3;
use crate::Resource::*;
use crate::ResourceType::*;
use libc;

extern "C" {
    pub type FMOD_SOUND;
    pub type FMOD_SYSTEM;
    pub type FMOD_SOUNDGROUP;
    fn FMOD_System_CreateSound(
        system: *mut FMOD_SYSTEM,
        name_or_data: *const libc::c_char,
        mode: FMOD_MODE,
        exinfo: *mut FMOD_CREATESOUNDEXINFO,
        sound: *mut *mut FMOD_SOUND,
    ) -> FMOD_RESULT;
    fn FMOD_Sound_Release(sound: *mut FMOD_SOUND) -> FMOD_RESULT;
    fn FMOD_Sound_Lock(
        sound: *mut FMOD_SOUND,
        offset: u32,
        length: u32,
        ptr1: *mut *mut libc::c_void,
        ptr2: *mut *mut libc::c_void,
        len1: *mut u32,
        len2: *mut u32,
    ) -> FMOD_RESULT;
    fn FMOD_Sound_Unlock(
        sound: *mut FMOD_SOUND,
        ptr1: *mut libc::c_void,
        ptr2: *mut libc::c_void,
        len1: u32,
        len2: u32,
    ) -> FMOD_RESULT;
    fn FMOD_Sound_GetDefaults(
        sound: *mut FMOD_SOUND,
        frequency: *mut f32,
        priority: *mut i32,
    ) -> FMOD_RESULT;
    fn FMOD_Sound_GetLength(
        sound: *mut FMOD_SOUND,
        length: *mut u32,
        lengthtype: FMOD_TIMEUNIT,
    ) -> FMOD_RESULT;
    fn FMOD_Sound_GetFormat(
        sound: *mut FMOD_SOUND,
        type_0: *mut FMOD_SOUND_TYPE,
        format: *mut FMOD_SOUND_FORMAT,
        channels: *mut i32,
        bits: *mut i32,
    ) -> FMOD_RESULT;
    fn FMOD_Sound_GetOpenState(
        sound: *mut FMOD_SOUND,
        openstate: *mut FMOD_OPENSTATE,
        percentbuffered: *mut u32,
        starving: *mut FMOD_BOOL,
        diskbusy: *mut FMOD_BOOL,
    ) -> FMOD_RESULT;
    fn FMOD_Sound_SetUserData(sound: *mut FMOD_SOUND, userdata: *mut libc::c_void) -> FMOD_RESULT;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SoundDesc {
    pub _refCount: u32,
    pub handle: *mut FMOD_SOUND,
    pub name: *const libc::c_char,
    pub path: *const libc::c_char,
}
pub type FMOD_BOOL = i32;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct FMOD_ASYNCREADINFO {
    pub handle: *mut libc::c_void,
    pub offset: u32,
    pub sizebytes: u32,
    pub priority: i32,
    pub userdata: *mut libc::c_void,
    pub buffer: *mut libc::c_void,
    pub bytesread: u32,
    pub done: FMOD_FILE_ASYNCDONE_FUNC,
}
pub type FMOD_FILE_ASYNCDONE_FUNC =
    Option<unsafe extern "C" fn(*mut FMOD_ASYNCREADINFO, FMOD_RESULT) -> ()>;
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
pub type FMOD_CHANNELORDER = u32;
pub const FMOD_CHANNELORDER_FORCEINT: FMOD_CHANNELORDER = 65536;
pub const FMOD_CHANNELORDER_MAX: FMOD_CHANNELORDER = 6;
pub const FMOD_CHANNELORDER_ALSA: FMOD_CHANNELORDER = 5;
pub const FMOD_CHANNELORDER_ALLSTEREO: FMOD_CHANNELORDER = 4;
pub const FMOD_CHANNELORDER_ALLMONO: FMOD_CHANNELORDER = 3;
pub const FMOD_CHANNELORDER_PROTOOLS: FMOD_CHANNELORDER = 2;
pub const FMOD_CHANNELORDER_WAVEFORMAT: FMOD_CHANNELORDER = 1;
pub const FMOD_CHANNELORDER_DEFAULT: FMOD_CHANNELORDER = 0;
pub type FMOD_SOUND_TYPE = u32;
pub const FMOD_SOUND_TYPE_FORCEINT: FMOD_SOUND_TYPE = 65536;
pub const FMOD_SOUND_TYPE_MAX: FMOD_SOUND_TYPE = 25;
pub const FMOD_SOUND_TYPE_OPUS: FMOD_SOUND_TYPE = 24;
pub const FMOD_SOUND_TYPE_FADPCM: FMOD_SOUND_TYPE = 23;
pub const FMOD_SOUND_TYPE_MEDIACODEC: FMOD_SOUND_TYPE = 22;
pub const FMOD_SOUND_TYPE_MEDIA_FOUNDATION: FMOD_SOUND_TYPE = 21;
pub const FMOD_SOUND_TYPE_VORBIS: FMOD_SOUND_TYPE = 20;
pub const FMOD_SOUND_TYPE_AT9: FMOD_SOUND_TYPE = 19;
pub const FMOD_SOUND_TYPE_AUDIOQUEUE: FMOD_SOUND_TYPE = 18;
pub const FMOD_SOUND_TYPE_XMA: FMOD_SOUND_TYPE = 17;
pub const FMOD_SOUND_TYPE_XM: FMOD_SOUND_TYPE = 16;
pub const FMOD_SOUND_TYPE_WAV: FMOD_SOUND_TYPE = 15;
pub const FMOD_SOUND_TYPE_USER: FMOD_SOUND_TYPE = 14;
pub const FMOD_SOUND_TYPE_S3M: FMOD_SOUND_TYPE = 13;
pub const FMOD_SOUND_TYPE_RAW: FMOD_SOUND_TYPE = 12;
pub const FMOD_SOUND_TYPE_PLAYLIST: FMOD_SOUND_TYPE = 11;
pub const FMOD_SOUND_TYPE_OGGVORBIS: FMOD_SOUND_TYPE = 10;
pub const FMOD_SOUND_TYPE_MPEG: FMOD_SOUND_TYPE = 9;
pub const FMOD_SOUND_TYPE_MOD: FMOD_SOUND_TYPE = 8;
pub const FMOD_SOUND_TYPE_MIDI: FMOD_SOUND_TYPE = 7;
pub const FMOD_SOUND_TYPE_IT: FMOD_SOUND_TYPE = 6;
pub const FMOD_SOUND_TYPE_FSB: FMOD_SOUND_TYPE = 5;
pub const FMOD_SOUND_TYPE_FLAC: FMOD_SOUND_TYPE = 4;
pub const FMOD_SOUND_TYPE_DLS: FMOD_SOUND_TYPE = 3;
pub const FMOD_SOUND_TYPE_ASF: FMOD_SOUND_TYPE = 2;
pub const FMOD_SOUND_TYPE_AIFF: FMOD_SOUND_TYPE = 1;
pub const FMOD_SOUND_TYPE_UNKNOWN: FMOD_SOUND_TYPE = 0;
pub type FMOD_SOUND_FORMAT = u32;
pub const FMOD_SOUND_FORMAT_FORCEINT: FMOD_SOUND_FORMAT = 65536;
pub const FMOD_SOUND_FORMAT_MAX: FMOD_SOUND_FORMAT = 7;
pub const FMOD_SOUND_FORMAT_BITSTREAM: FMOD_SOUND_FORMAT = 6;
pub const FMOD_SOUND_FORMAT_PCMFLOAT: FMOD_SOUND_FORMAT = 5;
pub const FMOD_SOUND_FORMAT_PCM32: FMOD_SOUND_FORMAT = 4;
pub const FMOD_SOUND_FORMAT_PCM24: FMOD_SOUND_FORMAT = 3;
pub const FMOD_SOUND_FORMAT_PCM16: FMOD_SOUND_FORMAT = 2;
pub const FMOD_SOUND_FORMAT_PCM8: FMOD_SOUND_FORMAT = 1;
pub const FMOD_SOUND_FORMAT_NONE: FMOD_SOUND_FORMAT = 0;
pub type FMOD_OPENSTATE = u32;
pub const FMOD_OPENSTATE_FORCEINT: FMOD_OPENSTATE = 65536;
pub const FMOD_OPENSTATE_MAX: FMOD_OPENSTATE = 8;
pub const FMOD_OPENSTATE_SETPOSITION: FMOD_OPENSTATE = 7;
pub const FMOD_OPENSTATE_PLAYING: FMOD_OPENSTATE = 6;
pub const FMOD_OPENSTATE_SEEKING: FMOD_OPENSTATE = 5;
pub const FMOD_OPENSTATE_BUFFERING: FMOD_OPENSTATE = 4;
pub const FMOD_OPENSTATE_CONNECTING: FMOD_OPENSTATE = 3;
pub const FMOD_OPENSTATE_ERROR: FMOD_OPENSTATE = 2;
pub const FMOD_OPENSTATE_LOADING: FMOD_OPENSTATE = 1;
pub const FMOD_OPENSTATE_READY: FMOD_OPENSTATE = 0;
pub type FMOD_SOUND_NONBLOCK_CALLBACK =
    Option<unsafe extern "C" fn(*mut FMOD_SOUND, FMOD_RESULT) -> FMOD_RESULT>;
pub type FMOD_SOUND_PCMREAD_CALLBACK =
    Option<unsafe extern "C" fn(*mut FMOD_SOUND, *mut libc::c_void, u32) -> FMOD_RESULT>;
pub type FMOD_SOUND_PCMSETPOS_CALLBACK =
    Option<unsafe extern "C" fn(*mut FMOD_SOUND, i32, u32, FMOD_TIMEUNIT) -> FMOD_RESULT>;
pub type FMOD_FILE_OPEN_CALLBACK = Option<
    unsafe extern "C" fn(
        *const libc::c_char,
        *mut u32,
        *mut *mut libc::c_void,
        *mut libc::c_void,
    ) -> FMOD_RESULT,
>;
pub type FMOD_FILE_CLOSE_CALLBACK =
    Option<unsafe extern "C" fn(*mut libc::c_void, *mut libc::c_void) -> FMOD_RESULT>;
pub type FMOD_FILE_READ_CALLBACK = Option<
    unsafe extern "C" fn(
        *mut libc::c_void,
        *mut libc::c_void,
        u32,
        *mut u32,
        *mut libc::c_void,
    ) -> FMOD_RESULT,
>;
pub type FMOD_FILE_SEEK_CALLBACK =
    Option<unsafe extern "C" fn(*mut libc::c_void, u32, *mut libc::c_void) -> FMOD_RESULT>;
pub type FMOD_FILE_ASYNCREAD_CALLBACK =
    Option<unsafe extern "C" fn(*mut FMOD_ASYNCREADINFO, *mut libc::c_void) -> FMOD_RESULT>;
pub type FMOD_FILE_ASYNCCANCEL_CALLBACK =
    Option<unsafe extern "C" fn(*mut FMOD_ASYNCREADINFO, *mut libc::c_void) -> FMOD_RESULT>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct FMOD_GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [libc::c_uchar; 8],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct FMOD_CREATESOUNDEXINFO {
    pub cbsize: i32,
    pub length: u32,
    pub fileoffset: u32,
    pub numchannels: i32,
    pub defaultfrequency: i32,
    pub format: FMOD_SOUND_FORMAT,
    pub decodebuffersize: u32,
    pub initialsubsound: i32,
    pub numsubsounds: i32,
    pub inclusionlist: *mut i32,
    pub inclusionlistnum: i32,
    pub pcmreadcallback: FMOD_SOUND_PCMREAD_CALLBACK,
    pub pcmsetposcallback: FMOD_SOUND_PCMSETPOS_CALLBACK,
    pub nonblockcallback: FMOD_SOUND_NONBLOCK_CALLBACK,
    pub dlsname: *const libc::c_char,
    pub encryptionkey: *const libc::c_char,
    pub maxpolyphony: i32,
    pub userdata: *mut libc::c_void,
    pub suggestedsoundtype: FMOD_SOUND_TYPE,
    pub fileuseropen: FMOD_FILE_OPEN_CALLBACK,
    pub fileuserclose: FMOD_FILE_CLOSE_CALLBACK,
    pub fileuserread: FMOD_FILE_READ_CALLBACK,
    pub fileuserseek: FMOD_FILE_SEEK_CALLBACK,
    pub fileuserasyncread: FMOD_FILE_ASYNCREAD_CALLBACK,
    pub fileuserasynccancel: FMOD_FILE_ASYNCCANCEL_CALLBACK,
    pub fileuserdata: *mut libc::c_void,
    pub filebuffersize: i32,
    pub channelorder: FMOD_CHANNELORDER,
    pub initialsoundgroup: *mut FMOD_SOUNDGROUP,
    pub initialseekposition: u32,
    pub initialseekpostype: FMOD_TIMEUNIT,
    pub ignoresetfilesystem: i32,
    pub audioqueuepolicy: u32,
    pub minmidigranularity: u32,
    pub nonblockthreadid: i32,
    pub fsbguid: *mut FMOD_GUID,
}

#[inline]
unsafe extern "C" fn FMOD_CheckError(
    result: FMOD_RESULT,
    file: *const libc::c_char,
    line: i32,
    func: *const libc::c_char,
) {
    if result != FMOD_OK as i32 as u32 {
        Fatal(
            c_str!("%s: %s\n%s\n  [%s @ Line %d]"),
            func,
            FMODError_ToString(result),
            FMOD_ErrorString(result),
            file,
            line,
        );
    }
}

#[inline]
unsafe extern "C" fn FMODError_ToString(this: FMOD_RESULT) -> *const libc::c_char {
    match this {
        0 => return c_str!("FMOD_OK"),
        1 => return c_str!("FMOD_ERR_BADCOMMAND"),
        2 => return c_str!("FMOD_ERR_CHANNEL_ALLOC"),
        3 => return c_str!("FMOD_ERR_CHANNEL_STOLEN"),
        4 => return c_str!("FMOD_ERR_DMA"),
        5 => return c_str!("FMOD_ERR_DSP_CONNECTION"),
        6 => return c_str!("FMOD_ERR_DSP_DONTPROCESS"),
        7 => return c_str!("FMOD_ERR_DSP_FORMAT"),
        8 => return c_str!("FMOD_ERR_DSP_INUSE"),
        9 => return c_str!("FMOD_ERR_DSP_NOTFOUND"),
        10 => return c_str!("FMOD_ERR_DSP_RESERVED"),
        11 => return c_str!("FMOD_ERR_DSP_SILENCE"),
        12 => return c_str!("FMOD_ERR_DSP_TYPE"),
        13 => return c_str!("FMOD_ERR_FILE_BAD"),
        14 => return c_str!("FMOD_ERR_FILE_COULDNOTSEEK"),
        15 => return c_str!("FMOD_ERR_FILE_DISKEJECTED"),
        16 => return c_str!("FMOD_ERR_FILE_EOF"),
        17 => return c_str!("FMOD_ERR_FILE_ENDOFDATA"),
        18 => return c_str!("FMOD_ERR_FILE_NOTFOUND"),
        19 => return c_str!("FMOD_ERR_FORMAT"),
        20 => return c_str!("FMOD_ERR_HEADER_MISMATCH"),
        21 => return c_str!("FMOD_ERR_HTTP"),
        22 => return c_str!("FMOD_ERR_HTTP_ACCESS"),
        23 => return c_str!("FMOD_ERR_HTTP_PROXY_AUTH"),
        24 => return c_str!("FMOD_ERR_HTTP_SERVER_ERROR"),
        25 => return c_str!("FMOD_ERR_HTTP_TIMEOUT"),
        26 => return c_str!("FMOD_ERR_INITIALIZATION"),
        27 => return c_str!("FMOD_ERR_INITIALIZED"),
        28 => return c_str!("FMOD_ERR_INTERNAL"),
        29 => return c_str!("FMOD_ERR_INVALID_FLOAT"),
        30 => return c_str!("FMOD_ERR_INVALID_HANDLE"),
        31 => return c_str!("FMOD_ERR_INVALID_PARAM"),
        32 => return c_str!("FMOD_ERR_INVALID_POSITION"),
        33 => return c_str!("FMOD_ERR_INVALID_SPEAKER"),
        34 => return c_str!("FMOD_ERR_INVALID_SYNCPOINT"),
        35 => return c_str!("FMOD_ERR_INVALID_THREAD"),
        36 => return c_str!("FMOD_ERR_INVALID_VECTOR"),
        37 => return c_str!("FMOD_ERR_MAXAUDIBLE"),
        38 => return c_str!("FMOD_ERR_MEMORY"),
        39 => return c_str!("FMOD_ERR_MEMORY_CANTPOINT"),
        40 => return c_str!("FMOD_ERR_NEEDS3D"),
        41 => return c_str!("FMOD_ERR_NEEDSHARDWARE"),
        42 => return c_str!("FMOD_ERR_NET_CONNECT"),
        43 => return c_str!("FMOD_ERR_NET_SOCKET_ERROR"),
        44 => return c_str!("FMOD_ERR_NET_URL"),
        45 => return c_str!("FMOD_ERR_NET_WOULD_BLOCK"),
        46 => return c_str!("FMOD_ERR_NOTREADY"),
        47 => return c_str!("FMOD_ERR_OUTPUT_ALLOCATED"),
        48 => {
            return c_str!("FMOD_ERR_OUTPUT_CREATEBUFFER");
        }
        49 => return c_str!("FMOD_ERR_OUTPUT_DRIVERCALL"),
        50 => return c_str!("FMOD_ERR_OUTPUT_FORMAT"),
        51 => return c_str!("FMOD_ERR_OUTPUT_INIT"),
        52 => return c_str!("FMOD_ERR_OUTPUT_NODRIVERS"),
        53 => return c_str!("FMOD_ERR_PLUGIN"),
        54 => return c_str!("FMOD_ERR_PLUGIN_MISSING"),
        55 => return c_str!("FMOD_ERR_PLUGIN_RESOURCE"),
        56 => return c_str!("FMOD_ERR_PLUGIN_VERSION"),
        57 => return c_str!("FMOD_ERR_RECORD"),
        58 => {
            return c_str!("FMOD_ERR_REVERB_CHANNELGROUP");
        }
        59 => return c_str!("FMOD_ERR_REVERB_INSTANCE"),
        60 => return c_str!("FMOD_ERR_SUBSOUNDS"),
        61 => return c_str!("FMOD_ERR_SUBSOUND_ALLOCATED"),
        62 => return c_str!("FMOD_ERR_SUBSOUND_CANTMOVE"),
        63 => return c_str!("FMOD_ERR_TAGNOTFOUND"),
        64 => return c_str!("FMOD_ERR_TOOMANYCHANNELS"),
        65 => return c_str!("FMOD_ERR_TRUNCATED"),
        66 => return c_str!("FMOD_ERR_UNIMPLEMENTED"),
        67 => return c_str!("FMOD_ERR_UNINITIALIZED"),
        68 => return c_str!("FMOD_ERR_UNSUPPORTED"),
        69 => return c_str!("FMOD_ERR_VERSION"),
        70 => {
            return c_str!("FMOD_ERR_EVENT_ALREADY_LOADED");
        }
        71 => {
            return c_str!("FMOD_ERR_EVENT_LIVEUPDATE_BUSY");
        }
        72 => {
            return c_str!("FMOD_ERR_EVENT_LIVEUPDATE_MISMATCH");
        }
        73 => {
            return c_str!("FMOD_ERR_EVENT_LIVEUPDATE_TIMEOUT");
        }
        74 => return c_str!("FMOD_ERR_EVENT_NOTFOUND"),
        75 => {
            return c_str!("FMOD_ERR_STUDIO_UNINITIALIZED");
        }
        76 => return c_str!("FMOD_ERR_STUDIO_NOT_LOADED"),
        77 => return c_str!("FMOD_ERR_INVALID_STRING"),
        78 => return c_str!("FMOD_ERR_ALREADY_LOCKED"),
        79 => return c_str!("FMOD_ERR_NOT_LOCKED"),
        80 => {
            return c_str!("FMOD_ERR_RECORD_DISCONNECTED");
        }
        81 => return c_str!("FMOD_ERR_TOOMANYSAMPLES"),
        65536 => return c_str!("FMOD_RESULT_FORCEINT"),
        _ => {}
    }
    c_str!("Unknown Error")
}

unsafe extern "C" fn FMOD_ErrorString(errcode: FMOD_RESULT) -> *const libc::c_char {
    match errcode {
        0 => c_str!("No errors."),
        1 => {
            c_str!("Tried to call a function on a data type that does not allow this type of functionality (ie calling Sound::lock on a streaming sound).")
        }
        2 => {
            c_str!("Error trying to allocate a channel.")
        }
        3 => {
            c_str!("The specified channel has been reused to play another sound.")
        }
        4 => {
            c_str!("DMA Failure.  See debug output for more information.")
        }
        5 => {
            c_str!("DSP connection error.  Connection possibly caused a cyclic dependency or connected dsps with incompatible buffer counts.")
        }
        6 => {
            c_str!("DSP return code from a DSP process query callback.  Tells mixer not to call the process callback and therefore not consume CPU.  Use this to optimize the DSP graph.")
        }
        7 => {
            c_str!("DSP Format error.  A DSP unit may have attempted to connect to this network with the wrong format, or a matrix may have been set with the wrong size if the target unit has a specified channel map.")
        }
        8 => {
            c_str!("DSP is already in the mixer's DSP network. It must be removed before being reinserted or released.")
        }
        9 => {
            c_str!("DSP connection error.  Couldn't find the DSP unit specified.")
        }
        10 => {
            c_str!("DSP operation error.  Cannot perform operation on this DSP as it is reserved by the system.")
        }
        11 => {
            c_str!("DSP return code from a DSP process query callback.  Tells mixer silence would be produced from read, so go idle and not consume CPU.  Use this to optimize the DSP graph.")
        }
        12 => {
            c_str!("DSP operation cannot be performed on a DSP of this type.")
        }
        13 => c_str!("Error loading file."),
        14 => {
            c_str!("Couldn't perform seek operation.  This is a limitation of the medium (ie netstreams) or the file format.")
        }
        15 => {
            c_str!("Media was ejected while reading.")
        }
        16 => {
            c_str!("End of file unexpectedly reached while trying to read essential data (truncated?).")
        }
        17 => {
            c_str!("End of current chunk reached while trying to read data.")
        }
        18 => c_str!("File not found."),
        19 => {
            c_str!("Unsupported file or audio format.")
        }
        20 => {
            c_str!("There is a version mismatch between the FMOD header and either the FMOD Studio library or the FMOD Low Level library.")
        }
        21 => {
            c_str!(
                "A HTTP error occurred. This is a catch-all for HTTP errors not listed elsewhere."
            )
        }
        22 => {
            c_str!("The specified resource requires authentication or is forbidden.")
        }
        23 => {
            c_str!("Proxy authentication is required to access the specified resource.")
        }
        24 => {
            c_str!("A HTTP server error occurred.")
        }
        25 => c_str!("The HTTP request timed out."),
        26 => {
            c_str!("FMOD was not initialized correctly to support this function.")
        }
        27 => {
            c_str!("Cannot call this command after System::init.")
        }
        28 => {
            c_str!("An error occurred that wasn't supposed to.  Contact support.")
        }
        29 => {
            c_str!("Value passed in was a NaN, Inf or denormalized float.")
        }
        30 => {
            c_str!("An invalid object handle was used.")
        }
        31 => {
            c_str!("An invalid parameter was passed to this function.")
        }
        32 => {
            c_str!("An invalid seek position was passed to this function.")
        }
        33 => {
            c_str!(
                "An invalid speaker was passed to this function based on the current speaker mode."
            )
        }
        34 => {
            c_str!("The syncpoint did not come from this sound handle.")
        }
        35 => {
            c_str!("Tried to call a function on a thread that is not supported.")
        }
        36 => {
            c_str!("The vectors passed in are not unit length, or perpendicular.")
        }
        37 => {
            c_str!("Reached maximum audible playback count for this sound's soundgroup.")
        }
        38 => {
            c_str!("Not enough memory or resources.")
        }
        39 => {
            c_str!("Can't use FMOD_OPENMEMORY_POINT on non PCM source data, or non mp3/xma/adpcm data if FMOD_CREATECOMPRESSEDSAMPLE was used.")
        }
        40 => {
            c_str!("Tried to call a command on a 2d sound when the command was meant for 3d sound.")
        }
        41 => {
            c_str!("Tried to use a feature that requires hardware support.")
        }
        42 => {
            c_str!("Couldn't connect to the specified host.")
        }
        43 => {
            c_str!("A socket error occurred.  This is a catch-all for socket-related errors not listed elsewhere.")
        }
        44 => {
            c_str!("The specified URL couldn't be resolved.")
        }
        45 => {
            c_str!("Operation on a non-blocking socket could not complete immediately.")
        }
        46 => {
            c_str!("Operation could not be performed because specified sound/DSP connection is not ready.")
        }
        47 => {
            c_str!("Error initializing output device, but more specifically, the output device is already in use and cannot be reused.")
        }
        48 => {
            c_str!("Error creating hardware sound buffer.")
        }
        49 => {
            c_str!("A call to a standard soundcard driver failed, which could possibly mean a bug in the driver or resources were missing or exhausted.")
        }
        50 => {
            c_str!("Soundcard does not support the specified format.")
        }
        51 => {
            c_str!("Error initializing output device.")
        }
        52 => {
            c_str!("The output device has no drivers installed.  If pre-init, FMOD_OUTPUT_NOSOUND is selected as the output mode.  If post-init, the function just fails.")
        }
        53 => {
            c_str!("An unspecified error has been returned from a plugin.")
        }
        54 => {
            c_str!("A requested output, dsp unit type or codec was not available.")
        }
        55 => {
            c_str!("A resource that the plugin requires cannot be allocated or found. (ie the DLS file for MIDI playback)")
        }
        56 => {
            c_str!("A plugin was built with an unsupported SDK version.")
        }
        57 => {
            c_str!("An error occurred trying to initialize the recording device.")
        }
        58 => {
            c_str!("Reverb properties cannot be set on this channel because a parent channelgroup owns the reverb connection.")
        }
        59 => {
            c_str!("Specified instance in FMOD_REVERB_PROPERTIES couldn't be set. Most likely because it is an invalid instance number or the reverb doesn't exist.")
        }
        60 => {
            c_str!("The error occurred because the sound referenced contains subsounds when it shouldn't have, or it doesn't contain subsounds when it should have.  The operation may also not be able to be performed on a parent sound.")
        }
        61 => {
            c_str!("This subsound is already being used by another sound, you cannot have more than one parent to a sound.  Null out the other parent's entry first.")
        }
        62 => {
            c_str!("Shared subsounds cannot be replaced or moved from their parent stream, such as when the parent stream is an FSB file.")
        }
        63 => {
            c_str!("The specified tag could not be found or there are no tags.")
        }
        64 => {
            c_str!("The sound created exceeds the allowable input channel count.  This can be increased using the 'maxinputchannels' parameter in System::setSoftwareFormat.")
        }
        65 => {
            c_str!("The retrieved string is too long to fit in the supplied buffer and has been truncated.")
        }
        66 => {
            c_str!("Something in FMOD hasn't been implemented when it should be! contact support!")
        }
        67 => {
            c_str!("This command failed because System::init or System::setDriver was not called.")
        }
        68 => {
            c_str!("A command issued was not supported by this object.  Possibly a plugin without certain callbacks specified.")
        }
        69 => {
            c_str!("The version number of this file format is not supported.")
        }
        70 => {
            c_str!("The specified bank has already been loaded.")
        }
        71 => {
            c_str!("The live update connection failed due to the game already being connected.")
        }
        72 => {
            c_str!("The live update connection failed due to the game data being out of sync with the tool.")
        }
        73 => {
            c_str!("The live update connection timed out.")
        }
        74 => {
            c_str!("The requested event, parameter, bus or vca could not be found.")
        }
        75 => {
            c_str!("The Studio::System object is not yet initialized.")
        }
        76 => {
            c_str!("The specified resource is not loaded, so it can't be unloaded.")
        }
        77 => {
            c_str!("An invalid string was passed to this function.")
        }
        78 => {
            c_str!("The specified resource is already locked.")
        }
        79 => {
            c_str!("The specified resource is not locked, so it can't be unlocked.")
        }
        80 => {
            c_str!("The specified recording driver has been disconnected.")
        }
        81 => {
            c_str!("The length provided exceeds the allowable limit.")
        }
        _ => c_str!("Unknown error."),
    }
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_FinishLoad(this: *mut SoundDesc, func: *const libc::c_char) {
    let mut warned: bool = false;
    let openState: FMOD_OPENSTATE = FMOD_OPENSTATE_READY;
    loop {
        // FMOD_CheckError(
        //     FMOD_Sound_GetOpenState(
        //         (*this).handle,
        //         &mut openState,
        //         std::ptr::null_mut(),
        //         std::ptr::null_mut(),
        //         std::ptr::null_mut(),
        //     ),
        //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
        //         as *const libc::c_char,
        //     16 as i32,
        //     (*std::mem::transmute::<
        //         &[u8; 21],
        //         &[libc::c_char; 21],
        //     >(c_str!("SoundDesc_FinishLoad\0"))
        //         .as_ptr(),
        // );
        if openState == FMOD_OPENSTATE_ERROR as i32 as u32 {
            Fatal(
                c_str!("%s: Background file load has failed.\n  Path: %s"),
                func,
                (*this).path,
            );
        }
        if openState == FMOD_OPENSTATE_READY as i32 as u32
            || openState == FMOD_OPENSTATE_PLAYING as i32 as u32
        {
            break;
        }
        if !warned {
            warned = true;
            Warn(
                c_str!("%s: Background file load hasn't finished. Blocking the main thread.\n  Path: %s"),
                func,
                (*this).path,
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
    let mapKey: *const libc::c_char = StrAdd(
        if isLooped as i32 != 0 {
            c_str!("LOOPED:")
        } else {
            c_str!("UNLOOPED:")
        },
        name,
    );
    let this: *mut SoundDesc = Audio_AllocSoundDesc(mapKey);
    StrFree(mapKey);
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
        // FMOD_CheckError(
        //     FMOD_System_CreateSound(
        //         Audio_GetHandle() as *mut FMOD_SYSTEM,
        //         path,
        //         mode,
        //         std::ptr::null_mut(),
        //         &mut (*this).handle,
        //     ),
        //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
        //         as *const libc::c_char,
        //     48 as i32,
        //     (*std::mem::transmute::<
        //         &[u8; 15],
        //         &[libc::c_char; 15],
        //     >(c_str!("SoundDesc_Load\0"))
        //         .as_ptr(),
        // );
        // FMOD_CheckError(
        //     FMOD_Sound_SetUserData((*this).handle, this as *mut _),
        //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
        //         as *const libc::c_char,
        //     49 as i32,
        //     (*std::mem::transmute::<
        //         &[u8; 15],
        //         &[libc::c_char; 15],
        //     >(c_str!("SoundDesc_Load\0"))
        //         .as_ptr(),
        // );
        (*this).name = StrDup(name);
        (*this).path = StrDup(path);
        (*this)._refCount = 1;
    } else {
        (*this)._refCount = ((*this)._refCount).wrapping_add(1);
        if immediate {
            SoundDesc_FinishLoad(this, c_str!("SoundDesc_Load"));
        }
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_Acquire(this: *mut SoundDesc) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_Free(this: *mut SoundDesc) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0
    } {
        let name: *const libc::c_char = (*this).name;
        let path: *const libc::c_char = (*this).path;
        // FMOD_CheckError(
        //     FMOD_Sound_Release((*this).handle),
        //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c"),
        //     72 as i32,
        //     c_str!("SoundDesc_Free"),
        // );
        Audio_DeallocSoundDesc(this);
        StrFree(name);
        StrFree(path);
        MemZero(this as *mut _, std::mem::size_of::<SoundDesc>());
    }
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_GetDuration(this: *mut SoundDesc) -> f32 {
    SoundDesc_FinishLoad(this, c_str!("SoundDesc_GetDuration"));
    let duration: u32 = 0;
    // FMOD_CheckError(
    //     FMOD_Sound_GetLength(
    //         (*this).handle,
    //         &mut duration,
    //         0x1 as i32 as FMOD_TIMEUNIT,
    //     ),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
    //         as *const libc::c_char,
    //     85 as i32,
    //     (*std::mem::transmute::<
    //         &[u8; 22],
    //         &[libc::c_char; 22],
    //     >(c_str!("SoundDesc_GetDuration\0"))
    //         .as_ptr(),
    // );
    duration as f32 / 1000.0f32
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_GetName(this: *mut SoundDesc) -> *const libc::c_char {
    (*this).name
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_GetPath(this: *mut SoundDesc) -> *const libc::c_char {
    (*this).path
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_ToFile(this: *mut SoundDesc, name: *const libc::c_char) {
    SoundDesc_FinishLoad(this, c_str!("SoundDesc_ToFile"));
    let length: u32 = 0;
    let channels: i32 = 0;
    let bitsPerSample: i32 = 0;
    // FMOD_CheckError(
    //     FMOD_Sound_GetLength(
    //         (*this).handle,
    //         &mut length,
    //         0x8 as i32 as FMOD_TIMEUNIT,
    //     ),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
    //         as *const libc::c_char,
    //     128 as i32,
    //     (*std::mem::transmute::<
    //         &[u8; 17],
    //         &[libc::c_char; 17],
    //     >(c_str!("SoundDesc_ToFile\0"))
    //         .as_ptr(),
    // );
    // FMOD_CheckError(
    //     FMOD_Sound_GetFormat(
    //         (*this).handle,
    //         std::ptr::null_mut(),
    //         std::ptr::null_mut(),
    //         &mut channels,
    //         &mut bitsPerSample,
    //     ),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
    //         as *const libc::c_char,
    //     129 as i32,
    //     (*std::mem::transmute::<
    //         &[u8; 17],
    //         &[libc::c_char; 17],
    //     >(c_str!("SoundDesc_ToFile\0"))
    //         .as_ptr(),
    // );
    let bytesPerSample: i32 = bitsPerSample / 8;
    let sampleRate: f32 = 0.;
    // FMOD_Sound_GetDefaults((*this).handle, &mut sampleRate, std::ptr::null_mut());
    let ptr1: *mut libc::c_void = std::ptr::null_mut();
    let _len1: u32 = 0;
    let _ptr2: *mut libc::c_void = std::ptr::null_mut();
    let _len2: u32 = 0;
    // FMOD_CheckError(
    //     FMOD_Sound_Lock(
    //         (*this).handle,
    //         0 as i32 as u32,
    //         length,
    //         &mut ptr1,
    //         &mut ptr2,
    //         &mut len1,
    //         &mut len2,
    //     ),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
    //         as *const libc::c_char,
    //     137 as i32,
    //     (*std::mem::transmute::<
    //         &[u8; 17],
    //         &[libc::c_char; 17],
    //     >(c_str!("SoundDesc_ToFile\0"))
    //         .as_ptr(),
    // );
    let file: *mut File = File_Create(name);
    if file.is_null() {
        Fatal(
            c_str!("SoundDesc_ToFile: Failed to create file.\nPath: %s"),
            name,
        );
    }
    File_Write(file, c_str!("RIFF") as *const _, 4);
    File_WriteI32(file, (36_u32).wrapping_add(length) as i32);
    File_Write(file, c_str!("WAVE") as *const _, 4);
    File_Write(file, c_str!("fmt ") as *const _, 4);
    File_WriteI32(file, 16);
    File_WriteI16(file, 1);
    File_WriteI16(file, channels as i16);
    File_WriteI32(file, sampleRate as i32);
    File_WriteI32(
        file,
        ((bytesPerSample * channels) as f32 * sampleRate) as i32,
    );
    File_WriteI16(file, (bytesPerSample * channels) as i16);
    File_WriteI16(file, bitsPerSample as i16);
    File_Write(file, c_str!("data") as *const _, 4);
    File_WriteI32(file, length as i32);
    File_Write(file, ptr1, length);
    File_Close(file);
    // FMOD_CheckError(
    //     FMOD_Sound_Unlock((*this).handle, ptr1, ptr2, len1, len2),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
    //         as *const libc::c_char,
    //     163 as i32,
    //     (*std::mem::transmute::<
    //         &[u8; 17],
    //         &[libc::c_char; 17],
    //     >(c_str!("SoundDesc_ToFile\0"))
    //         .as_ptr(),
    // );
}
