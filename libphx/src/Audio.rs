use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use crate::MemPool::*;
use crate::Sound::*;
use crate::SoundDesc::*;
use crate::StrMap::*;
use libc;

extern "C" {
    pub type FMOD_CHANNEL;
    pub type FMOD_SOUND;
    pub type FMOD_SYSTEM;
    fn FMOD_Debug_Initialize(
        flags: FMOD_DEBUG_FLAGS,
        mode: FMOD_DEBUG_MODE,
        callback: FMOD_DEBUG_CALLBACK,
        filename: *const libc::c_char,
    ) -> FMOD_RESULT;
    fn FMOD_System_Create(system: *mut *mut FMOD_SYSTEM, headerversion: u32) -> FMOD_RESULT;
    fn FMOD_System_Release(system: *mut FMOD_SYSTEM) -> FMOD_RESULT;
    fn FMOD_System_Init(
        system: *mut FMOD_SYSTEM,
        maxchannels: i32,
        flags: FMOD_INITFLAGS,
        extradriverdata: *mut libc::c_void,
    ) -> FMOD_RESULT;
    fn FMOD_System_Update(system: *mut FMOD_SYSTEM) -> FMOD_RESULT;
    fn FMOD_System_Set3DSettings(
        system: *mut FMOD_SYSTEM,
        dopplerscale: f32,
        distancefactor: f32,
        rolloffscale: f32,
    ) -> FMOD_RESULT;
    fn FMOD_System_Set3DListenerAttributes(
        system: *mut FMOD_SYSTEM,
        listener: i32,
        pos: *const FMOD_VECTOR,
        vel: *const FMOD_VECTOR,
        forward: *const FMOD_VECTOR,
        up: *const FMOD_VECTOR,
    ) -> FMOD_RESULT;
    fn FMOD_System_GetVersion(system: *mut FMOD_SYSTEM, version: *mut u32) -> FMOD_RESULT;
}

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
pub type FMOD_INITFLAGS = u32;
pub type FMOD_DEBUG_CALLBACK = Option<
    unsafe extern "C" fn(
        FMOD_DEBUG_FLAGS,
        *const libc::c_char,
        i32,
        *const libc::c_char,
        *const libc::c_char,
    ) -> FMOD_RESULT,
