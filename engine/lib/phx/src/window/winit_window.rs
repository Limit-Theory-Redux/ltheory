#![allow(unsafe_code)]

use std::num::NonZeroU32;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{
    ContextApi, ContextAttributesBuilder, GlProfile, NotCurrentContext, PossiblyCurrentContext,
    Version,
};
use glutin::display::GetGlDisplay;
use glutin::prelude::{GlConfig, GlDisplay, NotCurrentGlContext, PossiblyCurrentGlContext};
use glutin::surface::{GlSurface, Surface, SurfaceAttributes, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use tracing::{debug, error, info, warn};
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::monitor::MonitorHandle;

use super::{
    CursorGrabMode, PresentMode, Window, WindowMode, WindowPosition, WindowResolution,
    glutin_render,
};
use crate::render::{clear_gl_unavailable, is_command_mode, is_gl_unavailable, submit_command, try_submit_command, RenderCommand};

// TODO: Add GlStateManager with state: Option<GlState> field to avoid std::mem::replace
#[derive(Debug)]
enum GlState {
    Current {
        context: PossiblyCurrentContext,
        surface: Surface<WindowSurface>,
    },
    NotCurrent {
        context: NotCurrentContext,
    },
    Undefined,
}

impl GlState {
    fn make_current(
        &mut self,
        config: &glutin::config::Config,
        attrs: &SurfaceAttributes<WindowSurface>,
        present_mode: PresentMode,
    ) -> bool {
        if matches!(self, Self::NotCurrent { .. }) {
            let old_self = std::mem::replace(self, Self::Undefined);
            let Self::NotCurrent { context } = old_self else {
                unreachable!()
            };

            #[allow(unsafe_code)] // TODO: remove
            let surface = unsafe {
                config
                    .display()
                    .create_window_surface(config, attrs)
                    .unwrap()
            };

            let context = context
                .make_current(&surface)
                .expect("Cannot make context current");

            // Try setting vsync.
            if let Err(res) = surface.set_swap_interval(&context, present_mode.into()) {
                warn!("Error setting vsync: {res:?}");
            }

            *self = Self::Current { context, surface };

            true
        } else if matches!(self, Self::Current { .. }) {
            warn!("Context is already current");

            false
        } else {
            panic!("Context is undefined");
        }
    }

    fn make_not_current(&mut self) -> bool {
        if matches!(self, Self::NotCurrent { .. }) {
            warn!("Context is already not current");

            false
        } else if matches!(self, Self::Current { .. }) {
            let old_self = std::mem::replace(self, Self::Undefined);
            let Self::Current { context, .. } = old_self else {
                unreachable!()
            };

            let context = context
                .make_not_current()
                .expect("Cannot make context not current");

            *self = Self::NotCurrent { context };

            true
        } else {
            panic!("Context is undefined");
        }
    }
}

/// Data needed to initialize GL context on the render thread.
/// This is extracted from WinitWindow and sent to the render thread.
/// Contains both context and the already-created surface (macOS requires
/// surfaces to be created on the main thread).
pub struct RenderThreadGlData {
    /// The not-current GL context that can be made current on render thread
    pub context: NotCurrentContext,
    /// The surface (already created on main thread)
    pub surface: Surface<WindowSurface>,
    /// GL configuration
    pub config: glutin::config::Config,
}

// Safety: NotCurrentContext and Surface are both Send when not current
unsafe impl Send for RenderThreadGlData {}

impl RenderThreadGlData {
    /// Make the context current on the render thread and return the active context + surface.
    /// This should only be called once from the render thread.
    pub fn make_current(self) -> Result<RenderThreadGlContext, String> {
        let context = self.context
            .make_current(&self.surface)
            .map_err(|e| format!("Failed to make context current: {:?}", e))?;

        Ok(RenderThreadGlContext {
            context,
            surface: self.surface,
            config: self.config,
        })
    }
}

/// Active GL context on the render thread.
pub struct RenderThreadGlContext {
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
    config: glutin::config::Config,
}

impl RenderThreadGlContext {
    /// Swap buffers
    pub fn swap_buffers(&self) -> Result<(), String> {
        self.surface
            .swap_buffers(&self.context)
            .map_err(|e| format!("Failed to swap buffers: {:?}", e))
    }

    /// Resize the surface
    pub fn resize(&self, width: u32, height: u32) {
        if let Some(width) = NonZeroU32::new(width) {
            if let Some(height) = NonZeroU32::new(height) {
                self.surface.resize(&self.context, width, height);
            }
        }
    }

    /// Make the context not current (for cleanup)
    pub fn make_not_current(self) -> Result<NotCurrentContext, String> {
        self.context
            .make_not_current()
            .map_err(|e| format!("Failed to make context not current: {:?}", e))
    }

    /// Make context not current and return both context and surface for transfer back to main thread.
    /// This is used when shutting down the render thread to return the GL resources.
    ///
    /// On macOS, make_not_current() uses run_on_main() which deadlocks when main thread
    /// is blocked waiting for this function to complete. So we skip it on macOS and just
    /// drop the context (it will be destroyed when the thread exits).
    #[cfg(not(target_os = "macos"))]
    pub fn release_for_main_thread(self) -> Result<(NotCurrentContext, Surface<WindowSurface>), String> {
        let context = self.context
            .make_not_current()
            .map_err(|e| format!("Failed to make context not current: {:?}", e))?;
        Ok((context, self.surface))
    }

    #[cfg(target_os = "macos")]
    pub fn release_for_main_thread(self) -> Result<(NotCurrentContext, Surface<WindowSurface>), String> {
        // On macOS, make_not_current() uses run_on_main() which deadlocks when main
        // thread is blocked waiting for this function.
        //
        // We cannot safely release the context back to main thread without causing
        // a deadlock. Use std::mem::forget to skip destructors (which might also
        // use run_on_main). The GL resources will be leaked but cleaned up by the
        // OS when the process exits.
        //
        // This means we cannot restore direct GL mode on macOS after using the
        // render thread, but that's an acceptable limitation.
        std::mem::forget(self.context);
        std::mem::forget(self.surface);
        Err("Cannot release GL context on macOS without deadlock - resources leaked".to_string())
    }
}

#[derive(Debug)]
pub struct WinitWindow {
    window: winit::window::Window,
    present_mode: PresentMode,
    gl_config: glutin::config::Config,
    gl_state: GlState,
}

impl WinitWindow {
    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop, window: &Window) -> Self {
        info!("Create new window: {}", window.title);

        let mut window_attributes = winit::window::Window::default_attributes();

        // Hide window until it is properly initialized
        window_attributes = window_attributes.with_visible(false);
        window_attributes = match window.mode {
            WindowMode::BorderlessFullscreen => window_attributes.with_fullscreen(Some(
                winit::window::Fullscreen::Borderless(event_loop.primary_monitor()),
            )),
            WindowMode::Fullscreen => {
                window_attributes.with_fullscreen(Some(winit::window::Fullscreen::Exclusive(
                    get_best_videomode(&event_loop.primary_monitor().unwrap()),
                )))
            }
            WindowMode::SizedFullscreen => window_attributes.with_fullscreen(Some(
                winit::window::Fullscreen::Exclusive(get_fitting_videomode(
                    &event_loop.primary_monitor().unwrap(),
                    window.width() as u32,
                    window.height() as u32,
                )),
            )),
            WindowMode::Windowed => {
                if let Some(position) = winit_window_position(
                    &window.position,
                    &window.resolution,
                    event_loop.available_monitors(),
                    event_loop.primary_monitor(),
                    None,
                ) {
                    window_attributes = window_attributes.with_position(position);
                }

                let logical_size = LogicalSize::new(window.width(), window.height());
                if let Some(sf) = window.resolution.scale_factor_override() {
                    window_attributes.with_inner_size(logical_size.to_physical::<f64>(sf))
                } else {
                    window_attributes.with_inner_size(logical_size)
                }
            }
        };

        window_attributes = window_attributes
            .with_theme(window.window_theme.map(winit::window::Theme::from))
            .with_resizable(window.resizable)
            .with_decorations(window.decorations);

        let constraints = window.resize_constraints.check_constraints();
        let min_inner_size = LogicalSize {
            width: constraints.min_width,
            height: constraints.min_height,
        };
        let max_inner_size = LogicalSize {
            width: constraints.max_width,
            height: constraints.max_height,
        };

        window_attributes =
            if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
                window_attributes
                    .with_min_inner_size(min_inner_size)
                    .with_max_inner_size(max_inner_size)
            } else {
                window_attributes.with_min_inner_size(min_inner_size)
            };

        window_attributes = window_attributes.with_title(window.title.as_str());

        let template = ConfigTemplateBuilder::new().with_alpha_size(8);
        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));
        let (winit_window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
                // Find the config with the maximum number of samples, so our triangle will
                // be smooth.
                configs
                    .reduce(|accum, config| {
                        let transparency_check = config.supports_transparency().unwrap_or(false)
                            & !accum.supports_transparency().unwrap_or(false);

                        if transparency_check || config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .unwrap();

        let winit_window = winit_window.unwrap();

        debug!("Picked a config with {} samples", gl_config.num_samples());

        let gl_display = gl_config.display();
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .with_profile(GlProfile::Core)
            .build(
                winit_window
                    .window_handle()
                    .as_ref()
                    .ok()
                    .map(|wh| wh.as_raw()),
            );

        #[allow(unsafe_code)] // TODO: remove
        let gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .expect("failed to create context")
        };

        // Do not set the grab mode on window creation if it's none, this can fail on mobile
        if window.cursor.grab_mode != CursorGrabMode::None {
            attempt_grab(&winit_window, window.cursor.grab_mode);
        }

        winit_window.set_cursor_visible(window.cursor.visible);

        // Do not set the cursor hittest on window creation if it's false, as it will always fail on some
        // platforms and log an unfixable warning.
        if !window.cursor.hit_test {
            if let Err(err) = winit_window.set_cursor_hittest(window.cursor.hit_test) {
                warn!(
                    "Could not set cursor hit test for window {:?}: {:?}",
                    window.title, err
                );
            }
        }

        Self {
            window: winit_window,
            present_mode: window.present_mode,
            gl_config,
            gl_state: GlState::NotCurrent {
                context: gl_context,
            },
        }
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn resume(&mut self) {
        debug!("WinitWindow::resume");

        let attrs = self
            .window
            .build_surface_attributes(<_>::default())
            .unwrap();

        if self
            .gl_state
            .make_current(&self.gl_config, &attrs, self.present_mode)
        {
            // The context needs to be current for the Renderer to set up shaders and
            // buffers. It also performs function loading, which needs a current context on
            // WGL.
            glutin_render::init_renderer(&self.gl_config.display());
        }
    }

    pub fn suspend(&mut self) {
        debug!("WinitWindow::suspend");

        self.gl_state.make_not_current();
    }

    pub fn resize(&self, width: u32, height: u32) {
        // Skip if GL context is unavailable
        if is_gl_unavailable() {
            return;
        }

        if let Some(width) = NonZeroU32::new(width) {
            if let Some(height) = NonZeroU32::new(height) {
                // Some platforms like EGL require resizing GL surface to update the size
                // Notable platforms here are Wayland and macOS, other don't require it
                // and the function is no-op, but it's wise to resize it for portability
                // reasons.
                if let GlState::Current { context, surface } = &self.gl_state {
                    surface.resize(context, width, height);

                    glutin_render::resize(width.get() as i32, height.get() as i32);
                } else if is_command_mode() {
                    // In command mode, send resize to render thread which owns the GL context.
                    // Use try_submit to avoid blocking if the render thread is busy (e.g., vsync).
                    // If the command is dropped, the next frame will catch up.
                    try_submit_command(RenderCommand::Resize {
                        width: width.get(),
                        height: height.get(),
                    });
                }
            }
        }
    }

    pub fn redraw(&self) {
        self.window.request_redraw();

        // Check if GL context is unavailable (lost after render thread shutdown)
        if is_gl_unavailable() {
            // Skip rendering - GL context was lost and cannot be recovered
            return;
        }

        if let GlState::Current { context, surface } = &self.gl_state {
            // Direct GL mode - swap buffers on main thread
            surface.swap_buffers(context).expect("Cannot redraw");
        } else if is_command_mode() {
            // Command mode - send SwapBuffers to render thread
            if !submit_command(RenderCommand::SwapBuffers) {
                warn!("Failed to submit SwapBuffers command - render thread may not be running");
            }
        } else {
            // Invalid state - GL context not current and not in command mode
            error!("redraw() called but GL context is not current and not in command mode!");
        }
    }

    /// Extract the GL context for transfer to the render thread.
    ///
    /// This makes the context not-current on the main thread and returns
    /// the data needed to initialize the context on the render thread.
    ///
    /// After calling this, the WinitWindow can no longer perform GL operations
    /// directly. All GL operations must go through the render thread.
    pub fn extract_gl_context_for_render_thread(&mut self) -> Option<RenderThreadGlData> {
        debug!("Extracting GL context for render thread");

        // Extract both context and surface from Current state
        // We need to transfer the existing surface because macOS doesn't allow
        // creating window surfaces from non-main threads.
        if matches!(self.gl_state, GlState::Current { .. }) {
            let old_state = std::mem::replace(&mut self.gl_state, GlState::Undefined);
            if let GlState::Current { context, surface } = old_state {
                // Make context not current before transferring
                let context = context
                    .make_not_current()
                    .expect("Failed to make context not current for transfer");

                info!("GL context extracted for render thread");

                return Some(RenderThreadGlData {
                    context,
                    surface,
                    config: self.gl_config.clone(),
                });
            }
        }

        warn!("Could not extract GL context - context is not current");
        None
    }

    /// Check if the GL context has been transferred to the render thread.
    pub fn is_context_on_render_thread(&self) -> bool {
        matches!(self.gl_state, GlState::Undefined)
    }

    /// Request redraw without swapping buffers (for use with render thread).
    /// The render thread will handle the actual buffer swap.
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    /// Restore the GL context from the render thread.
    ///
    /// This is called when the render thread shuts down and returns the GL context
    /// back to the main thread for direct GL rendering.
    pub fn restore_gl_context(
        &mut self,
        context: NotCurrentContext,
        surface: Surface<WindowSurface>,
    ) -> bool {
        info!("Attempting to restore GL context, current state: {:?}",
              match &self.gl_state {
                  GlState::Current { .. } => "Current",
                  GlState::NotCurrent { .. } => "NotCurrent",
                  GlState::Undefined => "Undefined",
              });

        if !matches!(self.gl_state, GlState::Undefined) {
            error!("Cannot restore GL context - current state is not Undefined!");
            return false;
        }

        // Make the context current on the main thread
        info!("Making context current on main thread...");
        match context.make_current(&surface) {
            Ok(current_context) => {
                // Try setting vsync
                if let Err(res) = surface.set_swap_interval(&current_context, self.present_mode.into()) {
                    warn!("Error setting vsync after restoring context: {:?}", res);
                }

                self.gl_state = GlState::Current {
                    context: current_context,
                    surface,
                };
                // Clear the GL unavailable flag since context is restored
                clear_gl_unavailable();
                info!("GL context restored to main thread successfully");
                true
            }
            Err(e) => {
                error!("Failed to make restored context current: {:?}", e);
                false
            }
        }
    }
}

/// Gets the "best" video mode which fits the given dimensions.
///
/// The heuristic for "best" prioritizes width, height, and refresh rate in that order.
pub fn get_fitting_videomode(
    monitor: &winit::monitor::MonitorHandle,
    width: u32,
    height: u32,
) -> winit::monitor::VideoModeHandle {
    let mut modes = monitor.video_modes().collect::<Vec<_>>();

    fn abs_diff(a: u32, b: u32) -> u32 {
        if a > b {
            return a - b;
        }
        b - a
    }

    modes.sort_by(|a, b| {
        use std::cmp::Ordering::*;
        match abs_diff(a.size().width, width).cmp(&abs_diff(b.size().width, width)) {
            Equal => {
                match abs_diff(a.size().height, height).cmp(&abs_diff(b.size().height, height)) {
                    Equal => b
                        .refresh_rate_millihertz()
                        .cmp(&a.refresh_rate_millihertz()),
                    default => default,
                }
            }
            default => default,
        }
    });

    modes.first().unwrap().clone()
}

/// Gets the "best" videomode from a monitor.
///
/// The heuristic for "best" prioritizes width, height, and refresh rate in that order.
pub fn get_best_videomode(
    monitor: &winit::monitor::MonitorHandle,
) -> winit::monitor::VideoModeHandle {
    let mut modes = monitor.video_modes().collect::<Vec<_>>();
    modes.sort_by(|a, b| {
        use std::cmp::Ordering::*;
        match b.size().width.cmp(&a.size().width) {
            Equal => match b.size().height.cmp(&a.size().height) {
                Equal => b
                    .refresh_rate_millihertz()
                    .cmp(&a.refresh_rate_millihertz()),
                default => default,
            },
            default => default,
        }
    });

    modes.first().unwrap().clone()
}

pub(crate) fn attempt_grab(winit_window: &winit::window::Window, grab_mode: CursorGrabMode) {
    let grab_result = match grab_mode {
        CursorGrabMode::None => winit_window.set_cursor_grab(winit::window::CursorGrabMode::None),
        CursorGrabMode::Confined => winit_window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .or_else(|_e| winit_window.set_cursor_grab(winit::window::CursorGrabMode::Locked)),
        CursorGrabMode::Locked => winit_window
            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            .or_else(|_e| winit_window.set_cursor_grab(winit::window::CursorGrabMode::Confined)),
    };

    if let Err(err) = grab_result {
        let err_desc = match grab_mode {
            CursorGrabMode::Confined | CursorGrabMode::Locked => "grab",
            CursorGrabMode::None => "ungrab",
        };

        error!("Unable to {} cursor: {}", err_desc, err);
    }
}

/// Compute the physical window position for a given [`WindowPosition`].
// Ideally we could generify this across window backends, but we only really have winit atm
// so whatever.
pub fn winit_window_position(
    position: &WindowPosition,
    resolution: &WindowResolution,
    mut available_monitors: impl Iterator<Item = MonitorHandle>,
    primary_monitor: Option<MonitorHandle>,
    current_monitor: Option<MonitorHandle>,
) -> Option<PhysicalPosition<i32>> {
    match position {
        WindowPosition::Automatic => {
            /* Window manager will handle position */
            None
        }
        WindowPosition::Centered(monitor_selection) => {
            use super::MonitorSelection::*;
            let maybe_monitor = match monitor_selection {
                Current => {
                    if current_monitor.is_none() {
                        warn!(
                            "Can't select current monitor on window creation or cannot find current monitor!"
                        );
                    }
                    current_monitor
                }
                Primary => primary_monitor,
                Index(n) => available_monitors.nth(*n),
            };

            if let Some(monitor) = maybe_monitor {
                let screen_size = monitor.size();

                // We use the monitors scale factor here since WindowResolution.scale_factor
                // is not yet populated when windows are created at plugin setup
                let scale_factor = monitor.scale_factor();

                // Logical to physical window size
                let (width, height): (u32, u32) =
                    LogicalSize::new(resolution.width(), resolution.height())
                        .to_physical::<u32>(scale_factor)
                        .into();

                let position = PhysicalPosition {
                    x: screen_size.width.saturating_sub(width) as f64 / 2.
                        + monitor.position().x as f64,
                    y: screen_size.height.saturating_sub(height) as f64 / 2.
                        + monitor.position().y as f64,
                };

                Some(position.cast::<i32>())
            } else {
                warn!("Couldn't get monitor selected with: {monitor_selection:?}");
                None
            }
        }
        WindowPosition::At(position) => {
            Some(PhysicalPosition::new(position[0] as f64, position[1] as f64).cast::<i32>())
        }
    }
}
