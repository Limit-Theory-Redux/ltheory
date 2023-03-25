use crate::internal::Memory::*;
use crate::Audio::*;
use crate::Common::*;
use crate::Math::Vec3;
use crate::SoundDesc::*;
use libc;

extern "C" {
    pub type FMOD_CHANNEL;
    pub type FMOD_SOUND;
    pub type FMOD_SYSTEM;
    pub type FMOD_CHANNELCONTROL;
    pub type FMOD_CHANNELGROUP;
    pub type FMOD_SPEAKERMODE;
    fn FMOD_System_GetSoftwareFormat(
        system: *mut FMOD_SYSTEM,
        samplerate: *mut i32,
        speakermode: *mut FMOD_SPEAKERMODE,
        numrawspeakers: *mut i32,
    );
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
    fn FMOD_Channel_GetVolume(channel: *mut FMOD_CHANNEL, volume: *mut f32);
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
    fn FMOD_Channel_GetAudibility(channel: *mut FMOD_CHANNEL, out: *mut f32);

    fn FMOD_Channel_GetDSPClock(
        channel: *mut FMOD_CHANNEL,
        dspclock: *mut u64,
        parentclock: *mut u64,
    );
    fn FMOD_Channel_SetDelay(
        channel: *mut FMOD_CHANNEL,
        dspclock_start: u64,
        dspclock_end: u64,
        stopchannels: u32,
    );
    fn FMOD_Channel_GetDelay(
        channel: *mut FMOD_CHANNEL,
        dspclock_start: *mut u64,
        dspclock_end: *mut u64,
        stopchannels: *mut u32,
    );
    fn FMOD_Channel_AddFadePoint(channel: *mut FMOD_CHANNEL, dspclock: u64, volume: f32);
    fn FMOD_Channel_SetFadePointRamp(channel: *mut FMOD_CHANNEL, dspclock: u64, volume: f32);
    fn FMOD_Channel_RemoveFadePoints(
        channel: *mut FMOD_CHANNEL,
        dspclock_start: u64,
        dspclock_end: u64,
    );
    fn FMOD_Channel_GetFadePoints(
        channel: *mut FMOD_CHANNEL,
        numpoints: *mut i32,
        point_dspclock: *mut u64,
        point_volume: *mut f32,
    );

}

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

unsafe extern "C" fn Sound_Callback(
    channel: *mut FMOD_CHANNELCONTROL,
    _controlType: FMOD_CHANNELCONTROL_TYPE,
    callbackType: FMOD_CHANNELCONTROL_CALLBACK_TYPE,
    _a: *mut libc::c_void,
    _b: *mut libc::c_void,
) -> FMOD_RESULT {
    if callbackType == FMOD_CHANNELCONTROL_CALLBACK_END as i32 as u32 {
        let mut this: *mut Sound = std::ptr::null_mut();
        FMOD_CheckError(
            FMOD_Channel_GetUserData(
                channel as *mut FMOD_CHANNEL,
                &mut this as *mut *mut Sound as *mut *mut libc::c_void,
            ),
            c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c"),
            23,
            c_str!("Sound_Callback"),
        );
        Sound_SetState(this, 4 as SoundState);
    }
    FMOD_OK
}

#[inline]
unsafe extern "C" fn Sound_EnsureLoadedImpl(this: *mut Sound, func: *const libc::c_char) {
    if (*this).state as i32 == 1 {
        SoundDesc_FinishLoad((*this).desc, func);
        // FMOD_CheckError(
        //     FMOD_System_PlaySound(
        //         Audio_GetHandle() as *mut FMOD_SYSTEM,
        //         (*(*this).desc).handle,
        //         std::ptr::null_mut(),
        //         1 as i32,
        //         &mut (*this).handle,
        //     ),
        //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
        //         as *const libc::c_char,
        //     33 as i32,
        //     (*std::mem::transmute::<
        //         &[u8; 23],
        //         &[libc::c_char; 23],
        //     >(c_str!("Sound_EnsureLoadedImpl\0"))
        //         .as_ptr(),
        // );
        // FMOD_CheckError(
        //     FMOD_Channel_SetUserData((*this).handle, this as *mut _),
        //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
        //         as *const libc::c_char,
        //     34 as i32,
        //     (*std::mem::transmute::<
        //         &[u8; 23],
        //         &[libc::c_char; 23],
        //     >(c_str!("Sound_EnsureLoadedImpl\0"))
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
        //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
        //         as *const libc::c_char,
        //     35 as i32,
        //     (*std::mem::transmute::<
        //         &[u8; 23],
        //         &[libc::c_char; 23],
        //     >(c_str!("Sound_EnsureLoadedImpl\0"))
        //         .as_ptr(),
        // );
        Sound_SetState(this, 2 as SoundState);
        if Sound_Get3D(this) {
            let mut zero: Vec3 = Vec3::ZERO;
            Sound_Set3DPos(this, &mut zero, &mut zero);
        }
    }
}

