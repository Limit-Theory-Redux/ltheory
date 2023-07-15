mod composite_alpha_mode;
mod cursor;
mod monitor_selection;
mod present_mode;
mod window_level;
mod window_mode;
mod window_position;
mod window_resize_constraints;
mod window_resolution;
mod winit_windows;

pub use composite_alpha_mode::*;
pub use cursor::*;
pub use monitor_selection::*;
pub use present_mode::*;
pub use window_level::*;
pub use window_mode::*;
pub use window_position::*;
pub use window_resize_constraints::*;
pub use window_resolution::*;

use super::*;
use crate::common::*;
use crate::internal::*;
use crate::math::*;
use crate::render::*;
use crate::system::*;

/// The defining [`Component`] for window entities,
/// storing information about how it should appear and behave.
///
/// Each window corresponds to an entity, and is uniquely identified by the value of their [`Entity`].
/// When the [`Window`] component is added to an entity, a new window will be opened.
/// When it is removed or the entity is despawned, the window will close.
///
/// This component is synchronized with `winit` through `bevy_winit`:
/// it will reflect the current state of the window and can be modified to change this state.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct Window {
    /// The cursor of this window.
    pub cursor: Cursor,
    /// What presentation mode to give the window.
    pub present_mode: PresentMode,
    /// Which fullscreen or windowing mode should be used.
    pub mode: WindowMode,
    /// Where the window should be placed.
    pub position: WindowPosition,
    /// What resolution the window should have.
    pub resolution: WindowResolution,
    /// Stores the title of the window.
    pub title: String,
    /// How the alpha channel of textures should be handled while compositing.
    pub composite_alpha_mode: CompositeAlphaMode,
    /// The limits of the window's logical size
    /// (found in its [`resolution`](WindowResolution)) when resizing.
    pub resize_constraints: WindowResizeConstraints,
    /// Should the window be resizable?
    ///
    /// Note: This does not stop the program from fullscreening/setting
    /// the size programmatically.
    pub resizable: bool,
    /// Should the window have decorations enabled?
    ///
    /// (Decorations are the minimize, maximize, and close buttons on desktop apps)
    ///
    //  ## Platform-specific
    //
    //  **`iOS`**, **`Android`**, and the **`Web`** do not have decorations.
    pub decorations: bool,
    /// Should the window be transparent?
    ///
    /// Defines whether the background of the window should be transparent.
    ///
    /// ## Platform-specific
    /// - iOS / Android / Web: Unsupported.
    /// - macOS X: Not working as expected.
    ///
    /// macOS X transparent works with winit out of the box, so this issue might be related to: <https://github.com/gfx-rs/wgpu/issues/687>.
    /// You should also set the window `composite_alpha_mode` to `CompositeAlphaMode::PostMultiplied`.
    pub transparent: bool,
    /// Get/set whether the window is focused.
    pub focused: bool,
    /// Where should the window appear relative to other overlapping window.
    ///
    /// ## Platform-specific
    ///
    /// - iOS / Android / Web / Wayland: Unsupported.
    pub window_level: WindowLevel,
    /// The "html canvas" element selector.
    ///
    /// If set, this selector will be used to find a matching html canvas element,
    /// rather than creating a new one.
    /// Uses the [CSS selector format](https://developer.mozilla.org/en-US/docs/Web/API/Document/querySelector).
    ///
    /// This value has no effect on non-web platforms.
    pub canvas: Option<String>,
    /// Whether or not to fit the canvas element's size to its parent element's size.
    ///
    /// **Warning**: this will not behave as expected for parents that set their size according to the size of their
    /// children. This creates a "feedback loop" that will result in the canvas growing on each resize. When using this
    /// feature, ensure the parent's size is not affected by its children.
    ///
    /// This value has no effect on non-web platforms.
    pub fit_canvas_to_parent: bool,
    /// Whether or not to stop events from propagating out of the canvas element
    ///
    ///  When `true`, this will prevent common browser hotkeys like F5, F12, Ctrl+R, tab, etc.
    /// from performing their default behavior while the bevy app has focus.
    ///
    /// This value has no effect on non-web platforms.
    pub prevent_default_event_handling: bool,
    /// Stores internal state that isn't directly accessible.
    pub internal: InternalWindowState,
    /// Should the window use Input Method Editor?
    ///
    /// If enabled, the window will receive [`Ime`](crate::Ime) events instead of
    /// [`ReceivedCharacter`](crate::ReceivedCharacter) or
    /// [`KeyboardInput`](bevy_input::keyboard::KeyboardInput).
    ///
    /// IME should be enabled during text input, but not when you expect to get the exact key pressed.
    ///
    ///  ## Platform-specific
    ///
    /// - iOS / Android / Web: Unsupported.
    pub ime_enabled: bool,
    /// Sets location of IME candidate box in client area coordinates relative to the top left.
    ///
    ///  ## Platform-specific
    ///
    /// - iOS / Android / Web: Unsupported.
    pub ime_position: Vec2,
    /// Sets a specific theme for the window.
    ///
    /// If `None` is provided, the window will use the system theme.
    ///
    /// ## Platform-specific
    ///
    /// - iOS / Android / Web: Unsupported.
    pub window_theme: Option<WindowTheme>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            title: "Limit Theory Redux".to_owned(),
            cursor: Default::default(),
            present_mode: Default::default(),
            mode: Default::default(),
            position: Default::default(),
            resolution: Default::default(),
            internal: Default::default(),
            composite_alpha_mode: Default::default(),
            resize_constraints: Default::default(),
            ime_enabled: Default::default(),
            ime_position: Default::default(),
            resizable: true,
            decorations: true,
            transparent: false,
            focused: true,
            window_level: Default::default(),
            fit_canvas_to_parent: false,
            prevent_default_event_handling: true,
            canvas: None,
            window_theme: None,
        }
    }
}

