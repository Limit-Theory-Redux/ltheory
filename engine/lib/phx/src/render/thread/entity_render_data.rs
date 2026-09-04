use glam::{Mat4, Vec3};

use crate::render::ResourceId;

/// Data for a single entity to be prepared for rendering
#[derive(Clone, Debug)]
pub struct EntityRenderData {
    /// World transform matrix
    pub transform: Mat4,
    /// Bounding sphere center (world space)
    pub bounds_center: Vec3,
    /// Bounding sphere radius
    pub bounds_radius: f32,
    /// GPU resource id of the mesh to draw
    pub mesh_id: ResourceId,
    /// Number of indices in mesh
    pub index_count: i32,
    /// GPU resource id of the shader program to bind
    pub shader_id: ResourceId,
    /// Sort key for render ordering (lower = render first)
    pub sort_key: u32,
    /// Opaque caller tag, echoed back by `cull_batch`. Unused by the draw path.
    pub user_id: u32,
}
