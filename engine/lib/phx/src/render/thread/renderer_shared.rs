//! Common code for threaded and immediate renderers.

use tracing::error;

use crate::render::{
    CmdPrimitiveType, GenericUniformName, RenderBatch, RenderCommand, Renderer, ResourceHandle,
    ResourceId,
};

impl Renderer {
    /// Mint a new GPU resource: a unique `ResourceId` bundled with the means
    /// to destroy it (see `ResourceHandle`). This is the only way to obtain
    /// either, so a resource can never exist without its destructor wired up.
    pub fn create_resource(&mut self) -> ResourceHandle {
        let id = ResourceId(self.data.next_resource_id);
        self.data.next_resource_id += 1;
        ResourceHandle::new(id, self.data.destroy_tx.clone())
    }

    pub fn process_batch(&mut self) {
        Self::process_batch_intern(&mut self.data.active_batch, &mut self.data.command_buffer);
    }

    /// Run the active batch's accumulated entities through frustum culling and
    /// sorting, and append the resulting draw commands to `command_buffer`.
    ///
    /// Shared between both `Renderer` backends: this is pure CPU bookkeeping with
    /// no GL or channel involvement, so there is nothing backend-specific about
    /// it and no reason to maintain two copies.
    fn process_batch_intern(
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
}
