mod frame_state;

use std::path::PathBuf;
use std::time::Instant;

pub(crate) use frame_state::*;

use glam::*;
use mlua::{Function, Lua};
use tracing::*;
use winit::dpi::*;
use winit::event::Event;
use winit::event::{self, *};
use winit::event_loop::*;

use internal::ConvertIntoString;

use crate::common::*;
use crate::input::*;
use crate::logging::init_log;
use crate::render::*;
use crate::system::*;
use crate::ui::hmgui::HmGui;
use crate::window::*;

pub struct Engine {
    init_time: TimeStamp,
    window: Window,
    cache: CachedWindow,
    winit_windows: WinitWindows,
    winit_window_id: Option<winit::window::WindowId>,
    hmgui: HmGui,
    input: Input,
    frame_state: FrameState,
    exit_app: bool,
    lua: Lua,
}

impl Engine {
    fn new(gl_version_major: u8, gl_version_minor: u8) -> Self {
        unsafe {
            static mut FIRST_TIME: bool = true;
            Signal_Init();

            info!("Engine_Init: Requesting GL {gl_version_major}.{gl_version_minor}");

            if FIRST_TIME {
                FIRST_TIME = false;

                if !Directory_Create(c_str!("log")) {
                    panic!("Engine_Init: Failed to create log directory.");
                }
            }

            Metric_Reset();
            ShaderVar_Init();
        }

        let window = Window::default();
        let cache = CachedWindow {
            window: window.clone(),
        };

        // Unsafe is required for FFI and JIT libs
        let lua = unsafe { Lua::unsafe_new() };

        Self {
            init_time: TimeStamp::now(),
            window,
            cache,
            winit_windows: WinitWindows::new(gl_version_major, gl_version_minor),
            winit_window_id: None,
            hmgui: HmGui::new(),
            input: Default::default(),
            frame_state: Default::default(),
            exit_app: false,
            lua,
        }
    }

    fn init_winit_window(&mut self, event_loop: &EventLoop<()>) {
        debug!("Engine.init_winit_window");

        let winit_window_id = self.winit_windows.create_window(event_loop, &self.window);

        self.winit_window_id = Some(winit_window_id);
    }

