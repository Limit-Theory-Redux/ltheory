//! Per-frame stats snapshot shared with the stats dashboard server.
//!
//! The render thread publishes a [`RenderStats`] snapshot once per frame (via
//! the executor's `SwapBuffers` reply). The main thread combines it with its
//! own measurements (frame-end wait, mid-frame channel send stalls, channel
//! occupancy, frames in flight) into a single [`StatsSnapshot`] and pushes it
//! into the sink. The HTTP dashboard server (feature `stats-server`) reads the
//! latest snapshot from the same sink; when the feature is off the sink is
//! `None` and the only cost is the few `u64` accumulations.

#[cfg(not(feature = "immediate"))]
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::render::uniform_dedup_skips;
use super::Renderer;

/// Shared sink holding the most recent [`StatsSnapshot`].
pub type StatsSink = Arc<Mutex<StatsSnapshot>>;

/// Combined main-thread + render-thread statistics for one frame.
#[derive(Debug, Clone, Default)]
pub struct StatsSnapshot {
    // --- render thread (from RenderStats) ---
    pub commands_processed: u64,
    pub draw_calls: u64,
    pub state_changes: u64,
    pub frame_count: u64,
    pub last_frame_time_us: u64,
    pub commands_last_frame: u64,
    pub draw_calls_last_frame: u64,
    pub state_changes_last_frame: u64,
    pub present_wait_us: u64,
    pub texture_bind_calls_last_frame: u64,
    pub texture_binds_skipped_last_frame: u64,
    pub texture_cache_invalidations_last_frame: u64,
    pub texture_binds_skipped: u64,
    /// Texture-cache invalidations in the last frame, by source
    pub texture_invalidations_on_shader_bind_last_frame: u64,
    pub texture_invalidations_on_shader_unbind_last_frame: u64,
    /// Draw calls in the last frame, split by kind
    pub draw_mesh_calls_last_frame: u64,
    pub draw_immediate_calls_last_frame: u64,
    pub draw_instanced_calls_last_frame: u64,
    /// Vertices submitted via DrawImmediate in the last frame
    pub immediate_vertices_last_frame: u64,
    /// Instance-data items submitted in the last frame
    pub instanced_data_items_last_frame: u64,
    /// Total vertices submitted to the GPU in the last frame (index_count,
    /// or index_count * instance_count for instanced draws)
    pub vertices_drawn_last_frame: u64,
    /// Uniform-location cache hits vs driver round-trips in the last frame
    pub uniform_cache_hits_last_frame: u64,
    pub uniform_cache_misses_last_frame: u64,
    /// Command counts per category in the last frame (CmdCategory order)
    pub category_counts_last_frame: [u64; 12],
    /// Executor time per category in the last frame, microseconds
    /// (all zero when the dashboard isn't active; timing is opt-in)
    pub category_time_us_last_frame: [u64; 12],
    /// Render-thread producer-starvation (blocked in recv), last frame
    pub recv_wait_us_last_frame: u64,
    pub recv_wait_count_last_frame: u64,
    /// Shader churn: binds, redundant binds (same program), distinct programs
    pub shader_bind_commands_last_frame: u64,
    pub shader_redundant_binds_last_frame: u64,
    pub shader_distinct_programs_last_frame: u64,
    /// Uniform sends skipped by the per-shader value dedup last frame - the
    /// Lua→Rust crossings that were paid but produced no command. Shows the
    /// hidden producer cost the command count doesn't capture.
    pub uniform_dedup_skips_last_frame: u64,
    // --- main thread (from Renderer) ---
    /// Time blocked in `end_frame_triple_buffered` (frame-end pacing wait)
    pub main_thread_wait_us: u64,
    /// Time blocked in `submit()` on a full command channel, this frame
    pub send_blocked_us_last_frame: u64,
    /// Number of `submit()` calls that blocked this frame
    pub send_block_count_last_frame: u64,
    /// Highest command-channel occupancy observed this frame (capacity 8192)
    pub channel_high_water: u64,
    /// Frames submitted but not yet rendered (triple-buffer depth)
    pub frames_in_flight: u64,
}

impl Renderer {
    /// Attach a stats sink; the renderer pushes one snapshot per frame into it.
    #[cfg(not(feature = "immediate"))]
    pub fn attach_stats_sink(&mut self, sink: StatsSink) {
        self.stats_sink = Some(sink);
        // Enable per-category executor timing (dashboard mode). The executor
        // shares this flag via an Arc, so this works even though the executor
        // lives on the render thread.
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

        let stats = self.get_stats();

        let snapshot = StatsSnapshot {
            commands_processed: stats.commands_processed,
            draw_calls: stats.draw_calls,
            state_changes: stats.state_changes,
            frame_count: stats.frame_count,
            last_frame_time_us: stats.last_frame_time_us,
            commands_last_frame: stats.commands_last_frame,
            draw_calls_last_frame: stats.draw_calls_last_frame,
            state_changes_last_frame: stats.state_changes_last_frame,
            present_wait_us: stats.present_wait_us,
            texture_bind_calls_last_frame: stats.texture_bind_calls_last_frame,
            texture_binds_skipped_last_frame: stats.texture_binds_skipped_last_frame,
            texture_cache_invalidations_last_frame: stats.texture_cache_invalidations_last_frame,
            texture_binds_skipped: stats.texture_binds_skipped,
            texture_invalidations_on_shader_bind_last_frame:
                stats.texture_invalidations_on_shader_bind_last_frame,
            texture_invalidations_on_shader_unbind_last_frame:
                stats.texture_invalidations_on_shader_unbind_last_frame,
            draw_mesh_calls_last_frame: stats.draw_mesh_calls_last_frame,
            draw_immediate_calls_last_frame: stats.draw_immediate_calls_last_frame,
            draw_instanced_calls_last_frame: stats.draw_instanced_calls_last_frame,
            immediate_vertices_last_frame: stats.immediate_vertices_last_frame,
            instanced_data_items_last_frame: stats.instanced_data_items_last_frame,
            vertices_drawn_last_frame: stats.vertices_drawn_last_frame,
            uniform_cache_hits_last_frame: stats.uniform_cache_hits_last_frame,
            uniform_cache_misses_last_frame: stats.uniform_cache_misses_last_frame,
            category_counts_last_frame: stats.category_counts_last_frame,
            category_time_us_last_frame: stats.category_time_us_last_frame,
            recv_wait_us_last_frame: stats.recv_wait_us_last_frame,
            recv_wait_count_last_frame: stats.recv_wait_count_last_frame,
            shader_bind_commands_last_frame: stats.shader_bind_commands_last_frame,
            shader_redundant_binds_last_frame: stats.shader_redundant_binds_last_frame,
            shader_distinct_programs_last_frame: stats.shader_distinct_programs_last_frame,
            uniform_dedup_skips_last_frame: uniform_dedup_skips(),
            main_thread_wait_us: self.main_thread_wait_us,
            send_blocked_us_last_frame: self.send_blocked_us_last_frame,
            send_block_count_last_frame: self.send_block_count_last_frame,
            channel_high_water: self.channel_high_water,
            frames_in_flight: self.get_frames_in_flight(),
        };

        if let Ok(mut guard) = sink.lock() {
            *guard = snapshot;
        }
    }
}
