use glam::{DVec2, Vec2};

/// Controls the size of a [`Window`]
///
/// ## Physical, logical and requested sizes
///
/// There are three sizes associated with a window:
/// - the physical size,
///   which represents the actual height and width in physical pixels
///   the window occupies on the monitor,
/// - the logical size,
///   which represents the size that should be used to scale elements
///   inside the window, measured in logical pixels,
/// - the requested size,
///   measured in logical pixels, which is the value submitted
///   to the API when creating the window, or requesting that it be resized.
///
/// ## Scale factor
///
/// The reason logical size and physical size are separated and can be different
/// is to account for the cases where:
/// - several monitors have different pixel densities,
/// - the user has set up a pixel density preference in its operating system,
/// - the LTR `App` has specified a specific scale factor between both.
///
/// The factor between physical size and logical size can be retrieved with
/// [`WindowResolution::scale_factor`].
///
/// For the first two cases, a scale factor is set automatically by the operating
/// system through the window backend. You can get it with
/// [`WindowResolution::base_scale_factor`].
///
/// For the third case, you can override this automatic scale factor with
/// [`WindowResolution::set_scale_factor_override`].
///
/// ## Requested and obtained sizes
///
/// The logical size should be equal to the requested size after creating/resizing,
/// when possible.
/// The reason the requested size and logical size might be different
/// is because the corresponding physical size might exceed limits (either the
/// size limits of the monitor, or limits defined in [`WindowResizeConstraints`]).
///
/// Note: The requested size is not kept in memory, for example requesting a size
/// too big for the screen, making the logical size different from the requested size,
/// and then setting a scale factor that makes the previous requested size within
/// the limits of the screen will not get back that previous requested size.

#[derive(Debug, Clone, PartialEq)]
pub struct WindowResolution {
    /// Width of the window in physical pixels.
    physical_width: u32,
    /// Height of the window in physical pixels.
    physical_height: u32,
    /// Code-provided ratio of physical size to logical size.
    ///
    /// Should be used instead `scale_factor` when set.
    scale_factor_override: Option<f64>,
    /// OS-provided ratio of physical size to logical size.
    ///
    /// Set automatically depending on the pixel density of the screen.
    scale_factor: f64,
}

impl Default for WindowResolution {
    fn default() -> Self {
        WindowResolution {
            physical_width: 1280,
            physical_height: 720,
            scale_factor_override: None,
            scale_factor: 1.0,
        }
    }
}

impl WindowResolution {
    /// Creates a new [`WindowResolution`].
    pub fn new(logical_width: f32, logical_height: f32) -> Self {
        Self {
            physical_width: logical_width as u32,
            physical_height: logical_height as u32,
            ..Default::default()
        }
    }

    /// Builder method for adding a scale factor override to the resolution.
    pub fn with_scale_factor_override(mut self, scale_factor_override: f64) -> Self {
        self.scale_factor_override = Some(scale_factor_override);
        self
    }

    /// The window's client area width in logical pixels.
    #[inline]
    pub fn width(&self) -> f32 {
        (self.physical_width() as f64 / self.scale_factor()) as f32
    }

    /// The window's client area height in logical pixels.
    #[inline]
    pub fn height(&self) -> f32 {
        (self.physical_height() as f64 / self.scale_factor()) as f32
    }

    /// The window's client area width in physical pixels.
    #[inline]
    pub fn physical_width(&self) -> u32 {
        self.physical_width
    }

    /// The window's client area height in physical pixels.
    #[inline]
    pub fn physical_height(&self) -> u32 {
        self.physical_height
    }

    /// The ratio of physical pixels to logical pixels.
    ///
    /// `physical_pixels = logical_pixels * scale_factor`
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor_override
            .unwrap_or_else(|| self.base_scale_factor())
    }

    /// The window scale factor as reported by the window backend.
    ///
    /// This value is unaffected by [`WindowResolution::scale_factor_override`].
    #[inline]
    pub fn base_scale_factor(&self) -> f64 {
        self.scale_factor
    }

    /// The scale factor set with [`WindowResolution::set_scale_factor_override`].
    ///
    /// This value may be different from the scale factor reported by the window backend.
    #[inline]
    pub fn scale_factor_override(&self) -> Option<f64> {
        self.scale_factor_override
    }

    /// Set the window's logical resolution.
    #[inline]
    pub fn set(&mut self, width: f32, height: f32) {
        self.set_physical_resolution(
            (width as f64 * self.scale_factor()) as u32,
            (height as f64 * self.scale_factor()) as u32,
        );
    }

    /// Set the window's physical resolution.
    ///
    /// This will ignore the scale factor setting, so most of the time you should
    /// prefer to use [`WindowResolution::set`].
    #[inline]
    pub fn set_physical_resolution(&mut self, width: u32, height: u32) {
        self.physical_width = width;
        self.physical_height = height;
    }

    /// Set the window's scale factor, this may get overridden by the backend.
    #[inline]
    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        let (width, height) = (self.width(), self.height());
        self.scale_factor = scale_factor;
        self.set(width, height);
    }

    /// Set the window's scale factor, this will be used over what the backend decides.
    ///
    /// This can change the logical and physical sizes if the resulting physical
    /// size is not within the limits.
    #[inline]
    pub fn set_scale_factor_override(&mut self, scale_factor_override: Option<f64>) {
        let (width, height) = (self.width(), self.height());
        self.scale_factor_override = scale_factor_override;
        self.set(width, height);
    }
}

impl<I> From<(I, I)> for WindowResolution
where
    I: Into<f32>,
{
    fn from((width, height): (I, I)) -> WindowResolution {
        WindowResolution::new(width.into(), height.into())
    }
}

impl<I> From<[I; 2]> for WindowResolution
where
    I: Into<f32>,
{
    fn from([width, height]: [I; 2]) -> WindowResolution {
        WindowResolution::new(width.into(), height.into())
    }
}

impl From<Vec2> for WindowResolution {
    fn from(res: Vec2) -> WindowResolution {
        WindowResolution::new(res.x, res.y)
    }
}

impl From<DVec2> for WindowResolution {
    fn from(res: DVec2) -> WindowResolution {
        WindowResolution::new(res.x as f32, res.y as f32)
    }
}
