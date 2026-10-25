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

impl From<PresentMode> for wgpu::PresentMode {
    fn from(value: PresentMode) -> Self {
        match value {
            // Fifo waits for the display refresh; Immediate presents at once.
            PresentMode::Vsync => Self::Fifo,
            PresentMode::NoVsync => Self::Immediate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn present_mode_maps_to_wgpu() {
        assert_eq!(
            wgpu::PresentMode::from(PresentMode::Vsync),
            wgpu::PresentMode::Fifo
        );
        assert_eq!(
            wgpu::PresentMode::from(PresentMode::NoVsync),
            wgpu::PresentMode::Immediate
        );
    }
}
