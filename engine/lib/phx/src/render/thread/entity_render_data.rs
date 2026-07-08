use glam::{Mat4, Vec3};

/// Data for a single entity to be prepared for rendering
#[derive(Clone, Debug)]
pub struct EntityRenderData {
    /// Unique entity ID
    pub entity_id: u64,
    /// World transform matrix
    pub transform: Mat4,
    /// Bounding sphere center (world space)
    pub bounds_center: Vec3,
    /// Bounding sphere radius
    pub bounds_radius: f32,
    /// Mesh VAO handle
    pub mesh_vao: u32,
    /// Number of indices in mesh
    pub index_count: i32,
    /// Shader program handle
    pub shader_handle: u32,
    /// MVP uniform location in shader
    pub mvp_location: i32,
    /// Model matrix uniform location
    pub model_location: i32,
    /// Sort key for render ordering (lower = render first)
    pub sort_key: u32,
}
