use ::libc;
use crate::internal::Memory::*;
use crate::PhxSignal::*;
use crate::ResourceType::*;

extern "C" {
    fn Directory_Create(path: cstr) -> bool;
    fn Fatal(_: cstr, _: ...);
    fn Gamepad_Update();
    fn Input_Init();
    fn Input_Free();
    fn Input_Update();
    fn Joystick_Update();
    fn Keyboard_Init();
    fn Keyboard_Free();
    fn Keyboard_UpdatePre();
    fn Keyboard_UpdatePost();
    fn Metric_Reset();
    fn Mouse_Init();
    fn Mouse_Free();
    fn Mouse_Update();
    fn Signal_Init();
    fn Signal_Free();
    fn Profiler_Begin(_: cstr);
    fn Profiler_End();
    fn Resource_Init();
    fn exit(_: libc::c_int) -> !;
    fn SDL_GL_SetAttribute(attr: SDL_GLattr, value: libc::c_int) -> libc::c_int;
    fn abort() -> !;
    fn SDL_QuitSubSystem(flags: Uint32);
    fn SDL_GetVersion(ver: *mut SDL_version);
    fn puts(_: *const libc::c_char) -> libc::c_int;
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn SDL_Init(flags: Uint32) -> libc::c_int;
    fn atexit(_: Option::<unsafe extern "C" fn() -> ()>) -> libc::c_int;
    fn SDL_Quit();
    fn SDL_InitSubSystem(flags: Uint32) -> libc::c_int;
    fn ShaderVar_Init();
    fn ShaderVar_Free();
    fn TimeStamp_Get() -> TimeStamp;
    fn TimeStamp_GetElapsed(start: TimeStamp) -> libc::c_double;
}
pub type int32_t = libc::c_int;
pub type uint8_t = libc::c_uchar;
pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
pub type uint64 = uint64_t;
pub type ResourceType = int32;
pub type TimeStamp = uint64;
pub type SDL_GLattr = libc::c_uint;
pub const SDL_GL_FLOATBUFFERS: SDL_GLattr = 27;
pub const SDL_GL_CONTEXT_NO_ERROR: SDL_GLattr = 26;
pub const SDL_GL_CONTEXT_RESET_NOTIFICATION: SDL_GLattr = 25;
pub const SDL_GL_CONTEXT_RELEASE_BEHAVIOR: SDL_GLattr = 24;
pub const SDL_GL_FRAMEBUFFER_SRGB_CAPABLE: SDL_GLattr = 23;
pub const SDL_GL_SHARE_WITH_CURRENT_CONTEXT: SDL_GLattr = 22;
pub const SDL_GL_CONTEXT_PROFILE_MASK: SDL_GLattr = 21;
pub const SDL_GL_CONTEXT_FLAGS: SDL_GLattr = 20;
pub const SDL_GL_CONTEXT_EGL: SDL_GLattr = 19;
pub const SDL_GL_CONTEXT_MINOR_VERSION: SDL_GLattr = 18;
pub const SDL_GL_CONTEXT_MAJOR_VERSION: SDL_GLattr = 17;
pub const SDL_GL_RETAINED_BACKING: SDL_GLattr = 16;
pub const SDL_GL_ACCELERATED_VISUAL: SDL_GLattr = 15;
pub const SDL_GL_MULTISAMPLESAMPLES: SDL_GLattr = 14;
pub const SDL_GL_MULTISAMPLEBUFFERS: SDL_GLattr = 13;
pub const SDL_GL_STEREO: SDL_GLattr = 12;
pub const SDL_GL_ACCUM_ALPHA_SIZE: SDL_GLattr = 11;
pub const SDL_GL_ACCUM_BLUE_SIZE: SDL_GLattr = 10;
pub const SDL_GL_ACCUM_GREEN_SIZE: SDL_GLattr = 9;
pub const SDL_GL_ACCUM_RED_SIZE: SDL_GLattr = 8;
pub const SDL_GL_STENCIL_SIZE: SDL_GLattr = 7;
pub const SDL_GL_DEPTH_SIZE: SDL_GLattr = 6;
pub const SDL_GL_DOUBLEBUFFER: SDL_GLattr = 5;
pub const SDL_GL_BUFFER_SIZE: SDL_GLattr = 4;
pub const SDL_GL_ALPHA_SIZE: SDL_GLattr = 3;
pub const SDL_GL_BLUE_SIZE: SDL_GLattr = 2;
pub const SDL_GL_GREEN_SIZE: SDL_GLattr = 1;
pub const SDL_GL_RED_SIZE: SDL_GLattr = 0;
pub const SDL_GL_CONTEXT_PROFILE_COMPATIBILITY: C2RustUnnamed = 2;
pub type Uint32 = uint32_t;
pub type Uint8 = uint8_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_version {
    pub major: Uint8,
    pub minor: Uint8,
    pub patch: Uint8,
}
pub type Signal = libc::c_int;
pub type C2RustUnnamed = libc::c_uint;
pub const SDL_GL_CONTEXT_PROFILE_ES: C2RustUnnamed = 4;
pub const SDL_GL_CONTEXT_PROFILE_CORE: C2RustUnnamed = 1;

#[no_mangle]
pub static mut subsystems: uint32 = 0x4000 as libc::c_uint | 0x20 as libc::c_uint
    | 0x1 as libc::c_uint | 0x1000 as libc::c_uint | 0x200 as libc::c_uint
    | 0x2000 as libc::c_uint;
