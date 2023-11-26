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

impl From<PresentMode> for wgpu::PresentMode {
    fn from(value: PresentMode) -> Self {
        match value {
            PresentMode::Vsync => Self::AutoVsync,
            PresentMode::NoVsync => Self::AutoNoVsync,
        }
    }
}