    // Apply user changes, and then detect changes to the window and update the winit window accordingly.
    //
    // Notes:
    // - [`Window::transparent`] currently cannot be updated after startup for winit.
    // - [`Window::canvas`] currently cannot be updated after startup, not entirely sure if it would work well with the
    //   event channel stuff.
    fn changed_window(&mut self) {
        for user_change in self.input.user_changes() {
            match user_change {
                UserChange::CursorVisible(visible) => self.window.cursor.visible = *visible,
                UserChange::CursorPosition(x, y) => {
                    self.window.set_cursor_position(Some(Vec2::new(*x, *y)))
                }
            }
        }

        let Some(winit_window_wrapper) = self
            .winit_window_id
            .map(|winit_window_id| self.winit_windows.get_window_mut(winit_window_id))
            .flatten()
        else {
            return;
        };

        if let Some(state) = self.window.state {
            match state {
                WindowState::Suspended => winit_window_wrapper.suspend(),
                WindowState::Resumed => winit_window_wrapper.resume(),
            }

            self.window.state = None;
        }

        let winit_window = winit_window_wrapper.window();

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
            let width = self.window.resolution.physical_width();
            let height = self.window.resolution.physical_height();
            let physical_size = PhysicalSize::new(width, height);

            winit_window.set_inner_size(physical_size);
            winit_window_wrapper.resize(width, height);
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

        winit_window_wrapper.redraw();

        self.cache.window = self.window.clone();
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Engine {
    #[bind(lua_ffi = false)]
    pub fn entry(entry_point: &str, app_name: &str, console_log: bool, log_dir: &str) {
        let app_name = app_name.to_string();
        // Keep log till the end of the execution
        let _log = init_log(console_log, log_dir);

        let mut engine = Engine::new(2, 1);

        let entry_point_path = PathBuf::from(entry_point);

        if !entry_point_path.exists() {
            // TODO: do we really need this magic?
            std::env::set_current_dir("../").expect("Cannot change folder to parent");

            if !entry_point_path.exists() {
                panic!("Can't find script entrypoint: {entry_point}");
            }
        }

        let event_loop = EventLoop::new();

        engine.init_winit_window(&event_loop);

        // Apply window changes made by a script
        engine.changed_window();

        let finished_and_setup_done = true;

        let event_handler = move |event: Event<()>,
                                  _event_loop: &EventLoopWindowTarget<()>,
                                  control_flow: &mut ControlFlow| {
            if engine.exit_app {
                call_lua_func(&engine, "AppClose");

                control_flow.set_exit();
                return;
            }

            match event {
                event::Event::NewEvents(start_cause) => {
                    if start_cause == StartCause::Init {
                        let globals = engine.lua.globals();

                        globals.set("__debug__", cfg!(debug_assertions)).unwrap();
                        globals.set("__embedded__", true).unwrap();
                        globals.set("__checklevel__", 0 as u64).unwrap();

                        if !app_name.is_empty() {
                            globals.set("__app__", app_name.clone()).unwrap();
                        }

                        engine
                            .lua
                            .load(&*entry_point_path)
                            .exec()
                            .expect("Cannot execute entry point script");

                        let set_engine_func: Function = globals.get("SetEngine").unwrap();

                        set_engine_func
                            .call::<_, ()>(&engine as *const Engine as usize)
                            .unwrap();

                        let init_system_func: Function = globals.get("InitSystem").unwrap();
                        init_system_func.call::<_, ()>(()).unwrap();

                        let app_init_func: Function = globals.get("AppInit").unwrap();
                        app_init_func.call::<_, ()>(()).unwrap();
                    }

                    // The low_power_event state and timeout must be reset at the start of every frame.
                    engine.frame_state.low_power_event = false;
                    engine.frame_state.timeout_reached = false; //auto_timeout_reached || manual_timeout_reached;
                }
                event::Event::WindowEvent {
                    event,
                    window_id: _winit_window_id,
                    ..
                } => {
                    engine.frame_state.low_power_event = true;

                    match event {
                        WindowEvent::Resized(size) => {
                            engine
                                .window
                                .resolution
                                .set_physical_resolution(size.width, size.height);
                        }
                        WindowEvent::CloseRequested => {
                            call_lua_func(&engine, "AppClose");

                            control_flow.set_exit();
                        }
                        WindowEvent::KeyboardInput {
                            device_id,
                            ref input,
                            ..
                        } => {
                            // TODO: scancode?
                            if let Some(virtual_keycode) = input.virtual_keycode {
                                engine.input.update_keyboard(device_id, |state| {
                                    state.update(
                                        convert_virtual_key_code(virtual_keycode),
                                        input.state == ElementState::Pressed,
                                    )
                                });
                            }
                        }
                        WindowEvent::CursorMoved {
                            device_id,
                            position,
                            ..
                        } => {
                            engine.input.update_mouse(device_id, |state| {
                                state.update_position(position.x as f32, position.y as f32)
                            });
                        }
                        WindowEvent::CursorEntered { device_id } => {
                            engine
                                .input
                                .update_mouse(device_id, |state| state.update_in_window(true));
                        }
                        WindowEvent::CursorLeft { device_id } => {
                            engine.input.update_mouse(device_id, |state| {
                                state.update_in_window(false);
                                true
                            });
                        }
                        WindowEvent::MouseInput {
                            device_id,
                            state: elm_state,
                            button,
                            ..
                        } => {
                            let control = convert_mouse_button(button);

                            if let Some(control) = control {
                                engine.input.update_mouse(device_id, |state| {
                                    state.update_button(control, elm_state == ElementState::Pressed)
                                });
                            }
                        }
                        WindowEvent::MouseWheel {
                            device_id, delta, ..
                        } => match delta {
                            event::MouseScrollDelta::LineDelta(x, y) => {
                                engine.input.update_mouse(device_id, |state| {
                                    state.update_scroll_line(x, y)
                                });
                            }
                            event::MouseScrollDelta::PixelDelta(p) => {
                                engine.input.update_mouse(device_id, |state| {
                                    state.update_scroll_pixel(p.x as f32, p.y as f32)
                                });
                            }
                        },
                        WindowEvent::TouchpadMagnify {
                            device_id, delta, ..
                        } => {
                            engine.input.update_touchpad(device_id, |state| {
                                state.update_magnify_delta(delta as f32)
                            });
                        }
                        WindowEvent::TouchpadRotate {
                            device_id, delta, ..
                        } => {
                            engine.input.update_touchpad(device_id, |state| {
                                state.update_rotate_delta(delta)
                            });
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

                            engine.input.update_touchpad(touch.device_id, |state| {
                                state.update_position(x, y)
                            });
                        }
                        WindowEvent::ReceivedCharacter(_c) => {
                            // TODO: typing in the GUI?
                        }
                        WindowEvent::ScaleFactorChanged {
                            scale_factor: _,
                            new_inner_size: _,
                        } => {
                            // TODO: implement
                        }
                        WindowEvent::Focused(focused) => {
                            engine.window.focused = focused;
                        }
                        WindowEvent::DroppedFile(file) => {
                            engine
                                .input
                                .update_drag_and_drop(|state| state.update_dropped(&file));
                        }
                        WindowEvent::HoveredFile(file) => {
                            engine
                                .input
                                .update_drag_and_drop(|state| state.update_hovered(&file));
                        }
                        WindowEvent::HoveredFileCancelled => {
                            engine
                                .input
                                .update_drag_and_drop(|state| state.update_cancelled());
                        }
                        WindowEvent::Moved(position) => {
                            let position = ivec2(position.x, position.y);

                            engine.window.position.set(position);
                        }
                        WindowEvent::Ime(event) => match event {
                            event::Ime::Preedit(_value, _cursor) => {
                                // TODO: implement
                            }
                            event::Ime::Commit(_value) => {
                                // TODO: implement
                            }
                            event::Ime::Enabled => {
                                // TODO: implement
                            }
                            event::Ime::Disabled => {
                                // TODO: implement
                            }
                        },
                        WindowEvent::ThemeChanged(_theme) => {
                            // TODO: implement
                        }
                        WindowEvent::Destroyed => {
                            // TODO: implement?
                        }
                        _ => {
                            trace!("Unprocessed window event: {event:?}");
                        }
                    }
                }
                event::Event::Suspended => {
                    engine.frame_state.active = false;
                    engine.window.state = Some(WindowState::Suspended);
                }
                event::Event::Resumed => {
                    engine.frame_state.active = true;
                    engine.window.state = Some(WindowState::Resumed);
                }
                event::Event::MainEventsCleared => {
                    if finished_and_setup_done {
                        engine.frame_state.last_update = Instant::now();

                        // Load all gamepad events
                        engine.input.update_gamepad(|state| state.update());

                        // Let Lua script perform frame operations
                        call_lua_func(&engine, "AppFrame");

                        // Apply window changes made by a script
                        engine.changed_window();
                        engine.input.reset();
                    }
                }
                _ => {
                    trace!("Unprocessed event: {event:?}");
                }
            }
        };

        // Start event loop and never exit
        event_loop.run(event_handler);
    }

    pub fn window(&mut self) -> &mut Window {
        &mut self.window
    }

    pub fn input(&mut self) -> &mut Input {
        &mut self.input
    }

    #[bind(name = "HmGui")]
    pub fn hmgui(&mut self) -> &mut HmGui {
        &mut self.hmgui
    }

    pub fn free() {
        unsafe {
            ShaderVar_Free();
            Signal_Free();
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
        env!("PHX_VERSION")
    }

    pub fn exit(&mut self) {
        self.exit_app = true;
    }

    pub fn terminate() {
        std::process::exit(0);
    }

    pub fn update() {
        unsafe {
            Profiler_Begin(c_str!("Engine_Update"));
            Metric_Reset();
            Profiler_End();
        }
    }
}

fn call_lua_func(engine: &Engine, func_name: &str) {
    let globals = engine.lua.globals();
    let app_frame_func: Function = globals.get(func_name).unwrap();

    app_frame_func.call::<_, ()>(()).unwrap();
}
