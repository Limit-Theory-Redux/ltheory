//! Per-frame stats snapshot shared with the stats dashboard server.
//!
//! The render thread publishes a per-frame [`RenderStats`] snapshot (via the
//! executor's `SwapBuffers` reply). The main thread combines it with its own
//! measurements into a [`StatsSnapshot`] and pushes it into the sink; the
//! HTTP dashboard server reads the latest snapshot from the same sink. The
//! whole publishing path exists only behind the `stats-server` feature, so
//! normal game builds carry none of it - the executor still fills its plain
//! `RenderStats`, which travels the existing SwapBuffers reply regardless.
//!
//! [`RenderStats`] is embedded verbatim instead of being copied field by
//! field: the snapshot adds only the main-thread producer measurements and
//! the publication timestamp on top of what the executor already produces.

use std::sync::{Arc, Mutex};

use super::{RenderStats, Renderer};
use crate::render::uniform_dedup_skips;

/// Shared sink holding the most recent [`StatsSnapshot`].
pub type StatsSink = Arc<Mutex<StatsSnapshot>>;

/// Combined main-thread + render-thread statistics for one frame.
///
/// The render-thread half is embedded as [`RenderStats`] (`snapshot.render`);
/// only the main-thread producer-side measurements live directly here.
#[derive(Debug, Clone, Default)]
pub struct StatsSnapshot {
    /// Publication timestamp (microseconds since the UNIX epoch). The
    /// dashboard uses it for wall-clock-accurate FPS averaging.
    pub server_time_us: u64,
    /// Everything measured on the render thread / GL executor.
    pub render: RenderStats,
    // --- main thread (from Renderer) ---
    /// Time blocked in `end_frame_triple_buffered` (frame-end pacing wait)
    pub main_thread_wait_us: u64,
    /// Time blocked in `submit()` on a full command channel, this frame
    pub send_blocked_us: u64,
    /// Number of `submit()` calls that blocked this frame
    pub send_block_count: u64,
    /// Highest command-channel occupancy observed this frame (capacity 8192)
    pub channel_high_water: u64,
    /// Frames submitted but not yet rendered (triple-buffer depth)
    pub frames_in_flight: u64,
    /// Uniform sends skipped by the per-shader value dedup last frame - the
    /// Lua→Rust crossings that were paid but produced no command. Shows the
    /// hidden producer cost the command count doesn't capture.
    pub uniform_dedup_skips: u64,
}

impl Renderer {
    /// Attach a stats sink; the renderer pushes one snapshot per frame into it.
    #[cfg(not(feature = "immediate"))]
    pub fn attach_stats_sink(&mut self, sink: StatsSink) {
        self.stats_sink = Some(sink);
        // Enable per-category executor timing (dashboard mode). The executor
        // shares this flag via an Arc, so this works even though the executor
        // lives on the render thread.
        use std::sync::atomic::Ordering;
        self.category_timing.store(true, Ordering::Relaxed);
    }

    /// Immediate mode: no render thread to publish from; the dashboard's
    /// /profile.json endpoints (shared profiler) still work, /stats.json is
    /// unavailable. Accepting the sink keeps the engine's stats-server wiring
    /// uniform across both backends.
    #[cfg(feature = "immediate")]
    pub fn attach_stats_sink(&mut self, _sink: StatsSink) {}

    /// Build a snapshot from the current per-frame state and push it into the
    /// sink, if one is attached.
    #[cfg(not(feature = "immediate"))]
    pub(crate) fn publish_stats_snapshot(&mut self) {
        // Clone the sink out so we can take `&mut self` for get_stats()
        let Some(sink) = self.stats_sink.clone() else {
            return;
        };

        let snapshot = StatsSnapshot {
            server_time_us: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_micros() as u64)
                .unwrap_or(0),
            render: self.get_stats(),
            main_thread_wait_us: self.main_thread_wait_us,
            send_blocked_us: self.send_blocked_us,
            send_block_count: self.send_block_count,
            channel_high_water: self.channel_high_water,
            frames_in_flight: self.get_frames_in_flight(),
            uniform_dedup_skips: uniform_dedup_skips(),
        };

        if let Ok(mut guard) = sink.lock() {
            *guard = snapshot;
        }
    }
}
