mod frame_state;

pub(crate) use frame_state::*;
use glam::ivec2;
use tracing::debug;
use tracing::error;
use tracing::trace;
use winit::dpi::*;
use winit::event::TouchPhase;

use std::path::PathBuf;
use std::time::Instant;

use crate::common::*;
// use crate::input::*;
use crate::input2::*;
use crate::internal::*;
use crate::logging::init_log;
use crate::lua::*;
use crate::render::*;
use crate::system::*;
use crate::window::*;

use glam::DVec2;
use tracing::{info, warn};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter};
use winit::event::{self, *};
use winit::event_loop::*;

pub struct Engine {
    init_time: TimeStamp,
    window: Window,
    cache: CachedWindow,
    winit_windows: WinitWindows,
    winit_window_id: Option<winit::window::WindowId>,
    input: Input2,
    frame_state: FrameState,
    exit_app: bool,
}

impl Engine {
    fn new(gl_version_major: i32, gl_version_minor: i32) -> Self {
        unsafe {
            static mut firstTime: bool = true;
            Signal_Init();

            info!("Engine_Init: Requesting GL {gl_version_major}.{gl_version_minor}");

            if firstTime {
                firstTime = false;

                if !Directory_Create(c_str!("log")) {
                    panic!("Engine_Init: Failed to create log directory.");
                }

                // /* Check SDL version compatibility. */
                // let compiled: SDL_version = SDL_version {
                //     major: SDL_MAJOR_VERSION as u8,
                //     minor: SDL_MINOR_VERSION as u8,
                //     patch: SDL_PATCHLEVEL as u8,
                // };
                // let mut linked: SDL_version = SDL_version {
                //     major: 0,
                //     minor: 0,
                //     patch: 0,
                // };

                // SDL_GetVersion(&mut linked);
                // if compiled.major != linked.major {
                //     info!("Engine_Init: Detected SDL major version mismatch:");
                //     info!(
                //         "  Version (Compiled) : {}.{}.{}",
                //         compiled.major, compiled.minor, compiled.patch,
                //     );
                //     info!(
                //         "  Version (Linked)   : {}.{}.{}",
                //         linked.major, linked.minor, linked.patch,
                //     );
                //     panic!("Engine_Init: Terminating.");
                // }

                // if SDL_Init(0) != 0 {
                //     panic!("Engine_Init: Failed to initialize SDL");
                // }
                // atexit(Some(SDL_Quit as unsafe extern "C" fn() -> ()));
            }

            // if SDL_InitSubSystem(subsystems) != 0 {
            //     panic!("Engine_Init: Failed to initialize SDL's subsystems");
            // }

            // SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_CONTEXT_MAJOR_VERSION, gl_version_major);
            // SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_CONTEXT_MINOR_VERSION, gl_version_minor);
            // SDL_GL_SetAttribute(
            //     SDL_GLattr::SDL_GL_CONTEXT_PROFILE_MASK,
            //     SDL_GLprofile::SDL_GL_CONTEXT_PROFILE_COMPATIBILITY as i32,
            // );
            // SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_ACCELERATED_VISUAL, 1);
            // SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_RED_SIZE, 8);
            // SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_GREEN_SIZE, 8);
            // SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_BLUE_SIZE, 8);
            // SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_DOUBLEBUFFER, 1);
            // SDL_GL_SetAttribute(SDL_GLattr::SDL_GL_DEPTH_SIZE, 24);

            // Keyboard_Init();
            Metric_Reset();
            // Mouse_Init();
            // Input_Init();
            Resource_Init();
            ShaderVar_Init();
        }

        let window = Window::default();
        let cache = CachedWindow {
            window: window.clone(),
        };

        Self {
            init_time: TimeStamp::now(),
            window,
            cache,
            winit_windows: Default::default(),
            winit_window_id: None,
            input: Default::default(),
            frame_state: Default::default(),
            exit_app: false,
        }
    }

