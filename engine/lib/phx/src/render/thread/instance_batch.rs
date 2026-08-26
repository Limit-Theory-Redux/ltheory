use crate::math::Matrix;
use crate::render::{CmdPrimitiveType, InstanceData, Mesh, Renderer, ResourceId};

/// Accumulates per-instance transforms/colors for a single mesh and draws
/// them in one `glDrawElementsInstanced` call via `DrawInstancedWithData`
/// (`command_executor_gl.rs`'s persistent instance VBO + attribute divisors
/// at locations 4-7/8 - see that file for the GL side, which needs no
/// changes to support this).
pub struct InstanceBatch {
    mesh_id: ResourceId,
    index_count: i32,
    instances: Vec<InstanceData>,
    primitive: CmdPrimitiveType,
}

#[luajit_ffi_gen::luajit_ffi]
impl InstanceBatch {
    /// Binds this batch to `mesh`'s GPU resource (lazily creating it, like
    /// `Mesh::drawBind`) and its current index count. Changing which mesh a
    /// batch draws requires creating a new `InstanceBatch`.
    #[bind(name = "Create")]
    pub fn create(mesh: &mut Mesh, r: &mut Renderer, primitive: CmdPrimitiveType) -> InstanceBatch {
        InstanceBatch {
            mesh_id: ResourceId(mesh.resource_id(r)),
            index_count: mesh.get_index_count(),
            instances: Vec::new(),
            primitive,
        }
    }

    /// Queue one instance. Has no GL effect until `draw`/`flush`.
    pub fn add_instance(&mut self, transform: &Matrix, r: f32, g: f32, b: f32, a: f32) {
        self.instances.push(InstanceData::from_transform_color(
            &transform.to_cols_array(),
            r,
            g,
            b,
            a,
        ));
    }

    /// Draw all queued instances in one instanced draw call. Does not clear
    /// the queue - call `clear` (or use `flush`) to start the next batch.
    pub fn draw(&mut self, r: &mut Renderer) {
        r.draw_instanced_with_data_intern(
            self.mesh_id,
            self.index_count,
            self.instances.clone(),
            self.primitive,
        );
    }

    /// Drop all queued instances without drawing them.
    pub fn clear(&mut self) {
        self.instances.clear();
    }

    /// `draw` followed by `clear` - the common per-frame pattern.
    pub fn flush(&mut self, r: &mut Renderer) {
        self.draw(r);
        self.clear();
    }

    pub fn instance_count(&self) -> i32 {
        self.instances.len() as i32
    }
}
