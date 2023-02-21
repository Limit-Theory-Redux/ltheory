use ::libc;
use glam::Vec3;
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
    fn exit(_: i32) -> !;
    fn SDL_GL_SetAttribute(attr: SDL_GLattr, value: i32) -> i32;
    fn abort() -> !;
    fn SDL_QuitSubSystem(flags: u32);
    fn SDL_GetVersion(ver: *mut SDL_version);
    fn puts(_: *const libc::c_char) -> i32;
    fn printf(_: *const libc::c_char, _: ...) -> i32;
    fn SDL_Init(flags: u32) -> i32;
    fn atexit(_: Option::<unsafe extern "C" fn() -> ()>) -> i32;
    fn SDL_Quit();
    fn SDL_InitSubSystem(flags: u32) -> i32;
    fn ShaderVar_Init();
    fn ShaderVar_Free();
    fn TimeStamp_Get() -> TimeStamp;
    fn TimeStamp_GetElapsed(start: TimeStamp) -> f64;
}
pub type cstr = *const libc::c_char;
pub type ResourceType = i32;
pub type TimeStamp = u64;
pub type SDL_GLattr = u32;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDL_version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}
pub type Signal = i32;
pub type C2RustUnnamed = u32;
pub const SDL_GL_CONTEXT_PROFILE_ES: C2RustUnnamed = 4;
pub const SDL_GL_CONTEXT_PROFILE_CORE: C2RustUnnamed = 1;

#[no_mangle]
pub static mut subsystems: u32 = 0x4000 as u32 | 0x20 as u32
    | 0x1 as u32 | 0x1000 as u32 | 0x200 as u32
    | 0x2000 as u32;
static mut versionString: cstr = b"Feb 12 2023 23:20:35\0" as *const u8
    as *const libc::c_char;
static mut initTime: TimeStamp = 0 as i32 as TimeStamp;
#[no_mangle]
pub unsafe extern "C" fn Engine_Init(
    mut glVersionMajor: i32,
    mut glVersionMinor: i32,
) {
    static mut firstTime: bool = 1 as i32 != 0;
    Signal_Init();
    printf(
        b"Engine_Init: Requesting GL %d.%d\n\0" as *const u8 as *const libc::c_char,
        glVersionMajor,
        glVersionMinor,
    );
    if firstTime {
        firstTime = 0 as i32 != 0;
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
        compiled.major = 2 as i32 as u8;
        compiled.minor = 26 as i32 as u8;
        compiled.patch = 1 as i32 as u8;
        SDL_GetVersion(&mut linked);
        if compiled.major as i32 != linked.major as i32 {
            puts(
                b"Engine_Init: Detected SDL major version mismatch:\0" as *const u8
                    as *const libc::c_char,
            );
            printf(
                b"  Version (Compiled) : %d.%d.%d\n\0" as *const u8
                    as *const libc::c_char,
                compiled.major as i32,
                compiled.minor as i32,
                compiled.patch as i32,
            );
            printf(
                b"  Version (Linked)   : %d.%d.%d\n\0" as *const u8
                    as *const libc::c_char,
                linked.major as i32,
                linked.minor as i32,
                linked.patch as i32,
            );
            Fatal(b"Engine_Init: Terminating.\0" as *const u8 as *const libc::c_char);
        }
        if SDL_Init(0 as i32 as u32) != 0 as i32 {
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
    if SDL_InitSubSystem(subsystems) != 0 as i32 {
        Fatal(
            b"Engine_Init: Failed to initialize SDL's subsystems\0" as *const u8
                as *const libc::c_char,
        );
    }
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, glVersionMajor);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, glVersionMinor);
    SDL_GL_SetAttribute(
        SDL_GL_CONTEXT_PROFILE_MASK,
        SDL_GL_CONTEXT_PROFILE_COMPATIBILITY as i32,
    );
    SDL_GL_SetAttribute(SDL_GL_ACCELERATED_VISUAL, 1 as i32);
    SDL_GL_SetAttribute(SDL_GL_RED_SIZE, 8 as i32);
    SDL_GL_SetAttribute(SDL_GL_GREEN_SIZE, 8 as i32);
    SDL_GL_SetAttribute(SDL_GL_BLUE_SIZE, 8 as i32);
    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1 as i32);
    SDL_GL_SetAttribute(SDL_GL_DEPTH_SIZE, 24 as i32);
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
pub unsafe extern "C" fn Engine_GetBits() -> i32 {
    return (8 as usize).wrapping_mul(::core::mem::size_of::<*mut libc::c_void>())
        as i32;
}
#[no_mangle]
pub unsafe extern "C" fn Engine_GetTime() -> f64 {
    return TimeStamp_GetElapsed(initTime);
}
#[no_mangle]
pub unsafe extern "C" fn Engine_GetVersion() -> cstr {
    return versionString;
}
#[no_mangle]
pub unsafe extern "C" fn Engine_IsInitialized() -> bool {
    return initTime != 0 as u64;
}
#[no_mangle]
pub unsafe extern "C" fn Engine_Terminate() {
    exit(0 as i32);
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