    fn init_winit_window(&mut self, event_loop: &EventLoop<()>) {
        let winit_window_id = self.winit_windows.create_window(event_loop, &self.window);

        self.winit_window_id = Some(winit_window_id);
    }

    // Detect changes to the window and update the winit window accordingly.
    //
    // Notes:
    // - [`Window::present_mode`] and [`Window::composite_alpha_mode`] updating should be handled in the bevy render crate.
    // - [`Window::transparent`] currently cannot be updated after startup for winit.
    // - [`Window::canvas`] currently cannot be updated after startup, not entirely sure if it would work well with the
    //   event channel stuff.
    fn changed_window(&mut self) {
        let Some(winit_window) = self.winit_window_id.map(|winit_window_id| self.winit_windows.get_window(winit_window_id)).flatten()
        else { return; };

        if self.window.title != self.cache.window.title {
            winit_window.set_title(self.window.title.as_str());
        }

        if self.window.mode != self.cache.window.mode {
            let new_mode = match self.window.mode {
                WindowMode::BorderlessFullscreen => {
                    Some(winit::window::Fullscreen::Borderless(None))
                }
                WindowMode::Fullscreen => Some(winit::window::Fullscreen::Exclusive(
                    get_best_videomode(&winit_window.current_monitor().unwrap()),
                )),
                WindowMode::SizedFullscreen => {
                    Some(winit::window::Fullscreen::Exclusive(get_fitting_videomode(
                        &winit_window.current_monitor().unwrap(),
                        self.window.width() as u32,
                        self.window.height() as u32,
                    )))
                }
                WindowMode::Windowed => None,
            };

            if winit_window.fullscreen() != new_mode {
                winit_window.set_fullscreen(new_mode);
            }
        }
        if self.window.resolution != self.cache.window.resolution {
            let physical_size = PhysicalSize::new(
                self.window.resolution.physical_width(),
                self.window.resolution.physical_height(),
            );

            winit_window.set_inner_size(physical_size);
        }

        if self.window.physical_cursor_position() != self.cache.window.physical_cursor_position() {
            if let Some(physical_position) = self.window.physical_cursor_position() {
                let inner_size = winit_window.inner_size();

                let position = PhysicalPosition::new(
                    physical_position.x,
                    // Flip the coordinate space back to winit's context.
                    inner_size.height as f32 - physical_position.y,
                );

                if let Err(err) = winit_window.set_cursor_position(position) {
                    error!("could not set cursor position: {:?}", err);
                }
            }
        }

        if self.window.cursor.icon != self.cache.window.cursor.icon {
            winit_window.set_cursor_icon(convert_cursor_icon(self.window.cursor.icon));
        }

        if self.window.cursor.grab_mode != self.cache.window.cursor.grab_mode {
            attempt_grab(&winit_window, self.window.cursor.grab_mode);
        }

        if self.window.cursor.visible != self.cache.window.cursor.visible {
            winit_window.set_cursor_visible(self.window.cursor.visible);
        }

        if self.window.cursor.hit_test != self.cache.window.cursor.hit_test {
            if let Err(err) = winit_window.set_cursor_hittest(self.window.cursor.hit_test) {
                self.window.cursor.hit_test = self.cache.window.cursor.hit_test;
                warn!(
                    "Could not set cursor hit test for window {:?}: {:?}",
                    self.window.title, err
                );
            }
        }

        if self.window.decorations != self.cache.window.decorations
            && self.window.decorations != winit_window.is_decorated()
        {
            winit_window.set_decorations(self.window.decorations);
        }

        if self.window.resizable != self.cache.window.resizable
            && self.window.resizable != winit_window.is_resizable()
        {
            winit_window.set_resizable(self.window.resizable);
        }

        if self.window.resize_constraints != self.cache.window.resize_constraints {
            let constraints = self.window.resize_constraints.check_constraints();
            let min_inner_size = LogicalSize {
                width: constraints.min_width,
                height: constraints.min_height,
            };
            let max_inner_size = LogicalSize {
                width: constraints.max_width,
                height: constraints.max_height,
            };

            winit_window.set_min_inner_size(Some(min_inner_size));
            if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
                winit_window.set_max_inner_size(Some(max_inner_size));
            }
        }

