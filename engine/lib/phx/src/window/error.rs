#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("Cannot extract GL context - current state is not Current")]
    CurrentGlState,
    #[error("Cannot restore GL context - current state is not Undefined")]
    UndefinedGlState,
    #[error("{0}")]
    GlutinError(glutin::error::Error),
    #[cfg(target_os = "macos")]
    #[error("Cannot release GL context on macOS without deadlock - resources leaked")]
    MacOsReleaseContext,
}

impl From<glutin::error::Error> for WindowError {
    fn from(err: glutin::error::Error) -> Self {
        Self::GlutinError(err)
    }
}
