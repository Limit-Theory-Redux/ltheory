mod composite_alpha_mode;
mod cursor;
mod glutin_render;
mod monitor_selection;
mod present_mode;
mod window_level;
mod window_mode;
mod window_position;
mod window_resize_constraints;
mod window_resolution;
mod winit_converters;
mod winit_window;
mod winit_windows;

pub use composite_alpha_mode::*;
pub use cursor::*;
pub use glutin_render::*;
pub use monitor_selection::*;
pub use present_mode::*;
pub use window_level::*;
pub use window_mode::*;
pub use window_position::*;
pub use window_resize_constraints::*;
pub use window_resolution::*;
pub use winit_converters::*;
pub use winit_window::*;
pub use winit_windows::*;

use internal::ConvertIntoString;

use super::*;
use crate::common::*;
use crate::math::*;
use crate::render::*;
use crate::system::*;

/// The cached state of the window so we can check which properties were changed from within the app.
#[derive(Debug, Clone)]
pub struct CachedWindow {
    pub window: Window,
}

#[derive(Debug, Clone, Copy)]
pub enum WindowState {
    Suspended,
    Resumed,
}

/// The defining [`Component`] for window entities,
/// storing information about how it should appear and behave.
///
/// Each window corresponds to an entity, and is uniquely identified by the value of their [`Entity`].
/// When the [`Window`] component is added to an entity, a new window will be opened.
/// When it is removed or the entity is despawned, the window will close.
///
/// This component will reflect the current state of the window and can be modified to change this state.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct Window {
    /// Stores the title of the window.
    pub title: String,
    /// The cursor of this window.
    pub cursor: Cursor,
    /// What presentation mode to give the window.
    pub present_mode: PresentMode,
    /// Which fullscreen or windowing mode should be used.
    pub mode: WindowMode,
    /// Contains window suspended or resumed state if it was set,
    pub state: Option<WindowState>,
    /// Where the window should be placed.
    pub position: WindowPosition,
    /// What resolution the window should have.
    pub resolution: WindowResolution,
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
    /// from performing their default behavior while the LTR app has focus.
    ///
    /// This value has no effect on non-web platforms.
    pub prevent_default_event_handling: bool,
    /// Stores internal state that isn't directly accessible.
    pub internal: InternalWindowState,
    /// Should the window use Input Method Editor?
    ///
    /// If enabled, the window will receive [`Ime`](crate::Ime) events instead of
    /// [`ReceivedCharacter`](crate::ReceivedCharacter) or
    /// [`KeyboardInput`](crate::KeyboardInput).
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
            state: None,
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

#[luajit_ffi_gen::luajit_ffi]
impl Window {
    pub fn begin_draw(&self) {
        let size = self.size();

        unsafe {
            Viewport_Push(0, 0, size.x as i32, size.y as i32, true);
        }
    }

    pub fn end_draw(&self) {
        unsafe {
            Viewport_Pop();
        }
    }

    /// The window title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set the window title.
    pub fn set_title(&mut self, title: &str) {
        self.title = title.into();
    }

    /// The window cursor.
    pub fn cursor(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    /// The window present mode.
    pub fn present_mode(&self) -> PresentMode {
        self.present_mode
    }

    /// Set window present mode.
    pub fn set_present_mode(&mut self, present_mode: PresentMode) {
        self.present_mode = present_mode
    }

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

    /// The window's client position in physical pixels.
    ///
    /// See [`WindowPosition`] for an explanation about logical/physical sizes.
    #[inline]
    pub fn position(&self) -> IVec2 {
        // TODO: fix Automatic and Centered cases
        match self.position {
            WindowPosition::Automatic => IVec2::default(),
            WindowPosition::Centered(_) => IVec2::default(),
            WindowPosition::At(pos) => pos,
        }
    }

    /// Set the window's client position in the center of the current monitor.
    #[inline]
    pub fn set_centered_position(&mut self) {
        self.position = WindowPosition::Centered(MonitorSelection::Current);
    }

    /// Set the window's client position in physical pixels.
    ///
    /// See [`WindowPosition`] for an explanation about logical/physical sizes.
    #[inline]
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.position = WindowPosition::At(IVec2 { x, y });
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

    /// The window's client area size in logical pixels.
    ///
    /// See [`WindowResolution`] for an explanation about logical/physical sizes.
    #[inline]
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.resolution.width(), self.resolution.height())
    }

    /// Set the window's client area size in logical pixels.
    ///
    /// See [`WindowResolution`] for an explanation about logical/physical sizes.
    #[inline]
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.resolution.set(width, height);
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

    /// The window's client area size in physical pixels.
    ///
    /// See [`WindowResolution`] for an explanation about logical/physical sizes.
    #[inline]
    pub fn physical_size(&self) -> IVec2 {
        IVec2::new(
            self.resolution.physical_width() as i32, // TODO: introduce UVec2/Vec2u types instead of casting
            self.resolution.physical_height() as i32,
        )
    }

    /// Set the window's client area size in physical pixels.
    ///
    /// See [`WindowResolution`] for an explanation about logical/physical sizes.
    #[inline]
    pub fn set_physical_size(&mut self, width: i32, height: i32) {
        // TODO: introduce UVec2/Vec2u types instead of casting
        self.resolution
            .set_physical_resolution(width as u32, height as u32);
    }

    /// Is the window resizable?
    pub fn is_resizable(&self) -> bool {
        self.resizable
    }

    /// Should the window be resizable?
    pub fn set_resizable(&mut self, resizable: bool) {
        self.resizable = resizable;
    }

    /// Has the window decorations?
    pub fn has_decorations(&self) -> bool {
        self.decorations
    }

    /// Should the window have decorations?
    pub fn set_decorations(&mut self, decorations: bool) {
        self.decorations = decorations;
    }

    /// Is the window transparent?
    pub fn is_transparent(&self) -> bool {
        self.transparent
    }

    /// Should the window be transparent?
    pub fn set_transparent(&mut self, transparent: bool) {
        self.transparent = transparent;
    }

    /// Is the window focused?
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Should the window be focused?
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn set_fullscreen(&mut self, fs: bool) {
        self.mode = if fs {
            WindowMode::Fullscreen
        } else {
            WindowMode::Windowed
        };
    }

    pub fn toggle_fullscreen(&mut self) {
        if self.mode == WindowMode::Fullscreen {
            self.mode = WindowMode::Windowed;
        } else {
            self.mode = WindowMode::Fullscreen;
        }
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

    /// Set the cursor position in this window in logical pixels.
    ///
    /// See [`WindowResolution`] for an explanation about logical/physical sizes.
    pub fn set_cursor_position(&mut self, position: Option<Vec2>) {
        self.internal.physical_cursor_position =
            position.map(|p| p.as_dvec2() * self.scale_factor());
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