static mut versionString: cstr = b"Feb 12 2023 23:20:35\0" as *const u8
    as *const libc::c_char;
static mut initTime: TimeStamp = 0 as libc::c_int as TimeStamp;
#[no_mangle]
pub unsafe extern "C" fn Engine_Init(
    mut glVersionMajor: libc::c_int,
    mut glVersionMinor: libc::c_int,
) {
    static mut firstTime: bool = 1 as libc::c_int != 0;
    Signal_Init();
    printf(
        b"Engine_Init: Requesting GL %d.%d\n\0" as *const u8 as *const libc::c_char,
        glVersionMajor,
        glVersionMinor,
    );
    if firstTime {
        firstTime = 0 as libc::c_int != 0;
        let mut compiled: SDL_version = SDL_version {
            major: 0,
            minor: 0,
            patch: 0,
        };
        let mut linked: SDL_version = SDL_version {
            major: 0,
            minor: 0,
            patch: 0,
        };
        compiled.major = 2 as libc::c_int as Uint8;
        compiled.minor = 26 as libc::c_int as Uint8;
        compiled.patch = 1 as libc::c_int as Uint8;
        SDL_GetVersion(&mut linked);
        if compiled.major as libc::c_int != linked.major as libc::c_int {
            puts(
                b"Engine_Init: Detected SDL major version mismatch:\0" as *const u8
                    as *const libc::c_char,
            );
            printf(
                b"  Version (Compiled) : %d.%d.%d\n\0" as *const u8
                    as *const libc::c_char,
                compiled.major as libc::c_int,
                compiled.minor as libc::c_int,
                compiled.patch as libc::c_int,
            );
            printf(
                b"  Version (Linked)   : %d.%d.%d\n\0" as *const u8
                    as *const libc::c_char,
                linked.major as libc::c_int,
                linked.minor as libc::c_int,
                linked.patch as libc::c_int,
            );
            Fatal(b"Engine_Init: Terminating.\0" as *const u8 as *const libc::c_char);
        }
        if SDL_Init(0 as libc::c_int as Uint32) != 0 as libc::c_int {
            Fatal(
                b"Engine_Init: Failed to initialize SDL\0" as *const u8
                    as *const libc::c_char,
            );
        }
        if !Directory_Create(b"log\0" as *const u8 as *const libc::c_char) {
            Fatal(
                b"Engine_Init: Failed to create log directory.\0" as *const u8
                    as *const libc::c_char,
            );
        }
        atexit(Some(SDL_Quit as unsafe extern "C" fn() -> ()));
    }
    if SDL_InitSubSystem(subsystems) != 0 as libc::c_int {
        Fatal(
            b"Engine_Init: Failed to initialize SDL's subsystems\0" as *const u8
                as *const libc::c_char,
        );
    }
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, glVersionMajor);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, glVersionMinor);
    SDL_GL_SetAttribute(
        SDL_GL_CONTEXT_PROFILE_MASK,
        SDL_GL_CONTEXT_PROFILE_COMPATIBILITY as libc::c_int,
    );
    SDL_GL_SetAttribute(SDL_GL_ACCELERATED_VISUAL, 1 as libc::c_int);
    SDL_GL_SetAttribute(SDL_GL_RED_SIZE, 8 as libc::c_int);
    SDL_GL_SetAttribute(SDL_GL_GREEN_SIZE, 8 as libc::c_int);
    SDL_GL_SetAttribute(SDL_GL_BLUE_SIZE, 8 as libc::c_int);
    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1 as libc::c_int);
    SDL_GL_SetAttribute(SDL_GL_DEPTH_SIZE, 24 as libc::c_int);
    Keyboard_Init();
    Metric_Reset();
    Mouse_Init();
    Input_Init();
    Resource_Init();
    ShaderVar_Init();
    initTime = TimeStamp_Get();
}
#[no_mangle]
pub unsafe extern "C" fn Engine_Free() {
    ShaderVar_Free();
    Keyboard_Free();
    Mouse_Free();
    Input_Free();
    Signal_Free();
    SDL_QuitSubSystem(subsystems);
}
#[no_mangle]
pub unsafe extern "C" fn Engine_Abort() {
    abort();
}
#[no_mangle]
pub unsafe extern "C" fn Engine_GetBits() -> libc::c_int {
    return (8 as libc::c_int as usize).wrapping_mul(::core::mem::size_of::<*mut libc::c_void>())
        as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Engine_GetTime() -> libc::c_double {
    return TimeStamp_GetElapsed(initTime);
}
#[no_mangle]
pub unsafe extern "C" fn Engine_GetVersion() -> cstr {
    return versionString;
}
#[no_mangle]
pub unsafe extern "C" fn Engine_IsInitialized() -> bool {
    return initTime != 0 as libc::c_ulonglong;
}
#[no_mangle]
pub unsafe extern "C" fn Engine_Terminate() {
    exit(0 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn Engine_Update() {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 14], &[libc::c_char; 14]>(b"Engine_Update\0"))
            .as_ptr(),
    );
    Metric_Reset();
    Keyboard_UpdatePre();
    Mouse_Update();
    Joystick_Update();
    Gamepad_Update();
    Input_Update();
    Keyboard_UpdatePost();
    Profiler_End();
}
