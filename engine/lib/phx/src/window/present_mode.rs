use std::num::NonZeroU32;

/// The presentation mode specifies when a frame is presented to the window.
///
/// `Vsync` will cap the framerate by the display refresh rate, while `NoVsync` will present as fast as possible.
#[luajit_ffi_gen::luajit_ffi]
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PresentMode {
    #[default]
    Vsync = 0,
    NoVsync = 1,
}

impl From<PresentMode> for glutin::surface::SwapInterval {
    fn from(value: PresentMode) -> Self {
        match value {
            PresentMode::Vsync => Self::Wait(NonZeroU32::new(1).unwrap()),
            PresentMode::NoVsync => Self::DontWait,
        }
    }
}

/// Converts the `ltr --present-mode` value carried in `EngineSettings`
/// (`internal::PresentMode`, not this type - see its doc comment for why)
/// into the real present mode.
impl From<internal::PresentMode> for PresentMode {
    fn from(value: internal::PresentMode) -> Self {
        match value {
            internal::PresentMode::Vsync => Self::Vsync,
            internal::PresentMode::NoVsync => Self::NoVsync,
        }
    }
}
