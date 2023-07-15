mod frame_state;

pub(crate) use frame_state::*;
use glam::ivec2;

use std::path::PathBuf;
use std::time::Instant;

use crate::common::*;
use crate::input::*;
use crate::internal::*;
use crate::logging::init_log;
use crate::lua::*;
use crate::render::*;
use crate::system::*;
use crate::window::Window;

use glam::DVec2;
use sdl2_sys::*;
use tracing::info;
use tracing::warn;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::EnvFilter;
use winit::event;
use winit::event::DeviceEvent;
use winit::event::Event;
use winit::event::StartCause;
use winit::event::WindowEvent;
use winit::event_loop::*;

#[no_mangle]
pub static subsystems: u32 = SDL_INIT_EVENTS
    | SDL_INIT_VIDEO
    | SDL_INIT_TIMER
    | SDL_INIT_HAPTIC
    | SDL_INIT_JOYSTICK
    | SDL_INIT_GAMECONTROLLER;

pub struct Engine {
    init_time: TimeStamp,
    window: Window,
    frame_state: FrameState,
}

impl Engine {
    fn new(gl_version_major: i32, gl_version_minor: i32) -> Self {
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
        }

        Self {
            init_time: TimeStamp::now(),
            window: Default::default(),
            frame_state: Default::default(),
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Engine {
    #[bind(lua_ffi = false)]
    pub fn entry(entry_point: &str, app_name: &str, console_log: bool, log_dir: &str) {
        // Keep log till the end of the execution
        let _log = init_log(console_log, log_dir);

        let mut engine = Engine::new(2, 1);

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
            // Lua_Free(lua);
        }

        // Engine::free();

        let event_loop = EventLoop::new();

        let mut finished_and_setup_done = false;

        let event_handler = move |event: Event<()>,
                                  event_loop: &EventLoopWindowTarget<()>,
                                  control_flow: &mut ControlFlow| {
            #[cfg(feature = "trace")]
            let _span = bevy_utils::tracing::info_span!("winit event_handler").entered();

            // if !finished_and_setup_done {
            //     if !app.ready() {
            //         #[cfg(not(target_arch = "wasm32"))]
            //         tick_global_task_pools_on_main_thread();
            //     } else {
            //         app.finish();
            //         app.cleanup();
            //         finished_and_setup_done = true;
            //     }
            // }

            // if let Some(app_exit_events) = app.world.get_resource::<Events<AppExit>>() {
            //     if app_exit_event_reader.iter(app_exit_events).last().is_some() {
            //         *control_flow = ControlFlow::Exit;
            //         return;
            //     }
            // }

            match event {
                event::Event::NewEvents(start) => {
                    // let (winit_config, window_focused_query) = focused_window_state.get(&app.world);

                    // let app_focused = window_focused_query.iter().any(|window| window.focused);

                    // // Check if either the `WaitUntil` timeout was triggered by winit, or that same
                    // // amount of time has elapsed since the last app update. This manual check is needed
                    // // because we don't know if the criteria for an app update were met until the end of
                    // // the frame.
                    // let auto_timeout_reached =
                    //     matches!(start, StartCause::ResumeTimeReached { .. });
                    // let now = Instant::now();
                    // let manual_timeout_reached = match winit_config.update_mode(app_focused) {
                    //     UpdateMode::Continuous => false,
                    //     UpdateMode::Reactive { max_wait }
                    //     | UpdateMode::ReactiveLowPower { max_wait } => {
                    //         now.duration_since(engine.frame_state.last_update) >= *max_wait
                    //     }
                    // };
                    // // The low_power_event state and timeout must be reset at the start of every frame.
                    engine.frame_state.low_power_event = false;
                    engine.frame_state.timeout_reached = false; //auto_timeout_reached || manual_timeout_reached;
                }
                event::Event::WindowEvent {
                    event,
                    window_id: winit_window_id,
                    ..
                } => {
                    // // Fetch and prepare details from the world
                    // let mut system_state: SystemState<(
                    //     NonSend<WinitWindows>,
                    //     Query<(&mut Window, &mut CachedWindow)>,
                    //     WindowEvents,
                    //     InputEvents,
                    //     CursorEvents,
                    //     EventWriter<FileDragAndDrop>,
                    // )> = SystemState::new(&mut app.world);
                    // let (
                    //     winit_windows,
                    //     mut window_query,
                    //     mut window_events,
                    //     mut input_events,
                    //     mut cursor_events,
                    //     mut file_drag_and_drop_events,
                    // ) = system_state.get_mut(&mut app.world);

                    // // Entity of this window
                    // let window_entity =
                    //     if let Some(entity) = winit_windows.get_window_entity(winit_window_id) {
                    //         entity
                    //     } else {
                    //         warn!(
                    //             "Skipped event {:?} for unknown winit Window Id {:?}",
                    //             event, winit_window_id
                    //         );
                    //         return;
                    //     };

                    // let (mut window, mut cache) =
                    //     if let Ok((window, info)) = window_query.get_mut(window_entity) {
                    //         (window, info)
                    //     } else {
                    //         warn!(
                    //             "Window {:?} is missing `Window` component, skipping event {:?}",
                    //             window_entity, event
                    //         );
                    //         return;
                    //     };

                    engine.frame_state.low_power_event = true;

                    match event {
                        WindowEvent::Resized(size) => {
                            engine
                                .window
                                .resolution
                                .set_physical_resolution(size.width, size.height);

                            // window_events.window_resized.send(WindowResized {
                            //     window: window_entity,
                            //     width: window.width(),
                            //     height: window.height(),
                            // });
                        }
                        WindowEvent::CloseRequested => {
                            // window_events
                            //     .window_close_requested
                            //     .send(WindowCloseRequested {
                            //         window: window_entity,
                            //     });
                        }
                        WindowEvent::KeyboardInput { ref input, .. } => {
                            // input_events
                            //     .keyboard_input
                            //     .send(converters::convert_keyboard_input(input, window_entity));
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            let physical_position = DVec2::new(position.x, position.y);

                            engine
                                .window
                                .set_physical_cursor_position(Some(physical_position));

                            // cursor_events.cursor_moved.send(CursorMoved {
                            //     window: window_entity,
                            //     position: (physical_position / window.resolution.scale_factor())
                            //         .as_vec2(),
                            // });
                        }
                        WindowEvent::CursorEntered { .. } => {
                            // cursor_events.cursor_entered.send(CursorEntered {
                            //     window: window_entity,
                            // });
                        }
                        WindowEvent::CursorLeft { .. } => {
                            engine.window.set_physical_cursor_position(Option::None);

                            // cursor_events.cursor_left.send(CursorLeft {
                            //     window: window_entity,
                            // });
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            // input_events.mouse_button_input.send(MouseButtonInput {
                            //     button: converters::convert_mouse_button(button),
                            //     state: converters::convert_element_state(state),
                            //     window: window_entity,
                            // });
                        }
                        WindowEvent::TouchpadMagnify { delta, .. } => {
                            // input_events
                            //     .touchpad_magnify_input
                            //     .send(TouchpadMagnify(delta as f32));
                        }
                        WindowEvent::TouchpadRotate { delta, .. } => {
                            // input_events
                            //     .touchpad_rotate_input
                            //     .send(TouchpadRotate(delta));
                        }
                        WindowEvent::MouseWheel { delta, .. } => match delta {
                            event::MouseScrollDelta::LineDelta(x, y) => {
                                // input_events.mouse_wheel_input.send(MouseWheel {
                                //     unit: MouseScrollUnit::Line,
                                //     x,
                                //     y,
                                //     window: window_entity,
                                // });
                            }
                            event::MouseScrollDelta::PixelDelta(p) => {
                                // input_events.mouse_wheel_input.send(MouseWheel {
                                //     unit: MouseScrollUnit::Pixel,
                                //     x: p.x as f32,
                                //     y: p.y as f32,
                                //     window: window_entity,
                                // });
                            }
                        },
                        WindowEvent::Touch(touch) => {
                            // let location =
                            //     touch.location.to_logical(window.resolution.scale_factor());

                            // // Event
                            // input_events
                            //     .touch_input
                            //     .send(converters::convert_touch_input(touch, location));
                        }
                        WindowEvent::ReceivedCharacter(c) => {
                            // input_events.character_input.send(ReceivedCharacter {
                            //     window: window_entity,
                            //     char: c,
                            // });
                        }
                        WindowEvent::ScaleFactorChanged {
                            scale_factor,
                            new_inner_size,
                        } => {
                            // window_events.window_backend_scale_factor_changed.send(
                            //     WindowBackendScaleFactorChanged {
                            //         window: window_entity,
                            //         scale_factor,
                            //     },
                            // );

                            // let prior_factor = window.resolution.scale_factor();
                            // window.resolution.set_scale_factor(scale_factor);
                            // let new_factor = window.resolution.scale_factor();

                            // if let Some(forced_factor) = window.resolution.scale_factor_override() {
                            //     // If there is a scale factor override, then force that to be used
                            //     // Otherwise, use the OS suggested size
                            //     // We have already told the OS about our resize constraints, so
                            //     // the new_inner_size should take those into account
                            //     *new_inner_size =
                            //         winit::dpi::LogicalSize::new(window.width(), window.height())
                            //             .to_physical::<u32>(forced_factor);
                            //     // TODO: Should this not trigger a WindowsScaleFactorChanged?
                            // } else if approx::relative_ne!(new_factor, prior_factor) {
                            //     // Trigger a change event if they are approximately different
                            //     window_events.window_scale_factor_changed.send(
                            //         WindowScaleFactorChanged {
                            //             window: window_entity,
                            //             scale_factor,
                            //         },
                            //     );
                            // }

                            // let new_logical_width =
                            //     (new_inner_size.width as f64 / new_factor) as f32;
                            // let new_logical_height =
                            //     (new_inner_size.height as f64 / new_factor) as f32;
                            // if approx::relative_ne!(window.width(), new_logical_width)
                            //     || approx::relative_ne!(window.height(), new_logical_height)
                            // {
                            //     window_events.window_resized.send(WindowResized {
                            //         window: window_entity,
                            //         width: new_logical_width,
                            //         height: new_logical_height,
                            //     });
                            // }
                            // window.resolution.set_physical_resolution(
                            //     new_inner_size.width,
                            //     new_inner_size.height,
                            // );
                        }
                        WindowEvent::Focused(focused) => {
                            // Component
                            engine.window.focused = focused;

                            // window_events.window_focused.send(WindowFocused {
                            //     window: window_entity,
                            //     focused,
                            // });
                        }
                        WindowEvent::DroppedFile(path_buf) => {
                            // file_drag_and_drop_events.send(FileDragAndDrop::DroppedFile {
                            //     window: window_entity,
                            //     path_buf,
                            // });
                        }
                        WindowEvent::HoveredFile(path_buf) => {
                            // file_drag_and_drop_events.send(FileDragAndDrop::HoveredFile {
                            //     window: window_entity,
                            //     path_buf,
                            // });
                        }
                        WindowEvent::HoveredFileCancelled => {
                            // file_drag_and_drop_events.send(FileDragAndDrop::HoveredFileCanceled {
                            //     window: window_entity,
                            // });
                        }
                        WindowEvent::Moved(position) => {
                            let position = ivec2(position.x, position.y);

                            engine.window.position.set(position);

                            // window_events.window_moved.send(WindowMoved {
                            //     entity: window_entity,
                            //     position,
                            // });
                        }
                        WindowEvent::Ime(event) => match event {
                            event::Ime::Preedit(value, cursor) => {
                                // input_events.ime_input.send(Ime::Preedit {
                                //     window: window_entity,
                                //     value,
                                //     cursor,
                                // });
                            }
                            event::Ime::Commit(value) => {
                                // input_events.ime_input.send(Ime::Commit {
                                //                             window: window_entity,
                                //                             value,
                                //                         })
                            }
                            event::Ime::Enabled => {
                                // input_events.ime_input.send(Ime::Enabled {
                                //                             window: window_entity,
                                //                         })
                            }
                            event::Ime::Disabled => {
                                // input_events.ime_input.send(Ime::Disabled {
                                //                             window: window_entity,
                                //                         })
                            }
                        },
                        WindowEvent::ThemeChanged(theme) => {
                            // window_events.window_theme_changed.send(WindowThemeChanged {
                            //     window: window_entity,
                            //     theme: convert_winit_theme(theme),
                            // });
                        }
                        WindowEvent::Destroyed => {
                            // window_events.window_destroyed.send(WindowDestroyed {
                            //     window: window_entity,
                            // });
                        }
                        _ => {}
                    }

                    // if engine.window.is_changed() {
                    //     cache.window = window.clone();
                    // }
                }
                event::Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta: (x, y) },
                    ..
                } => {
                    // let mut system_state: SystemState<EventWriter<MouseMotion>> =
                    //     SystemState::new(&mut app.world);
                    // let mut mouse_motion = system_state.get_mut(&mut app.world);

                    // mouse_motion.send(MouseMotion {
                    //     delta: Vec2::new(x as f32, y as f32),
                    // });
                }
                event::Event::Suspended => {
                    engine.frame_state.active = false;
                }
                event::Event::Resumed => {
                    engine.frame_state.active = true;
                }
                event::Event::MainEventsCleared => {
                    if finished_and_setup_done {
                        engine.frame_state.last_update = Instant::now();
                        // app.update();
                        // TODO: call Lua AppFrame() function
                        // TODO: clear all events
                    }
                }
                Event::RedrawEventsCleared => {
                    *control_flow = ControlFlow::Poll;
                    engine.frame_state.redraw_request_sent = true;
                    // {
                    //     // Fetch from world
                    //     let (winit_config, window_focused_query) =
                    //         focused_window_state.get(&app.world);

                    //     // True if _any_ windows are currently being focused
                    //     let app_focused = window_focused_query.iter().any(|window| window.focused);

                    //     let now = Instant::now();
                    //     use UpdateMode::*;
                    //     *control_flow = match winit_config.update_mode(app_focused) {
                    //         Continuous => ControlFlow::Poll,
                    //         Reactive { max_wait } | ReactiveLowPower { max_wait } => {
                    //             if let Some(instant) = now.checked_add(*max_wait) {
                    //                 ControlFlow::WaitUntil(instant)
                    //             } else {
                    //                 ControlFlow::Wait
                    //             }
                    //         }
                    //     };
                    // }

                    // // This block needs to run after `app.update()` in `MainEventsCleared`. Otherwise,
                    // // we won't be able to see redraw requests until the next event, defeating the
                    // // purpose of a redraw request!
                    // let mut redraw = false;
                    // if let Some(app_redraw_events) =
                    //     app.world.get_resource::<Events<RequestRedraw>>()
                    // {
                    //     if redraw_event_reader.iter(app_redraw_events).last().is_some() {
                    //         *control_flow = ControlFlow::Poll;
                    //         redraw = true;
                    //     }
                    // }

                    // engine.frame_state.redraw_request_sent = redraw;
                }

                _ => (),
            }

            // if engine.frame_state.active {
            //     #[cfg(not(target_arch = "wasm32"))]
            //     let (
            //         commands,
            //         mut new_windows,
            //         created_window_writer,
            //         winit_windows,
            //         adapters,
            //         handlers,
            //         accessibility_requested,
            //     ) = create_window_system_state.get_mut(&mut app.world);

            //     #[cfg(target_arch = "wasm32")]
            //     let (
            //         commands,
            //         mut new_windows,
            //         created_window_writer,
            //         winit_windows,
            //         adapters,
            //         handlers,
            //         accessibility_requested,
            //         canvas_parent_resize_channel,
            //     ) = create_window_system_state.get_mut(&mut app.world);

            //     // Responsible for creating new windows
            //     create_window(
            //         commands,
            //         event_loop,
            //         new_windows.iter_mut(),
            //         created_window_writer,
            //         winit_windows,
            //         adapters,
            //         handlers,
            //         accessibility_requested,
            //         #[cfg(target_arch = "wasm32")]
            //         canvas_parent_resize_channel,
            //     );

            //     create_window_system_state.apply(&mut app.world);
            // }
        };

        event_loop.run(event_handler);
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
        std::process::abort();
    }

    pub fn get_bits() -> i32 {
        8_usize.wrapping_mul(std::mem::size_of::<*mut libc::c_void>()) as i32
    }

    pub fn get_time(&self) -> f64 {
        self.init_time.get_elapsed()
    }

    pub fn get_version() -> &'static str {
        // TODO: think about string conversion to C string without pinning if necessary
        env!("PHX_VERSION")
    }

    pub fn terminate() {
        std::process::exit(0);
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
