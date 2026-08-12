//! `Renderer` is the single context every GL-touching type submits
//! [`RenderCommand`]s through - see `ai/multithreaded_rendering.md`. It has
//! two compile-time bodies behind the identical type name, public API and
//! FFI surface, declared as sibling modules in `render/thread/mod.rs`
//! (`renderer_threaded` by default, `renderer_immediate` under the
//! `immediate` feature) so the `#[luajit_ffi_gen::luajit_ffi] impl Renderer`
//! in `renderer_ffi.rs` only ever calls methods defined here and never
//! needs to know which backend is active. This file holds what both share.

use tracing::error;

use crate::render::{CmdPrimitiveType, RenderBatch, RenderCommand, GenericUniformName};

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