impl Window {
    /// Setting this to true will attempt to maximize the window.
    ///
    /// Setting it to false will attempt to un-maximize the window.
    pub fn set_maximized(&mut self, maximized: bool) {
        self.internal.maximize_request = Some(maximized);
    }

    /// Setting this to true will attempt to minimize the window.
    ///
    /// Setting it to false will attempt to un-minimize the window.
    pub fn set_minimized(&mut self, minimized: bool) {
        self.internal.minimize_request = Some(minimized);
    }

    /// The window's client area width in logical pixels.
    ///
    /// See [`WindowResolution`] for an explanation about logical/physical sizes.
    #[inline]
    pub fn width(&self) -> f32 {
        self.resolution.width()
    }

    /// The window's client area height in logical pixels.
    ///
    /// See [`WindowResolution`] for an explanation about logical/physical sizes.
    #[inline]
    pub fn height(&self) -> f32 {
        self.resolution.height()
    }

    /// The window's client area width in physical pixels.
    ///
    /// See [`WindowResolution`] for an explanation about logical/physical sizes.
    #[inline]
    pub fn physical_width(&self) -> u32 {
        self.resolution.physical_width()
    }

    /// The window's client area height in physical pixels.
    ///
    /// See [`WindowResolution`] for an explanation about logical/physical sizes.
    #[inline]
    pub fn physical_height(&self) -> u32 {
        self.resolution.physical_height()
    }

    /// The window's scale factor.
    ///
    /// Ratio of physical size to logical size, see [`WindowResolution`].
    #[inline]
    pub fn scale_factor(&self) -> f64 {
        self.resolution.scale_factor()
    }

    /// The cursor position in this window in logical pixels.
    ///
    /// Returns `None` if the cursor is outside the window area.
    ///
    /// See [`WindowResolution`] for an explanation about logical/physical sizes.
    #[inline]
    pub fn cursor_position(&self) -> Option<Vec2> {
        self.internal
            .physical_cursor_position
            .map(|position| (position / self.scale_factor()).as_vec2())
    }

    /// The cursor position in this window in physical pixels.
    ///
    /// Returns `None` if the cursor is outside the window area.
    ///
    /// See [`WindowResolution`] for an explanation about logical/physical sizes.
    #[inline]
    pub fn physical_cursor_position(&self) -> Option<Vec2> {
        self.internal
            .physical_cursor_position
            .map(|position| position.as_vec2())
    }

    /// Set the cursor position in this window in logical pixels.
    ///
    /// See [`WindowResolution`] for an explanation about logical/physical sizes.
    pub fn set_cursor_position(&mut self, position: Option<Vec2>) {
        self.internal.physical_cursor_position =
            position.map(|p| p.as_dvec2() * self.scale_factor());
    }

    /// Set the cursor position in this window in physical pixels.
    ///
    /// See [`WindowResolution`] for an explanation about logical/physical sizes.
    pub fn set_physical_cursor_position(&mut self, position: Option<DVec2>) {
        self.internal.physical_cursor_position = position;
    }
}

/// Stores internal [`Window`] state that isn't directly accessible.
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct InternalWindowState {
    /// If this is true then next frame we will ask to minimize the window.
    minimize_request: Option<bool>,
    /// If this is true then next frame we will ask to maximize/un-maximize the window depending on `maximized`.
    maximize_request: Option<bool>,
    /// Unscaled cursor position.
    physical_cursor_position: Option<DVec2>,
}

impl InternalWindowState {
    /// Consumes the current maximize request, if it exists. This should only be called by window backends.
    pub fn take_maximize_request(&mut self) -> Option<bool> {
        self.maximize_request.take()
    }

    /// Consumes the current minimize request, if it exists. This should only be called by window backends.
    pub fn take_minimize_request(&mut self) -> Option<bool> {
        self.minimize_request.take()
    }
}

/// The [`Window`] theme variant to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowTheme {
    /// Use the light variant.
    Light,

    /// Use the dark variant.
    Dark,
}

impl From<WindowTheme> for winit::window::Theme {
    fn from(value: WindowTheme) -> Self {
        match value {
            WindowTheme::Light => Self::Light,
            WindowTheme::Dark => Self::Dark,
        }
    }
}