        if self.window.position != self.cache.window.position {
            if let Some(position) = winit_window_position(
                &self.window.position,
                &self.window.resolution,
                winit_window.available_monitors(),
                winit_window.primary_monitor(),
                winit_window.current_monitor(),
            ) {
                let should_set = match winit_window.outer_position() {
                    Ok(current_position) => current_position != position,
                    _ => true,
                };

                if should_set {
                    winit_window.set_outer_position(position);
                }
            }
        }

        if let Some(maximized) = self.window.internal.take_maximize_request() {
            winit_window.set_maximized(maximized);
        }

        if let Some(minimized) = self.window.internal.take_minimize_request() {
            winit_window.set_minimized(minimized);
        }

        if self.window.focused != self.cache.window.focused && self.window.focused {
            winit_window.focus_window();
        }

        if self.window.window_level != self.cache.window.window_level {
            winit_window.set_window_level(convert_window_level(self.window.window_level));
        }

        // Currently unsupported changes
        if self.window.transparent != self.cache.window.transparent {
            self.window.transparent = self.cache.window.transparent;
            warn!("Winit does not currently support updating transparency after window creation.");
        }

        if self.window.ime_enabled != self.cache.window.ime_enabled {
            winit_window.set_ime_allowed(self.window.ime_enabled);
        }

        if self.window.ime_position != self.cache.window.ime_position {
            winit_window.set_ime_position(LogicalPosition::new(
                self.window.ime_position.x,
                self.window.ime_position.y,
            ));
        }

        if self.window.window_theme != self.cache.window.window_theme {
            winit_window.set_theme(self.window.window_theme.map(convert_window_theme));
        }

        self.cache.window = self.window.clone();
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

