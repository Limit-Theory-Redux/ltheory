//! `Renderer` is the single context every GL-touching type submits
//! [`RenderCommand`]s through - see `ai/multithreaded_rendering.md`. It has
//! two compile-time bodies behind the identical type name, public API and
//! FFI surface, declared as sibling modules in `render/thread/mod.rs`
//! (`renderer_threaded` by default, `renderer_immediate` under the
//! `immediate` feature) so the `#[luajit_ffi_gen::luajit_ffi] impl Renderer`
//! in `renderer_ffi.rs` only ever calls methods defined here and never
//! needs to know which backend is active. This file holds what both share.

/// A snapshot of the executor's counters, taken once per frame at
/// `SwapBuffers`. In immediate mode this is read straight off the executor;
/// in threaded mode it travels over a channel and is cached until the next
/// one arrives (see `renderer_threaded.rs`).
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    pub commands_processed: u64,
    pub draw_calls_cumulative: u64,
    pub state_changes_cumulative: u64,
    pub frame_count: u64,
    /// Last frame render time in microseconds
    pub last_frame_time_us: u64,
    /// Commands processed in the last frame
    pub commands: u64,
    /// Draw calls in the last frame
    pub draw_calls: u64,
    /// State changes in the last frame (per-frame equivalent of the
    /// cumulative counter)
    pub state_changes: u64,
    /// Time blocked inside `swap_buffers` (vsync wait) in the last frame
    pub present_wait_us: u64,
    /// Texture binds actually issued in the last frame (cache misses)
    pub texture_bind_calls: u64,
    /// Texture binds skipped due to caching in the last frame
    pub texture_binds_skipped: u64,
    /// Texture cache invalidations in the last frame
    pub texture_cache_invalidations: u64,
    /// Texture cache invalidations in the last frame, by source
    pub texture_invalidations_on_shader_bind: u64,
    pub texture_invalidations_on_shader_unbind: u64,
    /// Draw calls in the last frame, split by kind
    pub draw_mesh_calls: u64,
    pub draw_immediate_calls: u64,
    pub draw_instanced_calls: u64,
    /// Vertices submitted via DrawImmediate in the last frame
    pub immediate_vertices: u64,
    /// Instance-data items submitted in the last frame
    pub instanced_data_items: u64,
    /// Total vertices submitted to the GPU in the last frame (index_count
    /// for plain draws, index_count * instance_count for instanced draws).
    /// The true GPU vertex load - items * per-mesh vertex count - is what
    /// sync/frame time actually reflects for dense instanced scenes.
    pub vertices_drawn: u64,
    /// Uniform-location cache hits vs driver round-trips in the last frame
    pub uniform_cache_hits: u64,
    pub uniform_cache_misses: u64,
    /// Command counts per category in the last frame (CommandCategory order)
    pub category_counts: [u64; 12],
    /// Executor time per category in the last frame, microseconds
    /// (all zero when the dashboard isn't active; timing is opt-in)
    pub category_time_us: [u64; 12],
    /// Time the render thread was blocked waiting for commands (producer
    /// starvation), last frame, microseconds + number of starvation waits
    pub recv_wait_us: u64,
    pub recv_wait_count: u64,
    /// Shader churn: BindShader commands last frame, how many hit an already
    /// bound program (redundant), and how many distinct programs were bound.
    pub shader_bind_commands: u64,
    pub shader_redundant_binds: u64,
    pub shader_distinct_programs: u64,
    /// Texture binds skipped due to caching (cumulative)
    pub texture_binds_skipped_cumulative: u64,
}
