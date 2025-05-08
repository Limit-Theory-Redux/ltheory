use tracing::warn;

/// The size limits on a [`Window`].
///
/// These values are measured in logical pixels (see [`WindowResolution`]), so the user's
/// scale factor does affect the size limits on the window.
///
/// Please note that if the window is resizable, then when the window is
/// maximized it may have a size outside of these limits. The functionality
/// required to disable maximizing is not yet exposed by winit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowResizeConstraints {
    /// The minimum width the window can have.
    pub min_width: f32,
    /// The minimum height the window can have.
    pub min_height: f32,
    /// The maximum width the window can have.
    pub max_width: f32,
    /// The maximum height the window can have.
    pub max_height: f32,
}

impl Default for WindowResizeConstraints {
    fn default() -> Self {
        Self {
            min_width: 180.,
            min_height: 120.,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
        }
    }
}

impl WindowResizeConstraints {
    /// Checks if the constraints are valid.
    ///
    /// Will output warnings if it isn't.
    #[must_use]
    pub fn check_constraints(&self) -> Self {
        let &WindowResizeConstraints {
            mut min_width,
            mut min_height,
            mut max_width,
            mut max_height,
        } = self;
        min_width = min_width.max(1.);
        min_height = min_height.max(1.);
        if max_width < min_width {
            warn!(
                "The given maximum width {} is smaller than the minimum width {}",
                max_width, min_width
            );
            max_width = min_width;
        }
        if max_height < min_height {
            warn!(
                "The given maximum height {} is smaller than the minimum height {}",
                max_height, min_height
            );
            max_height = min_height;
        }
        WindowResizeConstraints {
            min_width,
            min_height,
            max_width,
            max_height,
        }
    }
}