            // TODO: call AppInit(engine)
        }

        // Engine::free();

        let event_loop = EventLoop::new();

        engine.init_winit_window(&event_loop);

        // Apply window changes made by a script
        engine.changed_window();

        let finished_and_setup_done = false;

        let event_handler = move |event: Event<()>,
                                  _event_loop: &EventLoopWindowTarget<()>,
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

            if engine.exit_app {
                *control_flow = ControlFlow::Exit;
                return;
            }

            match event {
                event::Event::NewEvents(_start) => {
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
                    window_id: _winit_window_id,
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
                            // TODO: scancode?
                            if let Some(virtual_keycode) = input.virtual_keycode {
                                engine.input.keyboard_state.update(
                                    convert_virtual_key_code(virtual_keycode),
                                    input.state == ElementState::Pressed,
                                );
                            }
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            let physical_position = DVec2::new(position.x, position.y);

                            engine
                                .window
                                .set_physical_cursor_position(Some(physical_position));

                            engine
                                .input
                                .cursor_state
                                .update_position(position.x as f32, position.y as f32);
                        }
                        WindowEvent::CursorEntered { .. } => {
                            engine.input.cursor_state.update_in_window(true);
                        }
                        WindowEvent::CursorLeft { .. } => {
                            engine.window.set_physical_cursor_position(None);

                            engine.input.cursor_state.update_in_window(false);
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            let control = convert_mouse_button(button);

                            if let Some(control) = control {
                                engine
                                    .input
                                    .mouse_state
                                    .update_button(control, state == ElementState::Pressed);
                            }
                        }
                        WindowEvent::MouseWheel { delta, .. } => match delta {
                            event::MouseScrollDelta::LineDelta(x, y) => {
                                engine
                                    .input
                                    .mouse_state
                                    .update_axis(MouseControl::ScrollLineX, x);
                                engine
                                    .input
                                    .mouse_state
                                    .update_axis(MouseControl::ScrollLineY, y);
                            }
                            event::MouseScrollDelta::PixelDelta(p) => {
                                engine
                                    .input
                                    .mouse_state
                                    .update_axis(MouseControl::ScrollPixelX, p.x as f32);
                                engine
                                    .input
                                    .mouse_state
                                    .update_axis(MouseControl::ScrollPixelY, p.y as f32);
                            }
                        },
                        WindowEvent::TouchpadMagnify { delta, .. } => {
                            engine
                                .input
                                .touchpad_state
                                .update(TouchpadAxis::MagnifyDelta, delta as f32);
                        }
                        WindowEvent::TouchpadRotate { delta, .. } => {
                            engine
                                .input
                                .touchpad_state
                                .update(TouchpadAxis::RotateDelta, delta);
                        }
                        WindowEvent::Touch(touch) => {
                            // TODO: expose more info from touch
                            let location = touch
                                .location
                                .to_logical(engine.window.resolution.scale_factor());
                            let (x, y) = if touch.phase == TouchPhase::Started
                                || touch.phase == TouchPhase::Moved
                            {
                                (location.x, location.x)
                            } else {
                                (-1.0, -1.0) // TODO: special value for no touch?
                            };

                            engine.input.touchpad_state.update(TouchpadAxis::X, x);
                            engine.input.touchpad_state.update(TouchpadAxis::Y, y);
                        }
                        WindowEvent::ReceivedCharacter(_c) => {
                            // input_events.character_input.send(ReceivedCharacter {
                            //     window: window_entity,
                            //     char: c,
                            // });
                        }
                        WindowEvent::ScaleFactorChanged {
                            scale_factor: _,
                            new_inner_size: _,
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
                            engine.window.focused = focused;

                            // window_events.window_focused.send(WindowFocused {
                            //     window: window_entity,
                            //     focused,
                            // });
                        }
                        WindowEvent::DroppedFile(file) => {
                            engine.input.drag_and_drop_state.update_dropped(file);
                        }
                        WindowEvent::HoveredFile(file) => {
                            engine.input.drag_and_drop_state.update_hovered(file);
                        }
                        WindowEvent::HoveredFileCancelled => {
                            engine.input.drag_and_drop_state.update_cancelled();
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
                            event::Ime::Preedit(_value, _cursor) => {
                                // input_events.ime_input.send(Ime::Preedit {
                                //     window: window_entity,
                                //     value,
                                //     cursor,
                                // });
                            }
                            event::Ime::Commit(_value) => {
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
                        WindowEvent::ThemeChanged(_theme) => {
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
                        _ => {
                            trace!("Unprocessed window event: {event:?}");
                        }
                    }

                    // if engine.window.is_changed() {
                    //     engine.cache.window = engine.window.clone();
                    // }
                }
                event::Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta: (x, y) },
                    ..
                } => {
                    engine
                        .input
                        .mouse_state
                        .update_axis(MouseControl::DeltaX, x as f32);
                    engine
                        .input
                        .mouse_state
                        .update_axis(MouseControl::DeltaY, y as f32);
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

                        // Load all gamepad events
                        engine.input.gamepad_mut().update();

                        // TODO: call Lua AppFrame() function

                        // Apply window changes made by a script
                        engine.changed_window();
                        engine.input.reset();
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

                _ => {
                    trace!("Unprocessed event: {event:?}");
                }
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

        // Start event loop and never exit
        event_loop.run(event_handler);
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn input(&self) -> &Input2 {
        &self.input
    }

    pub fn free() {
        unsafe {
            ShaderVar_Free();
            // Keyboard_Free();
            // Mouse_Free();
            // Input_Free();
            Signal_Free();
            // SDL_QuitSubSystem(subsystems);
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
            // Keyboard_UpdatePre();
            // Mouse_Update();
            // Joystick_Update();
            // Gamepad_Update();
            // Input_Update();
            // Keyboard_UpdatePost();
            Profiler_End();
        }
    }
}
