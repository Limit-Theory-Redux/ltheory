use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, GlProfile, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::{GlConfig, GlDisplay};
use glutin_winit::DisplayBuilder;
use hashbrown::HashMap;
use raw_window_handle::HasRawWindowHandle;
use tracing::{debug, error, info, warn};
use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    monitor::MonitorHandle,
};

use super::{CursorGrabMode, Window, WindowMode, WindowPosition, WindowResolution, WinitWindow};

/// A resource which contains [`winit`] library windows.
#[derive(Debug)]
pub struct WinitWindows {
    gl_version_major: u8,
    gl_version_minor: u8,

    /// Stores [`winit`] windows by window identifier.
    pub windows: HashMap<winit::window::WindowId, WinitWindow>,

    // Some winit functions, such as `set_window_icon` can only be used from the main thread. If
    // they are used in another thread, the app will hang. This marker ensures `WinitWindows` is
    // only ever accessed with LTR's non-send functions.
    _not_send_sync: core::marker::PhantomData<*const ()>,
}

impl WinitWindows {
    /// Create WinitWindows manager for specified GL version
    pub fn new(gl_version_major: u8, gl_version_minor: u8) -> Self {
        Self {
            gl_version_major,
            gl_version_minor,
            windows: Default::default(),
            _not_send_sync: Default::default(),
        }
    }

    /// Creates a `winit` window and associates it with our entity.
    pub fn create_window(
        &mut self,
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        window: &Window,
    ) -> winit::window::WindowId {
        info!("Create new window: {}", window.title);

        let mut winit_window_builder = winit::window::WindowBuilder::new();

        // Hide window until it is properly initialized
        winit_window_builder = winit_window_builder.with_visible(false);

        winit_window_builder = match window.mode {
            WindowMode::BorderlessFullscreen => winit_window_builder.with_fullscreen(Some(
                winit::window::Fullscreen::Borderless(event_loop.primary_monitor()),
            )),
            WindowMode::Fullscreen => {
                winit_window_builder.with_fullscreen(Some(winit::window::Fullscreen::Exclusive(
                    get_best_videomode(&event_loop.primary_monitor().unwrap()),
                )))
            }
            WindowMode::SizedFullscreen => winit_window_builder.with_fullscreen(Some(
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
                    winit_window_builder = winit_window_builder.with_position(position);
                }

                let logical_size = LogicalSize::new(window.width(), window.height());
                if let Some(sf) = window.resolution.scale_factor_override() {
                    winit_window_builder.with_inner_size(logical_size.to_physical::<f64>(sf))
                } else {
                    winit_window_builder.with_inner_size(logical_size)
                }
            }
        };

        winit_window_builder = winit_window_builder
            .with_window_level(window.window_level.into())
            .with_theme(window.window_theme.map(winit::window::Theme::from))
            .with_resizable(window.resizable)
            .with_decorations(window.decorations)
            .with_transparent(window.transparent);

        let constraints = window.resize_constraints.check_constraints();
        let min_inner_size = LogicalSize {
            width: constraints.min_width,
            height: constraints.min_height,
        };
        let max_inner_size = LogicalSize {
            width: constraints.max_width,
            height: constraints.max_height,
        };

        let winit_window_builder =
            if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
                winit_window_builder
                    .with_min_inner_size(min_inner_size)
                    .with_max_inner_size(max_inner_size)
            } else {
                winit_window_builder.with_min_inner_size(min_inner_size)
            };

        #[allow(unused_mut)]
        let mut winit_window_builder = winit_window_builder.with_title(window.title.as_str());

        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_transparency(cfg!(cgl_backend));
        let display_builder = DisplayBuilder::new().with_window_builder(Some(winit_window_builder));
        let (winit_window, gl_config) = display_builder
            .build(&event_loop, template, |configs| {
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

        debug!("Picked a config with {} samples", gl_config.num_samples());

        let raw_window_handle = winit_window
            .as_ref()
            .map(|window| window.raw_window_handle());

        // XXX The display could be obtained from the any object created by it, so we
        // can query it from the config.
        let gl_display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(
                self.gl_version_major,
                self.gl_version_minor,
            ))))
            .with_profile(GlProfile::Compatibility)
            .build(raw_window_handle);

        let gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .expect("failed to create context")
        };

        let winit_window = winit_window.unwrap();

        winit_window.set_visible(true);

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

        let id = winit_window.id();

        let mut winit_window_wrapper = WinitWindow::new(winit_window, gl_config, gl_context);

        winit_window_wrapper.resume();

        self.windows.insert(id, winit_window_wrapper);

        id
    }

    /// Get the winit window by id.
    pub fn get_window(&self, winit_id: winit::window::WindowId) -> Option<&WinitWindow> {
        self.windows.get(&winit_id)
    }

    /// Get mutable winit window by id.
    pub fn get_window_mut(
        &mut self,
        winit_id: winit::window::WindowId,
    ) -> Option<&mut WinitWindow> {
        self.windows.get_mut(&winit_id)
    }

    /// Remove a window from winit.
    ///
    /// This should mostly just be called when the window is closing.
    pub fn remove_window(&mut self, winit_id: winit::window::WindowId) -> Option<WinitWindow> {
        self.windows.remove(&winit_id)
    }
}

/// Gets the "best" video mode which fits the given dimensions.
///
/// The heuristic for "best" prioritizes width, height, and refresh rate in that order.
pub fn get_fitting_videomode(
    monitor: &winit::monitor::MonitorHandle,
    width: u32,
    height: u32,
) -> winit::monitor::VideoMode {
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
pub fn get_best_videomode(monitor: &winit::monitor::MonitorHandle) -> winit::monitor::VideoMode {
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
                        warn!("Can't select current monitor on window creation or cannot find current monitor!");
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
