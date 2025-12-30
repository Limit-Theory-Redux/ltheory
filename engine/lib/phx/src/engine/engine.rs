use std::cell::RefCell;
use std::path::PathBuf;

use glam::*;
use mlua::{Function, Lua};
use tracing::*;
use winit::dpi::*;
use winit::event_loop::*;

use super::{EventBus, MainLoop, TaskQueue};
use crate::input::*;
use crate::logging::init_log;
use crate::math::Matrix;
use crate::render::{
    RenderContext, RenderCommand,
    flush_render_batch_with_workers,
    submit_command, is_command_mode, CameraUboData, update_global_camera_ubo,
    LightUboData, update_global_light_ubo, Shader,
};
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
    pub lua: Rf<Lua>,
    /// Multithreaded rendering subsystem (render thread + worker pool)
    pub render_context: RenderContext,
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
    pub fn new(event_loop: &ActiveEventLoop) -> Self {
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
        let window = Window::default();
        let cache = CachedWindow {
            window: window.clone(),
        };
        let mut winit_window = WinitWindow::new(event_loop, &window);
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
            event_bus: EventBus::new(),
            task_queue: TaskQueue::new(),
            lua,
            render_context: RenderContext::new(),
        }
    }

    /// Start the render thread.
    ///
    /// This transfers the GL context to a dedicated render thread.
    /// After calling this, all GL operations must go through the render queue.
    pub fn start_render_thread(&mut self) -> bool {
        // Extract GL context from the window
        if let Some(gl_data) = self.winit_window.extract_gl_context_for_render_thread() {
            self.render_context.start(gl_data)
        } else {
            error!("Failed to extract GL context for render thread");
            false
        }
    }

    /// Stop the render thread.
    ///
    /// This shuts down the render thread and returns the GL context to the main thread.
    pub fn stop_render_thread(&mut self) {
        if let Some(returned_ctx) = self.render_context.stop() {
            // Restore the context to WinitWindow
            if self.winit_window.restore_gl_context(returned_ctx.context, returned_ctx.surface) {
                info!("GL context successfully restored to main thread");
            } else {
                error!("Failed to restore GL context to main thread");
            }
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
            warn!("Unable to change present mode after window creation!");
            self.window.present_mode = self.cache.window.present_mode;
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

        let mut app_state = MainLoop {
            engine: None,
            app_name,
            entry_point_path,
        };
        let _ = build_event_loop().run_app(&mut app_state);
    }

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

    // === Render Thread Control ===

    /// Start the multithreaded render system.
    ///
    /// This transfers the GL context to a dedicated render thread.
    /// All GL operations will then go through command mode.
    /// Returns true if successfully started.
    #[bind(name = "StartRenderThread")]
    pub fn start_render_thread_ffi(&mut self) -> bool {
        self.start_render_thread()
    }

    /// Stop the multithreaded render system.
    ///
    /// This shuts down the render thread and returns the GL context
    /// to the main thread for direct rendering.
    #[bind(name = "StopRenderThread")]
    pub fn stop_render_thread_ffi(&mut self) {
        self.stop_render_thread()
    }

    /// Check if the render thread is currently active.
    #[bind(name = "IsRenderThreadActive")]
    pub fn is_render_thread_active(&self) -> bool {
        self.render_context.is_active()
    }

    /// Get total commands processed by the render thread.
    #[bind(name = "GetRenderThreadCommands")]
    pub fn get_render_thread_commands(&self) -> u64 {
        self.render_context.get_commands_processed()
    }

    /// Get total draw calls executed by the render thread.
    #[bind(name = "GetRenderThreadDrawCalls")]
    pub fn get_render_thread_draw_calls(&self) -> u64 {
        self.render_context.get_draw_calls()
    }

    /// Get total state changes on the render thread.
    #[bind(name = "GetRenderThreadStateChanges")]
    pub fn get_render_thread_state_changes(&self) -> u64 {
        self.render_context.get_state_changes()
    }

    /// Get total frames rendered by the render thread.
    #[bind(name = "GetRenderThreadFrameCount")]
    pub fn get_render_thread_frame_count(&self) -> u64 {
        self.render_context.get_frame_count()
    }

    /// Get the last frame render time in milliseconds.
    #[bind(name = "GetRenderThreadFrameTimeMs")]
    pub fn get_render_thread_frame_time_ms(&self) -> f64 {
        self.render_context.get_last_frame_time_ms()
    }

    /// Get commands processed in the last frame.
    #[bind(name = "GetRenderThreadCommandsPerFrame")]
    pub fn get_render_thread_commands_per_frame(&self) -> u64 {
        self.render_context.get_commands_last_frame()
    }

    /// Get draw calls executed in the last frame.
    #[bind(name = "GetRenderThreadDrawCallsPerFrame")]
    pub fn get_render_thread_draw_calls_per_frame(&self) -> u64 {
        self.render_context.get_draw_calls_last_frame()
    }

    /// Get total texture binds skipped due to caching (cumulative).
    #[bind(name = "GetRenderThreadTextureBindsSkipped")]
    pub fn get_render_thread_texture_binds_skipped(&self) -> u64 {
        self.render_context.get_texture_binds_skipped()
    }

    /// Get main thread wait time in milliseconds (time spent waiting for render thread).
    #[bind(name = "GetMainThreadWaitTimeMs")]
    pub fn get_main_thread_wait_time_ms(&self) -> f64 {
        self.render_context.get_main_thread_wait_ms()
    }

    /// Get current frames in flight (submitted but not yet rendered).
    #[bind(name = "GetFramesInFlight")]
    pub fn get_frames_in_flight(&self) -> u64 {
        self.render_context.get_frames_in_flight()
    }

    /// Get the number of CPU cores available for worker threads.
    #[bind(name = "GetCpuCount")]
    pub fn get_cpu_count(&self) -> u32 {
        num_cpus::get() as u32
    }

    /// Get the number of worker threads that would be spawned.
    /// This is CPU cores - 2 (reserve for main + render thread), minimum 1.
    #[bind(name = "GetWorkerThreadCount")]
    pub fn get_worker_thread_count(&self) -> u32 {
        let cores = num_cpus::get();
        (cores.saturating_sub(2)).max(1) as u32
    }

    /// Check if the worker pool is active.
    #[bind(name = "IsWorkerPoolActive")]
    pub fn is_worker_pool_active(&self) -> bool {
        self.render_context.worker_pool().is_some()
    }

    /// Get the actual number of active workers in the pool.
    #[bind(name = "GetActiveWorkerCount")]
    pub fn get_active_worker_count(&self) -> u32 {
        self.render_context.worker_pool().map_or(0, |p| p.num_workers() as u32)
    }

    /// Flush the render batch using the worker pool for parallel processing.
    /// This submits accumulated entities to workers for frustum culling and
    /// command generation. Returns the number of entities visible after culling.
    #[bind(name = "FlushRenderBatch")]
    pub fn flush_render_batch(&self) -> u32 {
        let stats = flush_render_batch_with_workers(self.render_context.worker_pool());
        stats.entities_visible
    }

    /// Create the camera UBO on the render thread.
    /// Call this once after starting the render thread.
    #[bind(name = "CreateCameraUBO")]
    pub fn create_camera_ubo(&self) {
        if is_command_mode() {
            submit_command(RenderCommand::CreateCameraUBO);
        }
    }

    /// Update the camera UBO with new matrix and uniform data.
    /// This should be called each frame before rendering when in command mode.
    ///
    /// Parameters:
    /// - m_view: View matrix (camera-relative, position at origin)
    /// - m_view_inv: View inverse matrix (with actual world position for worldray.glsl)
    /// - m_proj: Projection matrix
    /// - eye_x, eye_y, eye_z: Camera eye position
    /// - star_dir_x, star_dir_y, star_dir_z: Star/light direction
    #[bind(name = "UpdateCameraUBO")]
    pub fn update_camera_ubo(
        &self,
        m_view: &Matrix,
        m_view_inv: &Matrix,
        m_proj: &Matrix,
        eye_x: f32,
        eye_y: f32,
        eye_z: f32,
        star_dir_x: f32,
        star_dir_y: f32,
        star_dir_z: f32,
    ) {
        // Convert matrices
        let view = Mat4::from_cols_array(&m_view.to_cols_array());
        let view_inv = Mat4::from_cols_array(&m_view_inv.to_cols_array());
        let proj = Mat4::from_cols_array(&m_proj.to_cols_array());
        let eye = Vec3::new(eye_x, eye_y, eye_z);
        let star_dir = Vec3::new(star_dir_x, star_dir_y, star_dir_z);

        if is_command_mode() {
            // Render thread mode: submit command
            let mut ubo_data = CameraUboData::new();
            ubo_data.set_view(&view);
            ubo_data.set_view_inv(&view_inv);
            ubo_data.set_proj(&proj);
            ubo_data.set_eye(eye);
            ubo_data.set_star_dir(star_dir);

            let bytes = ubo_data.as_bytes();
            let mut boxed: Box<[u8; 288]> = Box::new([0u8; 288]);
            boxed.copy_from_slice(bytes);

            submit_command(RenderCommand::UpdateCameraUBO { data: boxed });
        } else {
            // Direct GL mode: update global UBO directly
            update_global_camera_ubo(&view, &view_inv, &proj, eye, star_dir);
        }
    }

    /// Create the light UBO on the render thread.
    /// This should be called once before using UpdateLightUBO.
    #[bind(name = "CreateLightUBO")]
    pub fn create_light_ubo(&self) {
        if is_command_mode() {
            submit_command(RenderCommand::CreateLightUBO);
        }
    }

    /// Update the light UBO with new light data.
    /// This should be called for each point light before rendering it.
    ///
    /// Parameters:
    /// - pos_x, pos_y, pos_z: Light position in world space
    /// - radius: Light falloff radius
    /// - r, g, b: Light color (0.0 to 1.0)
    /// - intensity: Light intensity multiplier
    #[bind(name = "UpdateLightUBO")]
    pub fn update_light_ubo(
        &self,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        radius: f32,
        r: f32,
        g: f32,
        b: f32,
        intensity: f32,
    ) {
        if is_command_mode() {
            // Render thread mode: submit command
            let mut ubo_data = LightUboData::new();
            ubo_data.set_position(pos_x, pos_y, pos_z);
            ubo_data.set_radius(radius);
            ubo_data.set_color(r, g, b);
            ubo_data.set_intensity(intensity);

            let boxed: Box<[u8; 32]> = Box::new(*ubo_data.as_bytes());

            submit_command(RenderCommand::UpdateLightUBO { data: boxed });
        } else {
            // Direct GL mode: update global UBO directly
            update_global_light_ubo(pos_x, pos_y, pos_z, radius, r, g, b, intensity);
        }
    }

    /// Reload a shader on the render thread.
    ///
    /// This compiles a shader on the render thread (which owns the GL context)
    /// and returns whether it succeeded. Use this for hot-reloading shaders
    /// when the render thread is active.
    ///
    /// Parameters:
    /// - shader_key: The cache key for the shader (e.g., "wvpfragment/material/solidcolor")
    /// - vs_name: Vertex shader resource name (e.g., "vertex/wvp")
    /// - fs_name: Fragment shader resource name (e.g., "fragment/material/solidcolor")
    ///
    /// Returns: true if shader compiled successfully, false otherwise
    #[bind(name = "ReloadShaderOnRenderThread")]
    pub fn reload_shader_on_render_thread(
        &self,
        shader_key: &str,
        vs_name: &str,
        fs_name: &str,
    ) -> bool {
        if let Some(handle) = self.render_context.handle() {
            // Load and preprocess shader source files
            let (vs_src, fs_src) = Shader::get_preprocessed_source(vs_name, fs_name);

            // Send to render thread for compilation
            let result = handle.reload_shader(shader_key, &vs_src, &fs_src);

            if result.success {
                info!(
                    "Shader '{}' reloaded successfully on render thread (program={})",
                    shader_key, result.program
                );
            } else {
                warn!(
                    "Shader '{}' failed to reload on render thread: {}",
                    shader_key,
                    result.error.as_deref().unwrap_or("unknown error")
                );
            }

            result.success
        } else {
            warn!("ReloadShaderOnRenderThread called but render thread is not active");
            false
        }
    }
}
