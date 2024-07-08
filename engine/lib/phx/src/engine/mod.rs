mod main_loop;

use main_loop::*;

use glam::*;
use mlua::{Function, Lua};
use tracing::*;
use winit::dpi::*;
use winit::event_loop::*;
use std::path::PathBuf;

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
    winit_window: WinitWindow,
    hmgui: HmGui,
    input: Input,
    exit_app: bool,
    lua: Lua,
}

impl Engine {
    fn new(event_loop: &ActiveEventLoop) -> Self {
        unsafe {
            static mut FIRST_TIME: bool = true;
            Signal_Init();

            if FIRST_TIME {
                FIRST_TIME = false;
            }

            Metric_Reset();
            ShaderVar_Init();
        }

        // Unsafe is required for FFI and JIT libs
        let lua = unsafe { Lua::unsafe_new() };

        // Create window.
        let window = Window::default();
        let cache = CachedWindow {
            window: window.clone(),
        };
        let mut winit_window = WinitWindow::new(&event_loop, &window);
        winit_window.resume();
        let scale_factor = window.scale_factor();

        Self {
            init_time: TimeStamp::now(),
            window,
            cache,
            winit_window,
            hmgui: HmGui::new(scale_factor),
            input: Default::default(),
            exit_app: false,
            lua,
        }
    }

    // Apply user changes, and then detect changes to the window and update the winit window accordingly.
    fn changed_window(&mut self) {
        for user_change in self.input.user_changes() {
            match user_change {
                UserChange::CursorVisible(visible) => self.window.cursor.visible = *visible,
                UserChange::CursorPosition(x, y) => {
                    self.window.set_cursor_position(Some(Vec2::new(*x, *y)))
                }
            }
        }

        if let Some(state) = self.window.state {
            match state {
                WindowState::Suspended => self.winit_window.suspend(),
                WindowState::Resumed => self.winit_window.resume(),
            }

            self.window.state = None;
        }

        if self.window.title != self.cache.window.title {
            self.winit_window
                .window()
                .set_title(self.window.title.as_str());
        }

        if self.window.mode != self.cache.window.mode {
            let new_mode = match self.window.mode {
                WindowMode::BorderlessFullscreen => {
                    Some(winit::window::Fullscreen::Borderless(None))
                }
                WindowMode::Fullscreen => Some(winit::window::Fullscreen::Exclusive(
                    get_best_videomode(&self.winit_window.window().current_monitor().unwrap()),
                )),
                WindowMode::SizedFullscreen => {
                    Some(winit::window::Fullscreen::Exclusive(get_fitting_videomode(
                        &self.winit_window.window().current_monitor().unwrap(),
                        self.window.width() as u32,
                        self.window.height() as u32,
                    )))
                }
                WindowMode::Windowed => None,
            };

            if self.winit_window.window().fullscreen() != new_mode {
                self.winit_window.window().set_fullscreen(new_mode);
            }
        }

        if self.window.resolution != self.cache.window.resolution {
            let width = self.window.resolution.physical_width();
            let height = self.window.resolution.physical_height();
            let physical_size = PhysicalSize::new(width, height);

            // Try to resize the window.
            if let Some(new_size) = self.winit_window.window().request_inner_size(physical_size) {
                self.winit_window.resize(new_size.width, new_size.height);
            }
        }

        if self.window.physical_cursor_position() != self.cache.window.physical_cursor_position() {
            if let Some(physical_position) = self.window.physical_cursor_position() {
                let inner_size = self.winit_window.window().inner_size();

                let position = PhysicalPosition::new(
                    physical_position.x,
                    // Flip the coordinate space back to winit's context.
                    inner_size.height as f32 - physical_position.y,
                );

                if let Err(err) = self.winit_window.window().set_cursor_position(position) {
                    error!("could not set cursor position: {:?}", err);
                }
            }
        }

        if self.window.cursor.icon != self.cache.window.cursor.icon {
            self.winit_window
                .window()
                .set_cursor(convert_cursor_icon(self.window.cursor.icon));
        }

        if self.window.cursor.grab_mode != self.cache.window.cursor.grab_mode {
            attempt_grab(&self.winit_window.window(), self.window.cursor.grab_mode);
        }

        if self.window.cursor.visible != self.cache.window.cursor.visible {
            self.winit_window
                .window()
                .set_cursor_visible(self.window.cursor.visible);
        }

        if self.window.cursor.hit_test != self.cache.window.cursor.hit_test {
            if let Err(err) = self
                .winit_window
                .window()
                .set_cursor_hittest(self.window.cursor.hit_test)
            {
                self.window.cursor.hit_test = self.cache.window.cursor.hit_test;
                warn!(
                    "Could not set cursor hit test for window {:?}: {:?}",
                    self.window.title, err
                );
            }
        }

        if self.window.decorations != self.cache.window.decorations
            && self.window.decorations != self.winit_window.window().is_decorated()
        {
            self.winit_window
                .window()
                .set_decorations(self.window.decorations);
        }

        if self.window.resizable != self.cache.window.resizable
            && self.window.resizable != self.winit_window.window().is_resizable()
        {
            self.winit_window
                .window()
                .set_resizable(self.window.resizable);
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

            self.winit_window
                .window()
                .set_min_inner_size(Some(min_inner_size));
            if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
                self.winit_window
                    .window()
                    .set_max_inner_size(Some(max_inner_size));
            }
        }

