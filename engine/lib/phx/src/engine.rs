use std::path::PathBuf;

use crate::common::*;
use crate::input::*;
use crate::internal::*;
use crate::logging::init_log;
use crate::lua::*;
use crate::render::*;
use crate::system::*;

use sdl2_sys::*;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::EnvFilter;

#[no_mangle]
pub static subsystems: u32 = SDL_INIT_EVENTS
    | SDL_INIT_VIDEO
    | SDL_INIT_TIMER
    | SDL_INIT_HAPTIC
    | SDL_INIT_JOYSTICK
    | SDL_INIT_GAMECONTROLLER;

static mut initTime: TimeStamp = TimeStamp::zero();

pub struct Engine;

impl Engine {
    fn init(gl_version_major: i32, gl_version_minor: i32) {
        unsafe {
            static mut firstTime: bool = true;
            Signal_Init();

            info!("Engine_Init: Requesting GL {gl_version_major}.{gl_version_minor}");

            if firstTime {
                firstTime = false;

                /* Check SDL version compatibility. */
                let compiled: SDL_version = SDL_version {
                    major: SDL_MAJOR_VERSION as u8,
                    minor: SDL_MINOR_VERSION as u8,
                    patch: SDL_PATCHLEVEL as u8,
                };
                let mut linked: SDL_version = SDL_version {
                    major: 0,
                    minor: 0,
                    patch: 0,
                };

                SDL_GetVersion(&mut linked);
                if compiled.major != linked.major {
                    info!("Engine_Init: Detected SDL major version mismatch:");
                    info!(
                        "  Version (Compiled) : {}.{}.{}",
                        compiled.major, compiled.minor, compiled.patch,
                    );
                    info!(
                        "  Version (Linked)   : {}.{}.{}",
                        linked.major, linked.minor, linked.patch,
                    );
                    panic!("Engine_Init: Terminating.");
                }

                if SDL_Init(0) != 0 {
                    panic!("Engine_Init: Failed to initialize SDL");
                }
                if !Directory_Create(c_str!("log")) {
                    panic!("Engine_Init: Failed to create log directory.");
                }
                atexit(Some(SDL_Quit as unsafe extern "C" fn() -> ()));
            }

            if SDL_InitSubSystem(subsystems) != 0 {
                panic!("Engine_Init: Failed to initialize SDL's subsystems");
            }

            SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_CONTEXT_MAJOR_VERSION, gl_version_major);
            SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_CONTEXT_MINOR_VERSION, gl_version_minor);
            SDL_GL_SetAttribute(
                SDL_GLattr::SDL_GL_CONTEXT_PROFILE_MASK,
                SDL_GLprofile::SDL_GL_CONTEXT_PROFILE_COMPATIBILITY as i32,
            );
            SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_ACCELERATED_VISUAL, 1);
            SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_RED_SIZE, 8);
            SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_GREEN_SIZE, 8);
            SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_BLUE_SIZE, 8);
            SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_DOUBLEBUFFER, 1);
            SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_DEPTH_SIZE, 24);

            Keyboard_Init();
            Metric_Reset();
            Mouse_Init();
            Input_Init();
            Resource_Init();
            ShaderVar_Init();

            initTime = TimeStamp::now();
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Engine {
    #[bind(lua_ffi = false)]
    pub fn entry(entry_point: &str, app_name: &str, console_log: bool, log_dir: &str) {
        // Keep log till the end of the execution
        let _log = init_log(console_log, log_dir);

        Engine::init(2, 1);

        let entry_point_path = PathBuf::new().join(entry_point);

        if !entry_point_path.exists() {
            // TODO: do we really need this magic?
            std::env::set_current_dir("../").expect("Cannot change folder to parent");

            if !entry_point_path.exists() {
                panic!("Can't find script entrypoint: {entry_point}");
            }
        }

        unsafe {
            let lua = Lua_Create();

            Lua_SetBool(lua, c_str!("__debug__"), cfg!(debug_assertions));
            Lua_SetBool(lua, c_str!("__embedded__"), true);
            Lua_SetNumber(lua, c_str!("__checklevel__"), 0 as f64);

            if !app_name.is_empty() {
                let an = static_string!(app_name);

                Lua_SetStr(lua, c_str!("__app__"), an);
            }

            let script_file = static_string!(entry_point);

            Lua_DoFile(lua, script_file);
            Lua_Free(lua);
        }

        Engine::free();
    }

    pub fn free() {
        unsafe {
            ShaderVar_Free();
            Keyboard_Free();
            Mouse_Free();
            Input_Free();
            Signal_Free();
            SDL_QuitSubSystem(subsystems);
        }
    }

    pub fn abort() {
        unsafe {
            abort();
        }
    }

    pub fn get_bits() -> i32 {
        8_usize.wrapping_mul(std::mem::size_of::<*mut libc::c_void>()) as i32
    }

    pub fn get_time() -> f64 {
        unsafe { initTime.get_elapsed() }
    }

    pub fn get_version() -> &'static str {
        // TODO: think about string conversion to C string without pinning if necessary
        env!("PHX_VERSION")
    }

    pub fn is_initialized() -> bool {
        unsafe { initTime != TimeStamp::zero() }
    }

    pub fn terminate() {
        unsafe {
            exit(0);
        }
    }

    pub fn update() {
        unsafe {
            Profiler_Begin(c_str!("Engine_Update"));
            Metric_Reset();
            Keyboard_UpdatePre();
            Mouse_Update();
            Joystick_Update();
            Gamepad_Update();
            Input_Update();
            Keyboard_UpdatePost();
            Profiler_End();
        }
    }
}
