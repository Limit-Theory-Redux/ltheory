use std::cell::RefCell;

use glam::*;
use internal::EngineSettings;
use mlua::{Function, Lua};
use tracing::*;
use winit::dpi::*;
use winit::event_loop::*;

use super::{EventBus, MainLoop, TaskQueue};
use crate::input::*;
use crate::logging::init_log;
use crate::render::Renderer;
use crate::rf::*;
use crate::system::*;
use crate::ui::hmgui::HmGui;
use crate::window::*;

pub struct Engine {
    pub init_time: TimeStamp,
    pub window: Window,
    pub cache: CachedWindow,
    pub winit_window: WinitWindow,
    pub hmgui: HmGui,
    pub input: Input,
    pub exit_app: bool,
    pub event_bus: EventBus,
    pub task_queue: TaskQueue,
    pub renderer: Renderer,
    pub lua: Rf<Lua>,
    /// Set from `settings.present_mode` (the `ltr --present-mode` flag),
    /// which forces the swap interval regardless of what Lua requests via
    /// `Window:setPresentMode`. `None` means scripts control it normally.
    present_mode_override: Option<PresentMode>,
    /// Whether `changed_window` has already logged that a script's present
    /// mode request was overridden - logged once, not every frame.
    present_mode_override_logged: bool,
}

// This thread local variable contains a ref counted instance of the current Lua VM.
// This is used by the panic hook to tell the Lua VM to generate backtrace.
thread_local! {
    static CURRENT_LUA_CTX: RefCell<Option<Rf<Lua>>> = const { RefCell::new(None) };
}

#[cfg(target_os = "windows")]
fn build_event_loop() -> EventLoop<()> {
    use winit::platform::windows::EventLoopBuilderExtWindows;
    EventLoop::builder()
        .with_any_thread(true)
        .build()
        .expect("Failed to build event loop")
}

#[cfg(not(target_os = "windows"))]
fn build_event_loop() -> EventLoop<()> {
    EventLoop::builder()
        .build()
        .expect("Failed to build event loop")
}

impl Engine {
    pub fn new(event_loop: &ActiveEventLoop, settings: &'static EngineSettings) -> Self {
        Metric::reset();

        // Unsafe is required for FFI and JIT libs
        #[allow(unsafe_code)] // TODO: remove
        let lua = Rf::new(unsafe { Lua::unsafe_new() });

        std::panic::set_hook(Box::new(|panic_info| {
            error!(
                "Panic occurred in engine code!\nMessage: {panic_info}\nBacktrace:\n{}",
                std::backtrace::Backtrace::force_capture()
            );

            let location = if let Some(location) = panic_info.location() {
                format!("{}:{}", location.file(), location.line(),)
            } else {
                "<unknown>".to_string()
            };

            let panic_message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                format!("panic occurred at {location} - {s}")
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                format!("panic occurred at {location} - {}", s.as_str())
            } else {
                format!("panic occurred at {location}")
            };

            CURRENT_LUA_CTX.with_borrow(|v| {
                if let Some(ctx) = v {
                    let lua = ctx.as_ref();
                    let handle_error_func: Function = lua
                        .globals()
                        .get("HandleEngineError")
                        .expect("Unknown function HandleEngineError");
                    if let Err(e) = handle_error_func.call::<()>(panic_message) {
                        trace!("{}", e);
                    }
                } else {
                    error!("No Lua VM context, cannot get Lua backtrace.\n{panic_message}");
                }
            });

            std::process::exit(1);
        }));

        // Create window.
        let present_mode_override = settings.present_mode.map(PresentMode::from);
        let mut window = Window::default();
        if let Some(mode) = present_mode_override {
            info!("Present mode forced to {mode:?} by --present-mode");
            window.present_mode = mode;
        }
        let cache = CachedWindow {
            window: window.clone(),
        };
        let mut winit_window = WinitWindow::new(event_loop, &window);
        winit_window.resume();
        let scale_factor = window.scale_factor();

        // Every GL-touching type submits through this renderer - see
        // doc/engine/render-thread.md. Extracting the context here hands it
        // off for good: WinitWindow no longer performs GL operations itself
        // (its own swap_buffers is a no-op from this point on; frame end goes
        // through `renderer.end_frame_triple_buffered()` instead, driven from
        // `MainLoop::about_to_wait`).
        let context = winit_window
            .extract_gl_context()
            .expect("Failed to extract GL context for renderer");
        let mut renderer = Renderer::start(context).expect("Failed to start renderer");

        // Optional live stats dashboard (feature `stats-server`, activated by
        // the ltr `--stats-server <port>` flag).
        #[cfg(feature = "stats-server")]
        {
            let sink = crate::render::start_stats_server(settings.stats_port);
            renderer.attach_stats_sink(sink);
        }

