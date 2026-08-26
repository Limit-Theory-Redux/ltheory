//! `Renderer` is the single context every GL-touching type submits
//! [`RenderCommand`]s through - see `ai/multithreaded_rendering.md`. It has
//! two compile-time bodies behind the identical type name, public API and
//! FFI surface, declared as sibling modules in `render/thread/mod.rs`
//! (`renderer_threaded` by default, `renderer_immediate` under the
//! `immediate` feature) so the `#[luajit_ffi_gen::luajit_ffi] impl Renderer`
//! in `renderer_ffi.rs` only ever calls methods defined here and never
//! needs to know which backend is active. This file holds what both share.

use tracing::error;

use crate::render::{CmdPrimitiveType, GenericUniformName, RenderBatch, RenderCommand};

/// A snapshot of the executor's counters, taken once per frame at
/// `SwapBuffers`. In immediate mode this is read straight off the executor;
/// in threaded mode it travels over a channel and is cached until the next
/// one arrives (see `renderer_threaded.rs`).
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    pub commands_processed: u64,
    pub draw_calls: u64,
    pub state_changes: u64,
    pub frame_count: u64,
    /// Last frame render time in microseconds
    pub last_frame_time_us: u64,
    /// Commands processed in the last frame
    pub commands_last_frame: u64,
    /// Draw calls in the last frame
    pub draw_calls_last_frame: u64,
    /// State changes in the last frame (per-frame equivalent of the
    /// cumulative counter)
    pub state_changes_last_frame: u64,
    /// Time blocked inside `swap_buffers` (vsync wait) in the last frame
    pub present_wait_us: u64,
    /// Texture binds actually issued in the last frame (cache misses)
    pub texture_bind_calls_last_frame: u64,
    /// Texture binds skipped due to caching in the last frame
    pub texture_binds_skipped_last_frame: u64,
    /// Texture cache invalidations in the last frame
    pub texture_cache_invalidations_last_frame: u64,
    /// Texture cache invalidations in the last frame, by source
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
    /// Total vertices submitted to the GPU in the last frame (index_count
    /// for plain draws, index_count * instance_count for instanced draws).
    /// The true GPU vertex load - items * per-mesh vertex count - is what
    /// sync/frame time actually reflects for dense instanced scenes.
    pub vertices_drawn_last_frame: u64,
    /// Uniform-location cache hits vs driver round-trips in the last frame
    pub uniform_cache_hits_last_frame: u64,
    pub uniform_cache_misses_last_frame: u64,
    /// Command counts per category in the last frame (CommandCategory order)
    pub category_counts_last_frame: [u64; 12],
    /// Executor time per category in the last frame, microseconds
    /// (all zero when the dashboard isn't active; timing is opt-in)
    pub category_time_us_last_frame: [u64; 12],
    /// Time the render thread was blocked waiting for commands (producer
    /// starvation), last frame, microseconds + number of starvation waits
    pub recv_wait_us_last_frame: u64,
    pub recv_wait_count_last_frame: u64,
    /// Shader churn: BindShader commands last frame, how many hit an already
    /// bound program (redundant), and how many distinct programs were bound.
    pub shader_bind_commands_last_frame: u64,
    pub shader_redundant_binds_last_frame: u64,
    pub shader_distinct_programs_last_frame: u64,
    /// Texture binds skipped due to caching (cumulative)
    pub texture_binds_skipped: u64,
}

/// Run the active batch's accumulated entities through frustum culling and
/// sorting, and append the resulting draw commands to `command_buffer`.
///
/// Shared between both `Renderer` backends: this is pure CPU bookkeeping with
/// no GL or channel involvement, so there is nothing backend-specific about
/// it and no reason to maintain two copies.
pub(in crate::render::thread) fn process_batch_intern(
    active_batch: &mut Option<RenderBatch>,
    command_buffer: &mut Vec<RenderCommand>,
) {
    let Some(batch) = active_batch else {
        error!("There is no active batch started. Use begin_batch() to start it.");
        return;
    };

    batch.stats.total_entities = batch.entities.len() as u32;
    batch.stats.entities_visible = 0;
    batch.stats.entities_culled = 0;

    // Sort entities by sort key for better batching
    batch.entities.sort_by_key(|e| e.sort_key);

    let mut current_shader = None;

    for entity in batch.entities.drain(..) {
        // Frustum culling
        if !batch
            .camera
            .sphere_in_frustum(entity.bounds_center, entity.bounds_radius)
        {
            batch.stats.entities_culled += 1;
            continue;
        }

        batch.stats.entities_visible += 1;

        // Bind shader if changed
        if current_shader != Some(entity.shader_id) {
            command_buffer.push(RenderCommand::BindShaderByResource {
                id: entity.shader_id,
                shader_key: None,
            });
            current_shader = Some(entity.shader_id);
        }

        // Set per-draw transform uniforms. View/projection come from
        // `CameraUBO`, bound once per frame - `mWorld`/`mWorldIT` are the
        // only per-draw uniforms this engine's shaders expect (see
        // `res/shader/include/vertex.glsl`, `res/shader/include/camera_ubo.glsl`).
        command_buffer.push(RenderCommand::SetUniformMat4ByGenericName {
            name: GenericUniformName::MWorld,
            value: entity.transform.to_cols_array(),
        });
        command_buffer.push(RenderCommand::SetUniformMat4ByGenericName {
            name: GenericUniformName::MWorldIT,
            value: entity.transform.inverse().transpose().to_cols_array(),
        });

        // Draw call
        command_buffer.push(RenderCommand::DrawMeshByResource {
            id: entity.mesh_id,
            index_count: entity.index_count,
            primitive: CmdPrimitiveType::Triangles,
        });
    }

    batch.stats.batches_processed += 1;
}
