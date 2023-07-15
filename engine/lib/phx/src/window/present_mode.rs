/// Presentation mode for a [`Window`].
///
/// The presentation mode specifies when a frame is presented to the window. The [`Fifo`]
/// option corresponds to a traditional `VSync`, where the framerate is capped by the
/// display refresh rate. Both [`Immediate`] and [`Mailbox`] are low-latency and are not
/// capped by the refresh rate, but may not be available on all platforms. Tearing
/// may be observed with [`Immediate`] mode, but will not be observed with [`Mailbox`] or
/// [`Fifo`].
///
/// [`AutoVsync`] or [`AutoNoVsync`] will gracefully fallback to [`Fifo`] when unavailable.
///
/// [`Immediate`] or [`Mailbox`] will panic if not supported by the platform.
///
/// [`Fifo`]: PresentMode::Fifo
/// [`Immediate`]: PresentMode::Immediate
/// [`Mailbox`]: PresentMode::Mailbox
/// [`AutoVsync`]: PresentMode::AutoVsync
/// [`AutoNoVsync`]: PresentMode::AutoNoVsync
///
#[repr(C)]
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[doc(alias = "vsync")]
pub enum PresentMode {
    /// Chooses FifoRelaxed -> Fifo based on availability.
    ///
    /// Because of the fallback behavior, it is supported everywhere.
    AutoVsync = 0,
    /// Chooses Immediate -> Mailbox -> Fifo (on web) based on availability.
    ///
    /// Because of the fallback behavior, it is supported everywhere.
    AutoNoVsync = 1,
    /// The presentation engine does **not** wait for a vertical blanking period and
    /// the request is presented immediately. This is a low-latency presentation mode,
    /// but visible tearing may be observed. Not optimal for mobile.
    ///
    /// Selecting this variant will panic if not supported, it is preferred to use
    /// [`PresentMode::AutoNoVsync`].
    Immediate = 2,
    /// The presentation engine waits for the next vertical blanking period to update
    /// the current image, but frames may be submitted without delay. This is a low-latency
    /// presentation mode and visible tearing will **not** be observed. Not optimal for mobile.
    ///
    /// Selecting this variant will panic if not supported, it is preferred to use
    /// [`PresentMode::AutoNoVsync`].
    Mailbox = 3,
    /// The presentation engine waits for the next vertical blanking period to update
    /// the current image. The framerate will be capped at the display refresh rate,
    /// corresponding to the `VSync`. Tearing cannot be observed. Optimal for mobile.
    #[default]
    Fifo = 4, // NOTE: The explicit ordinal values mirror wgpu.
}
