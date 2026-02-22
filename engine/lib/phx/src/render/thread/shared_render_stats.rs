use std::sync::atomic::{AtomicU64, Ordering};

use crate::render::RenderStats;

/// Shared statistics accessible from main thread
#[derive(Debug)]
pub struct SharedRenderStats {
    pub commands_processed: AtomicU64,
    pub draw_calls: AtomicU64,
    pub state_changes: AtomicU64,
    pub frame_count: AtomicU64,
    /// Last frame render time in microseconds
    pub last_frame_time_us: AtomicU64,
    /// Commands processed in the last frame
    pub commands_last_frame: AtomicU64,
    /// Draw calls in the last frame
    pub draw_calls_last_frame: AtomicU64,
    /// Texture binds skipped due to caching (cumulative)
    pub texture_binds_skipped: AtomicU64,
    /// Main thread wait time in microseconds (time spent waiting for render thread)
    pub main_thread_wait_us: AtomicU64,
}

impl Default for SharedRenderStats {
    fn default() -> Self {
        Self {
            commands_processed: AtomicU64::default(),
            draw_calls: AtomicU64::default(),
            state_changes: AtomicU64::default(),
            frame_count: AtomicU64::default(),
            last_frame_time_us: AtomicU64::default(),
            commands_last_frame: AtomicU64::default(),
            draw_calls_last_frame: AtomicU64::default(),
            texture_binds_skipped: AtomicU64::default(),
            main_thread_wait_us: AtomicU64::default(),
        }
    }
}

impl SharedRenderStats {
    pub fn snapshot(&self) -> RenderStats {
        RenderStats {
            commands_processed: self.commands_processed.load(Ordering::Relaxed),
            draw_calls: self.draw_calls.load(Ordering::Relaxed),
            state_changes: self.state_changes.load(Ordering::Relaxed),
            frame_count: self.frame_count.load(Ordering::Relaxed),
        }
    }
}
