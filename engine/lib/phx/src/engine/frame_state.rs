use std::time::{Duration, Instant};

/// Stores state that must persist between frames.
pub struct FrameState {
    /// Tracks whether or not the application is active or suspended.
    pub active: bool,
    /// Tracks whether or not an event has occurred this frame that would trigger an update in low
    /// power mode. Should be reset at the end of every frame.
    pub low_power_event: bool,
    /// Tracks whether the event loop was started this frame because of a redraw request.
    pub redraw_request_sent: bool,
    /// Tracks if the event loop was started this frame because of a [`ControlFlow::WaitUntil`]
    /// timeout.
    pub timeout_reached: bool,
    pub last_update: Instant,
    pub delta_time: Duration,
}

impl Default for FrameState {
    fn default() -> Self {
        Self {
            active: false,
            low_power_event: false,
            redraw_request_sent: false,
            timeout_reached: false,
            last_update: Instant::now(),
            delta_time: Duration::from_secs(0),
        }
    }
}