#[inline]
unsafe extern "C" fn Sound_EnsureNotFreedImpl(this: *mut Sound, func: *const libc::c_char) {
    if (*this).state as i32 == 5 {
        let name: *const libc::c_char = if (*(*this).desc)._refCount > 0 {
            (*(*this).desc).name
        } else {
            c_str!("<SoundDesc has been freed>")
        };
        Fatal(c_str!("%s: Sound has been freed.\n  Name: %s"), func, name);
    }
}

#[inline]
unsafe extern "C" fn Sound_EnsureStateImpl(this: *mut Sound, func: *const libc::c_char) {
    Sound_EnsureLoadedImpl(this, func);
    Sound_EnsureNotFreedImpl(this, func);
}

unsafe extern "C" fn Sound_SetState(this: *mut Sound, nextState: SoundState) {
    if nextState as i32 == (*this).state as i32 {}
    // match nextState as i32 {
    //     3 => {
    //         FMOD_CheckError(
    //             FMOD_Channel_SetPaused((*this).handle, 0 as i32),
    //             c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //                 as *const libc::c_char,
    //             71 as i32,
    //             (*std::mem::transmute::<
    //                 &[u8; 15],
    //                 &[libc::c_char; 15],
    //             >(c_str!("Sound_SetState\0"))
    //                 .as_ptr(),
    //         );
    //     }
    //     2 => {
    //         FMOD_CheckError(
    //             FMOD_Channel_SetPaused((*this).handle, 1 as i32),
    //             c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //                 as *const libc::c_char,
    //             75 as i32,
    //             (*std::mem::transmute::<
    //                 &[u8; 15],
    //                 &[libc::c_char; 15],
    //             >(c_str!("Sound_SetState\0"))
    //                 .as_ptr(),
    //         );
    //     }
    //     4 => {
    //         FMOD_CheckError(
    //             FMOD_Channel_Stop((*this).handle),
    //             c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //                 as *const libc::c_char,
    //             79 as i32,
    //             (*std::mem::transmute::<
    //                 &[u8; 15],
    //                 &[libc::c_char; 15],
    //             >(c_str!("Sound_SetState\0"))
    //                 .as_ptr(),
    //         );
    //     }
    //     1 | 5 => {}
    //     _ => {
    //         Fatal(
    //             c_str!("Sound_SetState: Unhandled case: %i\0" as *const u8
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
    name: *const libc::c_char,
    immediate: bool,
    isLooped: bool,
    is3D: bool,
) -> *mut Sound {
    let desc: *mut SoundDesc = SoundDesc_Load(name, immediate, isLooped, is3D);
    let this: *mut Sound = Audio_AllocSound();
    (*this).desc = desc;
    Sound_SetState(this, 1 as SoundState);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Load(
    name: *const libc::c_char,
    isLooped: bool,
    is3D: bool,
) -> *mut Sound {
    let this: *mut Sound = Sound_Create(name, true, isLooped, is3D);
    Sound_EnsureLoadedImpl(this, c_str!("Sound_Load"));
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
pub unsafe extern "C" fn Sound_Clone(this: *mut Sound) -> *mut Sound {
    Sound_EnsureStateImpl(this, c_str!("Sound_Clone"));
    let clone: *mut Sound = Audio_AllocSound();
    *clone = *this;
    SoundDesc_Acquire((*this).desc);
    (*clone).handle = std::ptr::null_mut();
    (*clone).state = 0 as SoundState;
    Sound_SetState(clone, 1 as SoundState);
    clone
}

#[no_mangle]
pub unsafe extern "C" fn Sound_ToFile(this: *mut Sound, name: *const libc::c_char) {
    Sound_EnsureStateImpl(this, c_str!("Sound_ToFile"));
    SoundDesc_ToFile((*this).desc, name);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Acquire(this: *mut Sound) {
    Sound_EnsureStateImpl(this, c_str!("Sound_Acquire"));
    (*(*this).desc)._refCount = ((*(*this).desc)._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Free(this: *mut Sound) {
    Sound_EnsureStateImpl(this, c_str!("Sound_Free"));
    Sound_SetState(this, 4 as SoundState);
    Sound_SetState(this, 5 as SoundState);
    SoundDesc_Free((*this).desc);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Play(this: *mut Sound) {
    Sound_EnsureStateImpl(this, c_str!("Sound_Play"));
    Sound_SetState(this, 3 as SoundState);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Pause(this: *mut Sound) {
    Sound_EnsureStateImpl(this, c_str!("Sound_Pause"));
    Sound_SetState(this, 2 as SoundState);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Rewind(this: *mut Sound) {
    Sound_EnsureStateImpl(this, c_str!("Sound_Rewind"));
    // FMOD_CheckError(
    //     FMOD_Channel_SetPosition(
    //         (*this).handle,
    //         0 as i32 as u32,
    //         0x2 as i32 as FMOD_TIMEUNIT,
    //     ),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     153 as i32,
    //     c_str!("Sound_Rewind\0"))
    //         .as_ptr(),
    // );
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Get3D(this: *mut Sound) -> bool {
    Sound_EnsureStateImpl(this, c_str!("Sound_Get3D"));
    let mode: FMOD_MODE = 0;
    // FMOD_CheckError(
    //     FMOD_Channel_GetMode((*this).handle, &mut mode),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     159 as i32,
    //     c_str!("Sound_Get3D\0"))
    //         .as_ptr(),
    // );
    mode & 0x10 == 0x10
}

#[no_mangle]
pub unsafe extern "C" fn Sound_GetDuration(this: *mut Sound) -> f32 {
    Sound_EnsureStateImpl(this, c_str!("Sound_GetDuration"));
    SoundDesc_GetDuration((*this).desc)
}

#[no_mangle]
pub unsafe extern "C" fn Sound_GetLooped(_this: *mut Sound) -> bool {
    // Sound_EnsureStateImpl(
    //     this,
    //     c_str!("Sound_GetLooped\0"))
    //         .as_ptr(),
    // );
    let mode: FMOD_MODE = 0;
    // FMOD_CheckError(
    //     FMOD_Channel_GetMode((*this).handle, &mut mode),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     171 as i32,
    //     c_str!("Sound_GetLooped\0"))
    //         .as_ptr(),
    // );
    mode & 0x2 == 0x2
}

#[no_mangle]
pub unsafe extern "C" fn Sound_GetName(this: *mut Sound) -> *const libc::c_char {
    Sound_EnsureNotFreedImpl(this, c_str!("Sound_GetName"));
    SoundDesc_GetName((*this).desc)
}

#[no_mangle]
pub unsafe extern "C" fn Sound_GetPath(this: *mut Sound) -> *const libc::c_char {
    Sound_EnsureNotFreedImpl(this, c_str!("Sound_GetPath"));
    SoundDesc_GetPath((*this).desc)
}

#[no_mangle]
pub unsafe extern "C" fn Sound_IsFinished(this: *mut Sound) -> bool {
    (*this).state as i32 == 4
}

#[no_mangle]
pub unsafe extern "C" fn Sound_IsPlaying(this: *mut Sound) -> bool {
    (*this).state as i32 == 3
}

#[no_mangle]
pub unsafe extern "C" fn Sound_IsAudible(this: *mut Sound) -> bool {
    Sound_EnsureStateImpl(this, c_str!("Sound_Set3DLevel"));

    let audibility = 0.0f32;
    // FMOD_Channel_GetAudibility((*this).handle, &mut audibility);
    return audibility > 0.0f32;
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Attach3DPos(this: *mut Sound, pos: *const Vec3, vel: *const Vec3) {
    Sound_Set3DPos(this, pos, vel);
    (*this).autoPos = pos;
    (*this).autoVel = vel;
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Set3DLevel(this: *mut Sound, _level: f32) {
    Sound_EnsureStateImpl(this, c_str!("Sound_Set3DLevel"));
    // FMOD_CheckError(
    //     FMOD_Channel_Set3DLevel((*this).handle, level),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     202 as i32,
    //     (*std::mem::transmute::<
    //         &[u8; 17],
    //         &[libc::c_char; 17],
    //     >(c_str!("Sound_Set3DLevel\0"))
    //         .as_ptr(),
    // );
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Set3DPos(this: *mut Sound, pos: *const Vec3, vel: *const Vec3) {
    Sound_EnsureStateImpl(this, c_str!("Sound_Set3DPos"));
    // FMOD_CheckError(
    //     FMOD_Channel_Set3DAttributes(
    //         (*this).handle,
    //         pos as *mut FMOD_VECTOR,
    //         vel as *mut FMOD_VECTOR,
    //     ),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     210 as i32,
    //     c_str!("Sound_Set3DPos\0"))
    //         .as_ptr(),
    // );
}

#[no_mangle]
pub unsafe extern "C" fn Sound_SetFreeOnFinish(this: *mut Sound, freeOnFinish: bool) {
    (*this).freeOnFinish = freeOnFinish;
}

#[no_mangle]
pub unsafe extern "C" fn Sound_SetPan(this: *mut Sound, pan: f32) {
    Sound_EnsureStateImpl(this, c_str!("Sound_SetPan"));
    // FMOD_CheckError(
    //     FMOD_Channel_SetPan((*this).handle, pan),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     219 as i32,
    //     c_str!("Sound_SetPan\0"))
    //         .as_ptr(),
    // );
}

#[no_mangle]
pub unsafe extern "C" fn Sound_SetPitch(this: *mut Sound, pitch: f32) {
    Sound_EnsureStateImpl(this, c_str!("Sound_SetPitch"));
    // FMOD_CheckError(
    //     FMOD_Channel_SetPitch((*this).handle, pitch),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     224 as i32,
    //     c_str!("Sound_SetPitch\0"))
    //         .as_ptr(),
    // );
}

#[no_mangle]
pub unsafe extern "C" fn Sound_SetPlayPos(this: *mut Sound, seconds: f32) {
    Sound_EnsureStateImpl(this, c_str!("Sound_SetPlayPos"));
    let ms: u32 = f64::round((seconds * 1000.0f32) as f64) as u32;
    // FMOD_CheckError(
    //     FMOD_Channel_SetPosition(
    //         (*this).handle,
    //         ms,
    //         0x1 as i32 as FMOD_TIMEUNIT,
    //     ),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     232 as i32,
    //     (*std::mem::transmute::<
    //         &[u8; 17],
    //         &[libc::c_char; 17],
    //     >(c_str!("Sound_SetPlayPos\0"))
    //         .as_ptr(),
    // );
}

#[no_mangle]
pub unsafe extern "C" fn Sound_SetVolume(this: *mut Sound, volume: f32) {
    Sound_EnsureStateImpl(this, c_str!("Sound_SetVolume"));
    // FMOD_CheckError(
    //     FMOD_Channel_SetVolume((*this).handle, volume),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Sound.c\0" as *const u8
    //         as *const libc::c_char,
    //     237 as i32,
    //     c_str!("Sound_SetVolume\0"))
    //         .as_ptr(),
    // );
}

#[no_mangle]
pub unsafe extern "C" fn Sound_FadeIn(this: *mut Sound, seconds: f32) {
    Sound_EnsureStateImpl(this, c_str!("Sound_FadeIn"));
    // Assert(seconds >= 0.0f);

    if !Sound_IsPlaying(this) {
        Sound_Play(this);
    }

    // // Already fading in/out?
    // let numpoints = 0;
    // FMOD_Channel_GetFadePoints((*this).handle, &mut numpoints, std::ptr::null_mut(), std::ptr::null_mut());
    // if numpoints > 0 {
    //     return;
    // }

    // let rate = 0;
    // FMOD_System_GetSoftwareFormat(Audio_GetHandle() as *mut _, &mut rate, std::ptr::null_mut(), std::ptr::null_mut());
    // let fadeTime = (rate as f32 * seconds) as u64;

    // let volume = 1.0f32;
    // FMOD_Channel_GetVolume((*this).handle, &mut volume);

    // let dspClock = 0;
    // FMOD_Channel_GetDSPClock((*this).handle, std::ptr::null_mut(), &mut dspClock);

    // FMOD_Channel_SetDelay((*this).handle, dspClock, 0, 0);

    // FMOD_Channel_AddFadePoint((*this).handle, dspClock, 0.0f32);
    // FMOD_Channel_AddFadePoint((*this).handle, dspClock + fadeTime, volume);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_FadeOut(this: *mut Sound, seconds: f32) {
    Sound_EnsureStateImpl(this, c_str!("Sound_FadeOut"));
    // Assert(seconds >= 0.0f);

    // // Already fading in/out?
    // let numpoints = 0;
    // FMOD_Channel_GetFadePoints((*this).handle, &mut numpoints, std::ptr::null_mut(), std::ptr::null_mut());
    // if numpoints > 0 {
    //     return;
    // }

    // let rate = 0;
    // FMOD_System_GetSoftwareFormat(Audio_GetHandle() as *mut _, &mut rate, std::ptr::null_mut(), std::ptr::null_mut());
    // let fadeTime = (rate as f32 * seconds) as u64;

    // let volume = 1.0f32;
    // FMOD_Channel_GetVolume((*this).handle, &mut volume);

    // let dspClock = 0;
    // FMOD_Channel_GetDSPClock((*this).handle, std::ptr::null_mut(), &mut dspClock);

    // FMOD_Channel_AddFadePoint((*this).handle, dspClock, volume);
    // FMOD_Channel_AddFadePoint((*this).handle, dspClock + fadeTime, 0.0f32);

    // FMOD_Channel_SetDelay((*this).handle, dspClock + fadeTime, 0, 0);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_LoadPlay(
    name: *const libc::c_char,
    isLooped: bool,
    is3D: bool,
) -> *mut Sound {
    let this: *mut Sound = Sound_Load(name, isLooped, is3D);
    Sound_Play(this);
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
    Sound_Attach3DPos(this, pos, vel);
    Sound_Play(this);
    this
}

#[no_mangle]
pub unsafe extern "C" fn Sound_LoadPlayFree(name: *const libc::c_char, isLooped: bool, is3D: bool) {
    let this: *mut Sound = Sound_Load(name, isLooped, is3D);
    Sound_SetFreeOnFinish(this, true);
    Sound_Play(this);
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
    Sound_Attach3DPos(this, pos, vel);
    Sound_SetFreeOnFinish(this, true);
    Sound_Play(this);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_ClonePlay(this: *mut Sound) -> *mut Sound {
    let clone: *mut Sound = Sound_Clone(this);
    Sound_Play(clone);
    clone
}

#[no_mangle]
pub unsafe extern "C" fn Sound_ClonePlayAttached(
    this: *mut Sound,
    pos: *const Vec3,
    vel: *const Vec3,
) -> *mut Sound {
    let clone: *mut Sound = Sound_Clone(this);
    Sound_Attach3DPos(clone, pos, vel);
    Sound_Play(clone);
    clone
}

#[no_mangle]
pub unsafe extern "C" fn Sound_ClonePlayFree(this: *mut Sound) {
    let clone: *mut Sound = Sound_Clone(this);
    Sound_SetFreeOnFinish(clone, true);
    Sound_Play(clone);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_ClonePlayFreeAttached(
    this: *mut Sound,
    pos: *const Vec3,
    vel: *const Vec3,
) {
    let clone: *mut Sound = Sound_Clone(this);
    Sound_Attach3DPos(clone, pos, vel);
    Sound_SetFreeOnFinish(clone, true);
    Sound_Play(clone);
}

#[no_mangle]
pub unsafe extern "C" fn Sound_Update(this: *mut Sound) {
    if (*this).state as i32 == 1 {
        return;
    }
    if Sound_Get3D(this) {
        Sound_Set3DPos(this, (*this).autoPos, (*this).autoVel);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Sound_IsFreed(this: *mut Sound) -> bool {
    (*this).state as i32 == 5
}
