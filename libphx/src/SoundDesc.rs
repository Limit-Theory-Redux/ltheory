use crate::internal::Memory::*;
use crate::Audio::*;
use crate::File::*;
use crate::Resource::*;
use crate::ResourceType::*;
use crate::Math::Vec3;
use libc;

extern "C" {
    pub type FMOD_SOUND;
    pub type FMOD_SYSTEM;
    pub type FMOD_SOUNDGROUP;
    fn Fatal(_: *const libc::c_char, _: ...);
    fn Warn(_: *const libc::c_char, _: ...);
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
pub type ResourceType = i32;
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
    mut result: FMOD_RESULT,
    mut file: *const libc::c_char,
    mut line: i32,
    mut func: *const libc::c_char,
) {
    if result != FMOD_OK as i32 as u32 {
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
unsafe extern "C" fn FMODError_ToString(mut this: FMOD_RESULT) -> *const libc::c_char {
    match this {
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
    b"Unknown Error\0" as *const u8 as *const libc::c_char
}

unsafe extern "C" fn FMOD_ErrorString(mut errcode: FMOD_RESULT) -> *const libc::c_char {
    match errcode {
        0 => b"No errors.\0" as *const u8 as *const libc::c_char,
        1 => {
            b"Tried to call a function on a data type that does not allow this type of functionality (ie calling Sound::lock on a streaming sound).\0"
                as *const u8 as *const libc::c_char
        }
        2 => {
            b"Error trying to allocate a channel.\0" as *const u8 as *const libc::c_char
        }
        3 => {
            b"The specified channel has been reused to play another sound.\0" as *const u8
                as *const libc::c_char
        }
        4 => {
            b"DMA Failure.  See debug output for more information.\0" as *const u8
                as *const libc::c_char
        }
        5 => {
            b"DSP connection error.  Connection possibly caused a cyclic dependency or connected dsps with incompatible buffer counts.\0"
                as *const u8 as *const libc::c_char
        }
        6 => {
            b"DSP return code from a DSP process query callback.  Tells mixer not to call the process callback and therefore not consume CPU.  Use this to optimize the DSP graph.\0"
                as *const u8 as *const libc::c_char
        }
        7 => {
            b"DSP Format error.  A DSP unit may have attempted to connect to this network with the wrong format, or a matrix may have been set with the wrong size if the target unit has a specified channel map.\0"
                as *const u8 as *const libc::c_char
        }
        8 => {
            b"DSP is already in the mixer's DSP network. It must be removed before being reinserted or released.\0"
                as *const u8 as *const libc::c_char
        }
        9 => {
            b"DSP connection error.  Couldn't find the DSP unit specified.\0" as *const u8
                as *const libc::c_char
        }
        10 => {
            b"DSP operation error.  Cannot perform operation on this DSP as it is reserved by the system.\0"
                as *const u8 as *const libc::c_char
        }
        11 => {
            b"DSP return code from a DSP process query callback.  Tells mixer silence would be produced from read, so go idle and not consume CPU.  Use this to optimize the DSP graph.\0"
                as *const u8 as *const libc::c_char
        }
        12 => {
            b"DSP operation cannot be performed on a DSP of this type.\0" as *const u8
                as *const libc::c_char
        }
        13 => b"Error loading file.\0" as *const u8 as *const libc::c_char,
        14 => {
            b"Couldn't perform seek operation.  This is a limitation of the medium (ie netstreams) or the file format.\0"
                as *const u8 as *const libc::c_char
        }
        15 => {
            b"Media was ejected while reading.\0" as *const u8 as *const libc::c_char
        }
        16 => {
            b"End of file unexpectedly reached while trying to read essential data (truncated?).\0"
                as *const u8 as *const libc::c_char
        }
        17 => {
            b"End of current chunk reached while trying to read data.\0" as *const u8
                as *const libc::c_char
        }
        18 => b"File not found.\0" as *const u8 as *const libc::c_char,
        19 => {
            b"Unsupported file or audio format.\0" as *const u8 as *const libc::c_char
        }
        20 => {
            b"There is a version mismatch between the FMOD header and either the FMOD Studio library or the FMOD Low Level library.\0"
                as *const u8 as *const libc::c_char
        }
        21 => {
            b"A HTTP error occurred. This is a catch-all for HTTP errors not listed elsewhere.\0"
                as *const u8 as *const libc::c_char
        }
        22 => {
            b"The specified resource requires authentication or is forbidden.\0" as *const u8
                as *const libc::c_char
        }
        23 => {
            b"Proxy authentication is required to access the specified resource.\0"
                as *const u8 as *const libc::c_char
        }
        24 => {
            b"A HTTP server error occurred.\0" as *const u8 as *const libc::c_char
        }
        25 => b"The HTTP request timed out.\0" as *const u8 as *const libc::c_char,
        26 => {
            b"FMOD was not initialized correctly to support this function.\0" as *const u8
                as *const libc::c_char
        }
        27 => {
            b"Cannot call this command after System::init.\0" as *const u8
                as *const libc::c_char
        }
        28 => {
            b"An error occurred that wasn't supposed to.  Contact support.\0" as *const u8
                as *const libc::c_char
        }
        29 => {
            b"Value passed in was a NaN, Inf or denormalized float.\0" as *const u8
                as *const libc::c_char
        }
        30 => {
            b"An invalid object handle was used.\0" as *const u8 as *const libc::c_char
        }
        31 => {
            b"An invalid parameter was passed to this function.\0" as *const u8
                as *const libc::c_char
        }
        32 => {
            b"An invalid seek position was passed to this function.\0" as *const u8
                as *const libc::c_char
        }
        33 => {
            b"An invalid speaker was passed to this function based on the current speaker mode.\0"
                as *const u8 as *const libc::c_char
        }
        34 => {
            b"The syncpoint did not come from this sound handle.\0" as *const u8
                as *const libc::c_char
        }
        35 => {
            b"Tried to call a function on a thread that is not supported.\0" as *const u8
                as *const libc::c_char
        }
        36 => {
            b"The vectors passed in are not unit length, or perpendicular.\0" as *const u8
                as *const libc::c_char
        }
        37 => {
            b"Reached maximum audible playback count for this sound's soundgroup.\0"
                as *const u8 as *const libc::c_char
        }
        38 => {
            b"Not enough memory or resources.\0" as *const u8 as *const libc::c_char
        }
        39 => {
            b"Can't use FMOD_OPENMEMORY_POINT on non PCM source data, or non mp3/xma/adpcm data if FMOD_CREATECOMPRESSEDSAMPLE was used.\0"
                as *const u8 as *const libc::c_char
        }
        40 => {
            b"Tried to call a command on a 2d sound when the command was meant for 3d sound.\0"
                as *const u8 as *const libc::c_char
        }
        41 => {
            b"Tried to use a feature that requires hardware support.\0" as *const u8
                as *const libc::c_char
        }
        42 => {
            b"Couldn't connect to the specified host.\0" as *const u8
                as *const libc::c_char
        }
        43 => {
            b"A socket error occurred.  This is a catch-all for socket-related errors not listed elsewhere.\0"
                as *const u8 as *const libc::c_char
        }
        44 => {
            b"The specified URL couldn't be resolved.\0" as *const u8
                as *const libc::c_char
        }
        45 => {
            b"Operation on a non-blocking socket could not complete immediately.\0"
                as *const u8 as *const libc::c_char
        }
        46 => {
            b"Operation could not be performed because specified sound/DSP connection is not ready.\0"
                as *const u8 as *const libc::c_char
        }
        47 => {
            b"Error initializing output device, but more specifically, the output device is already in use and cannot be reused.\0"
                as *const u8 as *const libc::c_char
        }
        48 => {
            b"Error creating hardware sound buffer.\0" as *const u8 as *const libc::c_char
        }
        49 => {
            b"A call to a standard soundcard driver failed, which could possibly mean a bug in the driver or resources were missing or exhausted.\0"
                as *const u8 as *const libc::c_char
        }
        50 => {
            b"Soundcard does not support the specified format.\0" as *const u8
                as *const libc::c_char
        }
        51 => {
            b"Error initializing output device.\0" as *const u8 as *const libc::c_char
        }
        52 => {
            b"The output device has no drivers installed.  If pre-init, FMOD_OUTPUT_NOSOUND is selected as the output mode.  If post-init, the function just fails.\0"
                as *const u8 as *const libc::c_char
        }
        53 => {
            b"An unspecified error has been returned from a plugin.\0" as *const u8
                as *const libc::c_char
        }
        54 => {
            b"A requested output, dsp unit type or codec was not available.\0" as *const u8
                as *const libc::c_char
        }
        55 => {
            b"A resource that the plugin requires cannot be allocated or found. (ie the DLS file for MIDI playback)\0"
                as *const u8 as *const libc::c_char
        }
        56 => {
            b"A plugin was built with an unsupported SDK version.\0" as *const u8
                as *const libc::c_char
        }
        57 => {
            b"An error occurred trying to initialize the recording device.\0" as *const u8
                as *const libc::c_char
        }
        58 => {
            b"Reverb properties cannot be set on this channel because a parent channelgroup owns the reverb connection.\0"
                as *const u8 as *const libc::c_char
        }
        59 => {
            b"Specified instance in FMOD_REVERB_PROPERTIES couldn't be set. Most likely because it is an invalid instance number or the reverb doesn't exist.\0"
                as *const u8 as *const libc::c_char
        }
        60 => {
            b"The error occurred because the sound referenced contains subsounds when it shouldn't have, or it doesn't contain subsounds when it should have.  The operation may also not be able to be performed on a parent sound.\0"
                as *const u8 as *const libc::c_char
        }
        61 => {
            b"This subsound is already being used by another sound, you cannot have more than one parent to a sound.  Null out the other parent's entry first.\0"
                as *const u8 as *const libc::c_char
        }
        62 => {
            b"Shared subsounds cannot be replaced or moved from their parent stream, such as when the parent stream is an FSB file.\0"
                as *const u8 as *const libc::c_char
        }
        63 => {
            b"The specified tag could not be found or there are no tags.\0" as *const u8
                as *const libc::c_char
        }
        64 => {
            b"The sound created exceeds the allowable input channel count.  This can be increased using the 'maxinputchannels' parameter in System::setSoftwareFormat.\0"
                as *const u8 as *const libc::c_char
        }
        65 => {
            b"The retrieved string is too long to fit in the supplied buffer and has been truncated.\0"
                as *const u8 as *const libc::c_char
        }
        66 => {
            b"Something in FMOD hasn't been implemented when it should be! contact support!\0"
                as *const u8 as *const libc::c_char
        }
        67 => {
            b"This command failed because System::init or System::setDriver was not called.\0"
                as *const u8 as *const libc::c_char
        }
        68 => {
            b"A command issued was not supported by this object.  Possibly a plugin without certain callbacks specified.\0"
                as *const u8 as *const libc::c_char
        }
        69 => {
            b"The version number of this file format is not supported.\0" as *const u8
                as *const libc::c_char
        }
        70 => {
            b"The specified bank has already been loaded.\0" as *const u8
                as *const libc::c_char
        }
        71 => {
            b"The live update connection failed due to the game already being connected.\0"
                as *const u8 as *const libc::c_char
        }
        72 => {
            b"The live update connection failed due to the game data being out of sync with the tool.\0"
                as *const u8 as *const libc::c_char
        }
        73 => {
            b"The live update connection timed out.\0" as *const u8 as *const libc::c_char
        }
        74 => {
            b"The requested event, parameter, bus or vca could not be found.\0" as *const u8
                as *const libc::c_char
        }
        75 => {
            b"The Studio::System object is not yet initialized.\0" as *const u8
                as *const libc::c_char
        }
        76 => {
            b"The specified resource is not loaded, so it can't be unloaded.\0" as *const u8
                as *const libc::c_char
        }
        77 => {
            b"An invalid string was passed to this function.\0" as *const u8
                as *const libc::c_char
        }
        78 => {
            b"The specified resource is already locked.\0" as *const u8
                as *const libc::c_char
        }
        79 => {
            b"The specified resource is not locked, so it can't be unlocked.\0" as *const u8
                as *const libc::c_char
        }
        80 => {
            b"The specified recording driver has been disconnected.\0" as *const u8
                as *const libc::c_char
        }
        81 => {
            b"The length provided exceeds the allowable limit.\0" as *const u8
                as *const libc::c_char
        }
        _ => b"Unknown error.\0" as *const u8 as *const libc::c_char,
    }
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_FinishLoad(
    mut this: *mut SoundDesc,
    mut func: *const libc::c_char,
) {
    let mut warned: bool = false;
    let mut openState: FMOD_OPENSTATE = FMOD_OPENSTATE_READY;
    loop {
        // FMOD_CheckError(
        //     FMOD_Sound_GetOpenState(
        //         (*this).handle,
        //         &mut openState,
        //         std::ptr::null_mut(),
        //         std::ptr::null_mut(),
        //         std::ptr::null_mut(),
        //     ),
        //     b"/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
        //         as *const libc::c_char,
        //     16 as i32,
        //     (*::core::mem::transmute::<
        //         &[u8; 21],
        //         &[libc::c_char; 21],
        //     >(b"SoundDesc_FinishLoad\0"))
        //         .as_ptr(),
        // );
        if openState == FMOD_OPENSTATE_ERROR as i32 as u32 {
            Fatal(
                b"%s: Background file load has failed.\n  Path: %s\0" as *const u8
                    as *const libc::c_char,
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
                b"%s: Background file load hasn't finished. Blocking the main thread.\n  Path: %s\0"
                    as *const u8 as *const libc::c_char,
                func,
                (*this).path,
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_Load(
    mut name: *const libc::c_char,
    mut immediate: bool,
    mut isLooped: bool,
    mut is3D: bool,
) -> *mut SoundDesc {
    let mut mapKey: *const libc::c_char = StrAdd(
        if isLooped as i32 != 0 {
            b"LOOPED:\0" as *const u8 as *const libc::c_char
        } else {
            b"UNLOOPED:\0" as *const u8 as *const libc::c_char
        },
        name,
    );
    let mut this: *mut SoundDesc = Audio_AllocSoundDesc(mapKey);
    StrFree(mapKey);
    if ((*this).name).is_null() {
        let mut path: *const libc::c_char = Resource_GetPath(ResourceType_Sound, name);
        let mut mode: FMOD_MODE = 0_i32 as FMOD_MODE;
        mode |= 0x100_i32 as u32;
        mode |= 0x2000000_i32 as u32;
        mode |= 0x4000_i32 as u32;
        mode |= (if isLooped as i32 != 0 {
            0x2_i32
        } else {
            0x1_i32
        }) as u32;
        mode |= (if is3D as i32 != 0 {
            0x10_i32 | 0x80000_i32
        } else {
            0x8_i32
        }) as u32;
        if !immediate {
            mode |= 0x10000_i32 as u32;
        }
        // FMOD_CheckError(
        //     FMOD_System_CreateSound(
        //         Audio_GetHandle() as *mut FMOD_SYSTEM,
        //         path,
        //         mode,
        //         std::ptr::null_mut(),
        //         &mut (*this).handle,
        //     ),
        //     b"/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
        //         as *const libc::c_char,
        //     48 as i32,
        //     (*::core::mem::transmute::<
        //         &[u8; 15],
        //         &[libc::c_char; 15],
        //     >(b"SoundDesc_Load\0"))
        //         .as_ptr(),
        // );
        // FMOD_CheckError(
        //     FMOD_Sound_SetUserData((*this).handle, this as *mut libc::c_void),
        //     b"/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
        //         as *const libc::c_char,
        //     49 as i32,
        //     (*::core::mem::transmute::<
        //         &[u8; 15],
        //         &[libc::c_char; 15],
        //     >(b"SoundDesc_Load\0"))
        //         .as_ptr(),
        // );
        (*this).name = StrDup(name);
        (*this).path = StrDup(path);
        (*this)._refCount = 1_i32 as u32;
    } else {
        (*this)._refCount = ((*this)._refCount).wrapping_add(1);
        if immediate {
            SoundDesc_FinishLoad(
                this,
                (*::core::mem::transmute::<&[u8; 15], &[libc::c_char; 15]>(b"SoundDesc_Load\0"))
                    .as_ptr(),
            );
        }
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_Acquire(mut this: *mut SoundDesc) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_Free(mut this: *mut SoundDesc) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0_i32 as u32
    } {
        let mut name: *const libc::c_char = (*this).name;
        let mut path: *const libc::c_char = (*this).path;
        // FMOD_CheckError(
        //     FMOD_Sound_Release((*this).handle),
        //     b"/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
        //         as *const libc::c_char,
        //     72 as i32,
        //     (*::core::mem::transmute::<
        //         &[u8; 15],
        //         &[libc::c_char; 15],
        //     >(b"SoundDesc_Free\0"))
        //         .as_ptr(),
        // );
        Audio_DeallocSoundDesc(this);
        StrFree(name);
        StrFree(path);
        MemZero(
            this as *mut libc::c_void,
            ::core::mem::size_of::<SoundDesc>(),
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_GetDuration(mut this: *mut SoundDesc) -> f32 {
    SoundDesc_FinishLoad(
        this,
        (*::core::mem::transmute::<&[u8; 22], &[libc::c_char; 22]>(b"SoundDesc_GetDuration\0"))
            .as_ptr(),
    );
    let mut duration: u32 = 0;
    // FMOD_CheckError(
    //     FMOD_Sound_GetLength(
    //         (*this).handle,
    //         &mut duration,
    //         0x1 as i32 as FMOD_TIMEUNIT,
    //     ),
    //     b"/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
    //         as *const libc::c_char,
    //     85 as i32,
    //     (*::core::mem::transmute::<
    //         &[u8; 22],
    //         &[libc::c_char; 22],
    //     >(b"SoundDesc_GetDuration\0"))
    //         .as_ptr(),
    // );
    duration as f32 / 1000.0f32
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_GetName(mut this: *mut SoundDesc) -> *const libc::c_char {
    (*this).name
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_GetPath(mut this: *mut SoundDesc) -> *const libc::c_char {
    (*this).path
}

#[no_mangle]
pub unsafe extern "C" fn SoundDesc_ToFile(mut this: *mut SoundDesc, mut name: *const libc::c_char) {
    SoundDesc_FinishLoad(
        this,
        (*::core::mem::transmute::<&[u8; 17], &[libc::c_char; 17]>(b"SoundDesc_ToFile\0")).as_ptr(),
    );
    let mut length: u32 = 0;
    let mut channels: i32 = 0;
    let mut bitsPerSample: i32 = 0;
    // FMOD_CheckError(
    //     FMOD_Sound_GetLength(
    //         (*this).handle,
    //         &mut length,
    //         0x8 as i32 as FMOD_TIMEUNIT,
    //     ),
    //     b"/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
    //         as *const libc::c_char,
    //     128 as i32,
    //     (*::core::mem::transmute::<
    //         &[u8; 17],
    //         &[libc::c_char; 17],
    //     >(b"SoundDesc_ToFile\0"))
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
    //     b"/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
    //         as *const libc::c_char,
    //     129 as i32,
    //     (*::core::mem::transmute::<
    //         &[u8; 17],
    //         &[libc::c_char; 17],
    //     >(b"SoundDesc_ToFile\0"))
    //         .as_ptr(),
    // );
    let mut bytesPerSample: i32 = bitsPerSample / 8_i32;
    let mut sampleRate: f32 = 0.;
    // FMOD_Sound_GetDefaults((*this).handle, &mut sampleRate, std::ptr::null_mut());
    let mut ptr1: *mut libc::c_void = std::ptr::null_mut();
    let mut _len1: u32 = 0;
    let mut _ptr2: *mut libc::c_void = std::ptr::null_mut();
    let mut _len2: u32 = 0;
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
    //     b"/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
    //         as *const libc::c_char,
    //     137 as i32,
    //     (*::core::mem::transmute::<
    //         &[u8; 17],
    //         &[libc::c_char; 17],
    //     >(b"SoundDesc_ToFile\0"))
    //         .as_ptr(),
    // );
    let mut file: *mut File = File_Create(name);
    if file.is_null() {
        Fatal(
            b"SoundDesc_ToFile: Failed to create file.\nPath: %s\0" as *const u8
                as *const libc::c_char,
            name,
        );
    }
    File_Write(
        file,
        b"RIFF\0" as *const u8 as *const libc::c_char as *const libc::c_void,
        4_i32 as u32,
    );
    File_WriteI32(file, (36_i32 as u32).wrapping_add(length) as i32);
    File_Write(
        file,
        b"WAVE\0" as *const u8 as *const libc::c_char as *const libc::c_void,
        4_i32 as u32,
    );
    File_Write(
        file,
        b"fmt \0" as *const u8 as *const libc::c_char as *const libc::c_void,
        4_i32 as u32,
    );
    File_WriteI32(file, 16_i32);
    File_WriteI16(file, 1_i32 as i16);
    File_WriteI16(file, channels as i16);
    File_WriteI32(file, sampleRate as i32);
    File_WriteI32(
        file,
        ((bytesPerSample * channels) as f32 * sampleRate) as i32,
    );
    File_WriteI16(file, (bytesPerSample * channels) as i16);
    File_WriteI16(file, bitsPerSample as i16);
    File_Write(
        file,
        b"data\0" as *const u8 as *const libc::c_char as *const libc::c_void,
        4_i32 as u32,
    );
    File_WriteI32(file, length as i32);
    File_Write(file, ptr1, length);
    File_Close(file);
    // FMOD_CheckError(
    //     FMOD_Sound_Unlock((*this).handle, ptr1, ptr2, len1, len2),
    //     b"/Users/dgavedissian/Work/ltheory/libphx/src/SoundDesc.c\0" as *const u8
    //         as *const libc::c_char,
    //     163 as i32,
    //     (*::core::mem::transmute::<
    //         &[u8; 17],
    //         &[libc::c_char; 17],
    //     >(b"SoundDesc_ToFile\0"))
    //         .as_ptr(),
    // );
}