        let hmgui = HmGui::new(&mut renderer, scale_factor);

        Self {
            init_time: TimeStamp::now(),
            window,
            cache,
            winit_window,
            hmgui,
            input: Default::default(),
            exit_app: false,
            event_bus: EventBus::new(),
            task_queue: TaskQueue::new(),
            lua,
            renderer,
            present_mode_override,
            present_mode_override_logged: false,
        }
    }

    pub fn entry(settings: &'static EngineSettings) {
        // Keep log till the end of the execution
        let _log = init_log(settings);

        if !settings.entry_point.exists() {
            // If we can't find it, set the current dir to one above the executable path and try that instead.
            let mut dir = std::env::current_exe().expect("Cannot get the path to the executable");
            dir.pop();
            dir.pop();
            debug!("Changing working directory to {:?}", dir);
            std::env::set_current_dir(dir).expect("Cannot change folder to parent");

            if !settings.entry_point.exists() {
                panic!(
                    "Can't find script entrypoint: {}",
                    settings.entry_point.display()
                );
            }
        }

        let mut app_state = MainLoop {
            engine: None,
            settings,
        };

        if let Err(err) = build_event_loop().run_app(&mut app_state) {
            error!("Event loop error: {err}");
        }
    }

    pub fn call_lua(&self, func_name: &str) -> Result<(), mlua::Error> {
        CURRENT_LUA_CTX.with_borrow_mut(|v| *v = Some(self.lua.clone()));

        let lua = self.lua.as_ref();
        let lua_func: Function = lua
            .globals()
            .get(func_name)
            .unwrap_or_else(|err| panic!("Unknown function {func_name}. Error: {err}"));
        let result = lua_func.call::<()>(());

        CURRENT_LUA_CTX.with_borrow_mut(|v| *v = None);

        result
    }

    // Apply user changes, and then detect changes to the window and update the winit window accordingly.
    pub fn changed_window(&mut self) {
        // Apply user input changes first
        for user_change in self.input.user_changes() {
            match user_change {
                UserChange::CursorVisible(visible) => self.window.cursor.visible = *visible,
                UserChange::CursorPosition(x, y) => {
                    self.window.set_cursor_position(Some(Vec2::new(*x, *y)))
                }
            }
        }

        // Handle suspend/resume
        if let Some(state) = self.window.state {
            match state {
                WindowState::Suspended => self.winit_window.suspend(),
                WindowState::Resumed => self.winit_window.resume(),
            }
            self.window.state = None;
        }

        // Update title
        if self.window.title != self.cache.window.title {
            self.winit_window
                .window()
                .set_title(self.window.title.as_str());
        }

        // === Handle window mode changes ===
        if self.window.mode != self.cache.window.mode {
            match self.window.mode {
                WindowMode::BorderlessFullscreen => {
                    // Choose the monitor explicitly (primary monitor recommended)
                    let monitor = self.winit_window.window().primary_monitor();
                    if let Some(monitor) = monitor {
                        // Apply borderless fullscreen
                        let fullscreen =
                            winit::window::Fullscreen::Borderless(Some(monitor.clone()));
                        self.winit_window.window().set_fullscreen(Some(fullscreen));

                        // Force correct position at monitor origin
                        let pos = monitor.position();
                        self.winit_window.window().set_outer_position(pos);
                    }
                }
                WindowMode::Fullscreen => {
                    let monitor = self
                        .winit_window
                        .window()
                        .current_monitor()
                        .unwrap_or_else(|| self.winit_window.window().primary_monitor().unwrap());
                    let videomode = get_best_videomode(&monitor);
                    self.winit_window
                        .window()
                        .set_fullscreen(Some(winit::window::Fullscreen::Exclusive(videomode)));
                }
                WindowMode::SizedFullscreen => {
                    let monitor = self
                        .winit_window
                        .window()
                        .current_monitor()
                        .unwrap_or_else(|| self.winit_window.window().primary_monitor().unwrap());
                    let videomode = get_fitting_videomode(
                        &monitor,
                        self.window.width() as u32,
                        self.window.height() as u32,
                    );
                    self.winit_window
                        .window()
                        .set_fullscreen(Some(winit::window::Fullscreen::Exclusive(videomode)));
                }
                WindowMode::Windowed => {
                    self.winit_window.window().set_fullscreen(None);

                    // Restore cached position for windowed mode
                    if let Some(position) = winit_window_position(
                        &self.window.position,
                        &self.window.resolution,
                        self.winit_window.window().available_monitors(),
                        self.winit_window.window().primary_monitor(),
                        self.winit_window.window().current_monitor(),
                    ) {
                        self.winit_window.window().set_outer_position(position);
                    }
                }
            }
        }

        // === Handle resolution changes ===
        if self.window.resolution != self.cache.window.resolution {
            let width = self.window.resolution.physical_width();
            let height = self.window.resolution.physical_height();
            let physical_size = PhysicalSize::new(width, height);

            if let Some(new_size) = self.winit_window.window().request_inner_size(physical_size) {
                self.winit_window.resize(new_size.width, new_size.height);
            }
        }

        // === Handle cursor position ===
        if self.window.physical_cursor_position() != self.cache.window.physical_cursor_position() {
            if let Some(physical_position) = self.window.physical_cursor_position() {
                let inner_size = self.winit_window.window().inner_size();

                let position = PhysicalPosition::new(
                    physical_position.x,
                    inner_size.height as f32 - physical_position.y,
                );

                if let Err(err) = self.winit_window.window().set_cursor_position(position) {
                    error!("Could not set cursor position: {:?}", err);
                }
            }
        }

        // === Handle cursors, grab, visibility ===
        if self.window.cursor.icon != self.cache.window.cursor.icon {
            self.winit_window
                .window()
                .set_cursor(convert_cursor_icon(self.window.cursor.icon));
        }

        if self.window.cursor.grab_mode != self.cache.window.cursor.grab_mode {
            if !self.winit_window.window().has_focus() {
                self.winit_window.window().focus_window();
            }
            attempt_grab(self.winit_window.window(), self.window.cursor.grab_mode);
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

        // === Handle decorations & resizable ===
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

        // === Handle resize constraints ===
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

        // === Handle window position for windowed mode only ===
        if self.window.mode == WindowMode::Windowed
            && self.window.position != self.cache.window.position
        {
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

        // === Handle maximize / minimize requests ===
        if let Some(maximized) = self.window.internal.take_maximize_request() {
            self.winit_window.window().set_maximized(maximized);
        }

        if let Some(minimized) = self.window.internal.take_minimize_request() {
            self.winit_window.window().set_minimized(minimized);
        }

        // === Handle internal cursor position requests ===
        if let Some(position) = self.window.internal.take_cursor_position_request() {
            self.winit_window
                .window()
                .set_cursor_position(PhysicalPosition::new(position.x, position.y))
                .unwrap_or_else(|err| {
                    error!("Could not set cursor position: {:?}", err);
                });
        }

        // === Focus ===
        if self.window.focused != self.cache.window.focused && self.window.focused {
            self.winit_window.window().focus_window();
        }

        // === Present mode / IME / themes ===
        if self.window.present_mode != self.cache.window.present_mode {
            if let Some(forced_mode) = self.present_mode_override {
                // CLI (`ltr --present-mode`) wins over the script - restore
                // the forced mode so this branch doesn't keep re-firing, and
                // let the operator know their script's request was ignored.
                self.window.present_mode = forced_mode;
                if !self.present_mode_override_logged {
                    info!(
                        "Ignoring script's present mode request - overridden by --present-mode ({forced_mode:?})"
                    );
                    self.present_mode_override_logged = true;
                }
            } else {
                // The GL context lives on the render thread (see
                // Engine::new's extract_gl_context handoff), and
                // set_swap_interval needs it current, so this goes through a
                // command instead of being applied here directly.
                self.renderer.set_present_mode(self.window.present_mode);
                self.winit_window.set_present_mode(self.window.present_mode);
            }
        }

        if self.window.ime_enabled != self.cache.window.ime_enabled {
            self.winit_window
                .window()
                .set_ime_allowed(self.window.ime_enabled);
        }

        if self.window.ime_position != self.cache.window.ime_position {
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

        // === Update cache at the end ===
        self.cache.window = self.window.clone();
    }
}

/// Engine entry point called by the `ltr` launcher.
///
/// `settings` points at a leaked [`EngineSettings`] owned by the launcher;
/// the engine borrows it for the whole run and never drops it.
#[allow(unsafe_code, improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn Engine_Entry(settings: &'static EngineSettings) {
    Engine::entry(settings)
}

#[luajit_ffi_gen::luajit_ffi]
impl Engine {
    pub fn window(&mut self) -> &mut Window {
        &mut self.window
    }

    pub fn input(&mut self) -> &mut Input {
        &mut self.input
    }

    pub fn event_bus(&mut self) -> &mut EventBus {
        &mut self.event_bus
    }

    pub fn task_queue(&mut self) -> &mut TaskQueue {
        &mut self.task_queue
    }

    #[bind(name = "HmGui")]
    pub fn hmgui(&mut self) -> &mut HmGui {
        &mut self.hmgui
    }

    pub fn renderer(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

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
        Profiler::begin("Engine_Update");
        Metric::reset();
        Profiler::end();
    }
}