>;
pub type FMOD_DEBUG_FLAGS = u32;
pub type FMOD_DEBUG_MODE = u32;
pub const FMOD_DEBUG_MODE_FORCEINT: FMOD_DEBUG_MODE = 65536;
pub const FMOD_DEBUG_MODE_CALLBACK: FMOD_DEBUG_MODE = 2;
pub const FMOD_DEBUG_MODE_FILE: FMOD_DEBUG_MODE = 1;
pub const FMOD_DEBUG_MODE_TTY: FMOD_DEBUG_MODE = 0;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct FMOD_VECTOR {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[inline]
unsafe extern "C" fn FMODError_ToString(self_1: FMOD_RESULT) -> *const libc::c_char {
    match self_1 {
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
    // let flags: FMOD_DEBUG_FLAGS = 0 as i32 as FMOD_DEBUG_FLAGS;
    // flags |= 0 as i32 as u32;
    // let res: FMOD_RESULT = FMOD_OK;
    // res = FMOD_Debug_Initialize(
    //     flags,
    //     FMOD_DEBUG_MODE_FILE,
    //     None,
    //     c_str!("log/fmod.txt"),
    // );
    // if res as u32 != FMOD_OK as i32 as u32
    //     && res as u32 != FMOD_ERR_UNSUPPORTED as i32 as u32
    // {
    //     FMOD_CheckError(
    //         res,
    //         c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Audio.c\0" as *const u8
    //             as *const libc::c_char,
    //         39 as i32,
    //         c_str!("Audio_Init\0"))
    //             .as_ptr(),
    //     );
    // }
    // FMOD_CheckError(
    //     FMOD_System_Create(&mut this.handle, 0x20208 as i32 as u32),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Audio.c\0" as *const u8
    //         as *const libc::c_char,
    //     43 as i32,
    //     c_str!("Audio_Init\0"))
    //         .as_ptr(),
    // );
    // let version: u32 = 0;
    // FMOD_CheckError(
    //     FMOD_System_GetVersion(this.handle, &mut version),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Audio.c\0" as *const u8
    //         as *const libc::c_char,
    //     46 as i32,
    //     c_str!("Audio_Init\0"))
    //         .as_ptr(),
    // );
    // if version < 0x20208 as i32 as u32 {
    //     Fatal(
    //         c_str!("Audio_Create: FMOD library link/compile version mismatch\0" as *const u8
    //             as *const libc::c_char,
    //     );
    // }
    // let flags_0: FMOD_INITFLAGS = 0 as i32 as FMOD_INITFLAGS;
    // flags_0 |= 0 as i32 as u32;
    // flags_0 |= 0x4 as i32 as u32;
    // flags_0 |= 0x200 as i32 as u32;
    // FMOD_CheckError(
    //     FMOD_System_Init(
    //         this.handle,
    //         1024 as i32,
    //         flags_0,
    //         std::ptr::null_mut(),
    //     ),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Audio.c\0" as *const u8
    //         as *const libc::c_char,
    //     59 as i32,
    //     c_str!("Audio_Init\0"))
    //         .as_ptr(),
    // );
    this.descMap = StrMap_Create(128 as i32 as u32);
    this.soundPool = MemPool_Create(
        std::mem::size_of::<Sound>() as usize as u32,
        128 as i32 as u32,
    );
}

#[no_mangle]
pub unsafe extern "C" fn Audio_Free() {
    // FMOD_CheckError(
    //     FMOD_System_Release(this.handle),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Audio.c\0" as *const u8
    //         as *const libc::c_char,
    //     69 as i32,
    //     c_str!("Audio_Free\0"))
    //         .as_ptr(),
    // );
    StrMap_Free(this.descMap);
    MemPool_Free(this.soundPool);
    this.playingSounds.clear();
    this.freeingSounds.clear();
}

#[no_mangle]
pub unsafe extern "C" fn Audio_AttachListenerPos(
    _pos: *const Vec3,
    _vel: *const Vec3,
    _fwd: *const Vec3,
    _up: *const Vec3,
) {
    // this.autoPos = pos;
    // this.autoVel = vel;
    // this.autoFwd = fwd;
    // this.autoUp = up;
    // Audio_SetListenerPos(pos, vel, fwd, up);
}

#[no_mangle]
pub unsafe extern "C" fn Audio_Set3DSettings(_doppler: f32, _scale: f32, _rolloff: f32) {
    // FMOD_CheckError(
    //     FMOD_System_Set3DSettings(this.handle, doppler, scale, rolloff),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Audio.c\0" as *const u8
    //         as *const libc::c_char,
    //     85 as i32,
    //     (*std::mem::transmute::<
    //         &[u8; 20],
    //         &[libc::c_char; 20],
    //     >(c_str!("Audio_Set3DSettings\0"))
    //         .as_ptr(),
    // );
}

#[no_mangle]
pub unsafe extern "C" fn Audio_SetListenerPos(
    _pos: *const Vec3,
    _vel: *const Vec3,
    _fwd: *const Vec3,
    _up: *const Vec3,
) {
    // FMOD_CheckError(
    //     FMOD_System_Set3DListenerAttributes(
    //         this.handle,
    //         0 as i32,
    //         pos as *mut FMOD_VECTOR,
    //         vel as *mut FMOD_VECTOR,
    //         fwd as *mut FMOD_VECTOR,
    //         up as *mut FMOD_VECTOR,
    //     ),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Audio.c\0" as *const u8
    //         as *const libc::c_char,
    //     106 as i32,
    //     (*std::mem::transmute::<
    //         &[u8; 21],
    //         &[libc::c_char; 21],
    //     >(c_str!("Audio_SetListenerPos\0"))
    //         .as_ptr(),
    // );
}

#[no_mangle]
pub unsafe extern "C" fn Audio_Update() {
    // FMOD_CheckError(
    //     FMOD_System_Update(this.handle),
    //     c_str!("/Users/dgavedissian/Work/ltheory/libphx/src/Audio.c\0" as *const u8
    //         as *const libc::c_char,
    //     110 as i32,
    //     c_str!("Audio_Update\0"))
    //         .as_ptr(),
    // );
    Audio_SetListenerPos(this.autoPos, this.autoVel, this.autoFwd, this.autoUp);

    let mut soundsToRemove: Vec<usize> = Vec::new();
    for (i, sound) in this.playingSounds.iter().enumerate() {
        if !Sound_IsFreed(*sound) && Sound_IsPlaying(*sound) as i32 != 0 {
            Sound_Update(*sound);
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
    0
    // let size: u32 = StrMap_GetSize(this.descMap);
    // return size as i32;
}

#[no_mangle]
pub unsafe extern "C" fn Audio_GetPlayingCount() -> i32 {
    this.playingSounds.len() as i32
}

#[no_mangle]
pub unsafe extern "C" fn Audio_GetTotalCount() -> i32 {
    let size: u32 = MemPool_GetSize(this.soundPool);
    size as i32
}

#[no_mangle]
pub unsafe extern "C" fn Audio_GetHandle() -> *mut libc::c_void {
    this.handle as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn Audio_AllocSoundDesc(name: *const libc::c_char) -> *mut SoundDesc {
    let mut desc: *mut SoundDesc = StrMap_Get(this.descMap, name) as *mut SoundDesc;
    if desc.is_null() {
        desc = MemNewZero!(SoundDesc);
        StrMap_Set(this.descMap, name, desc as *mut _);
    }
    desc
}

#[no_mangle]
pub unsafe extern "C" fn Audio_DeallocSoundDesc(desc: *mut SoundDesc) {
    StrMap_Remove(this.descMap, (*desc).name);
    MemFree(desc as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn Audio_AllocSound() -> *mut Sound {
    MemPool_Alloc(this.soundPool) as *mut Sound
}

#[no_mangle]
pub unsafe extern "C" fn Audio_DeallocSound(sound: *mut Sound) {
    MemPool_Dealloc(this.soundPool, sound as *mut _);
}

#[no_mangle]
pub unsafe extern "C" fn Audio_SoundStateChanged(sound: *mut Sound) {
    if Sound_IsFreed(sound) {
        this.freeingSounds.push(sound);
    } else if Sound_IsPlaying(sound) {
        this.playingSounds.push(sound);
    }
}
