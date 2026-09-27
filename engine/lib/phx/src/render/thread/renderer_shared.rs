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

        // Frustum-cull and sort by sort key for better batching. Bumps
        // batch.stats accordingly; see `RenderBatch::cull_and_sort`.
        batch.cull_and_sort();
        let order = batch.visible().to_vec();

        let mut current_shader = None;

        for i in order {
            let entity = &batch.entities[i as usize];

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

        // Matches the old `drain(..)` semantics: the batch is empty once
        // processed.
        batch.entities.clear();
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::*;
    use crate::math::Matrix;

    fn test_batch() -> RenderBatch {
        let view = Matrix::look_at(&Vec3::ZERO, &Vec3::new(0.0, 0.0, -1.0), &Vec3::Y);
        let proj = Matrix::perspective(90.0, 1.0, 0.1, 1000.0);
        RenderBatch::new(&view, &proj, Vec3::ZERO)
    }

    /// Regression guard for routing `process_batch_intern` through
    /// `cull_and_sort`/`visible()` instead of the old sort+drain: the
    /// emitted command sequence must be unchanged - bind on shader change,
    /// mWorld/mWorldIT/draw per surviving entity, in sort-key order.
    #[test]
    fn process_batch_emits_bind_transform_draw_and_dedups_shader() {
        let mut r = Renderer::new_headless();
        r.data.active_batch = Some(test_batch());

        let transform = Matrix::identity();
        let mesh = ResourceId(10);
        let shader_a = ResourceId(1);
        let shader_b = ResourceId(2);

        let batch = r.data.active_batch.as_mut().unwrap();
        // Two entities share shader_a with equal sort keys (adjacent after
        // the stable sort); a third uses shader_b with a later sort key.
        let front = Vec3::new(0.0, 0.0, -10.0);
        batch.add_entity(&transform, front, 1.0, mesh, 3, shader_a, 0, 1);
        batch.add_entity(&transform, front, 1.0, mesh, 3, shader_a, 0, 2);
        batch.add_entity(&transform, front, 1.0, mesh, 3, shader_b, 1, 3);

        r.process_batch();

        let cmds = &r.data.command_buffer;
        assert_eq!(cmds.len(), 11);

        let bind_id = |cmd: &RenderCommand| match cmd {
            RenderCommand::BindShaderByResource { id, .. } => *id,
            other => panic!("expected BindShaderByResource, got {other:?}"),
        };
        let assert_mat4 = |cmd: &RenderCommand, expected: GenericUniformName| match cmd {
            RenderCommand::SetUniformMat4ByGenericName { name, .. } => assert_eq!(*name, expected),
            other => panic!("expected SetUniformMat4ByGenericName, got {other:?}"),
        };
        let assert_draw = |cmd: &RenderCommand| {
            assert!(
                matches!(cmd, RenderCommand::DrawMeshByResource { .. }),
                "expected DrawMeshByResource, got {cmd:?}"
            );
        };

        assert_eq!(bind_id(&cmds[0]), shader_a);
        assert_mat4(&cmds[1], GenericUniformName::MWorld);
        assert_mat4(&cmds[2], GenericUniformName::MWorldIT);
        assert_draw(&cmds[3]);
        // Second entity shares shader_a: no re-bind.
        assert_mat4(&cmds[4], GenericUniformName::MWorld);
        assert_mat4(&cmds[5], GenericUniformName::MWorldIT);
        assert_draw(&cmds[6]);
        assert_eq!(bind_id(&cmds[7]), shader_b);
        assert_mat4(&cmds[8], GenericUniformName::MWorld);
        assert_mat4(&cmds[9], GenericUniformName::MWorldIT);
        assert_draw(&cmds[10]);
    }

    #[test]
    fn process_batch_clears_entities() {
        let mut r = Renderer::new_headless();
        r.data.active_batch = Some(test_batch());

        let transform = Matrix::identity();
        let batch = r.data.active_batch.as_mut().unwrap();
        batch.add_entity(
            &transform,
            Vec3::new(0.0, 0.0, -10.0),
            1.0,
            ResourceId(1),
            3,
            ResourceId(1),
            0,
            0,
        );

        r.process_batch();

        assert!(r.data.active_batch.as_ref().unwrap().entities.is_empty());
    }
}