        if self.window.position != self.cache.window.position {
            if let Some(position) = winit_window_position(
                &self.window.position,
                &self.window.resolution,
                self.winit_window.window().available_monitors(),
                self.winit_window.window().primary_monitor(),
                self.winit_window.window().current_monitor(),
            ) {
                let should_set = match self.winit_window.window().outer_position() {
                    Ok(current_position) => current_position != position,
                    _ => true,
                };

                if should_set {
                    self.winit_window.window().set_outer_position(position);
                }
            }
        }

        if let Some(maximized) = self.window.internal.take_maximize_request() {
            self.winit_window.window().set_maximized(maximized);
        }

        if let Some(minimized) = self.window.internal.take_minimize_request() {
            self.winit_window.window().set_minimized(minimized);
        }

        if self.window.focused != self.cache.window.focused && self.window.focused {
            self.winit_window.window().focus_window();
        }

        if self.window.present_mode != self.cache.window.present_mode {
            warn!("unable to change present mode after the window was created!");
            self.window.present_mode = self.cache.window.present_mode;
        }

        // Currently unsupported changes
        if self.window.ime_enabled != self.cache.window.ime_enabled {
            self.winit_window
                .window()
                .set_ime_allowed(self.window.ime_enabled);
        }

        if self.window.ime_position != self.cache.window.ime_position {
            // TODO: Set the IME cursor area correctly.
            let position =
                LogicalPosition::new(self.window.ime_position.x, self.window.ime_position.y);
            let width = self.window.resolution.physical_width();
            let height = self.window.resolution.physical_height();
            let physical_size = PhysicalSize::new(width, height);

            self.winit_window
                .window()
                .set_ime_cursor_area(position, physical_size);
        }

        if self.window.window_theme != self.cache.window.window_theme {
            self.winit_window
                .window()
                .set_theme(self.window.window_theme.map(convert_window_theme));
        }

        self.winit_window.redraw();

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

        let entry_point_path = PathBuf::from(entry_point);
        if !entry_point_path.exists() {
            // If we can't find it, set the current dir to one above the executable path and try that instead.
            let mut dir = std::env::current_exe().expect("Cannot get the path to the executable");
            dir.pop();
            dir.pop();
            debug!("Changing working directory to {:?}", dir);
            std::env::set_current_dir(dir).expect("Cannot change folder to parent");

            if !entry_point_path.exists() {
                panic!("Can't find script entrypoint: {entry_point}");
            }
        }

        let event_loop = EventLoop::new().expect("Failed to build event loop");
        let mut app_state = MainLoop {
            engine: None,
            app_name,
            entry_point_path,
            finished_and_setup_done: true,
        };
        let _ = event_loop.run_app(&mut app_state);
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

    // TODO: convert ShaderVar and Signal into the proper Rust types
    // pub fn free() {
    //     unsafe {
    //         ShaderVar_ee();
    //         Signal_Free();
    //     }
    // }

    pub fn abort() {
        std::process::abort();
    }

    pub fn get_bits() -> i32 {
        8_usize.wrapping_mul(std::mem::size_of::<*mut libc::c_void>()) as i32
    }

    /// Return time passed since engine start.
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
    let app_frame_func: Function = globals
        .get(func_name)
        .expect(format!("Unknown function {}", func_name).as_str());

    if let Err(e) = app_frame_func.call::<_, ()>(()) {
        trace!("{}", e);
    }
}
