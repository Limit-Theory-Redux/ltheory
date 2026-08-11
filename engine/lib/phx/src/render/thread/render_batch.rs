use glam::{Mat4, Vec3};

use crate::render::{BatchStats, CameraRenderData, EntityRenderData, ResourceId};

/// Render batch collector - accumulates entities for worker processing
pub struct RenderBatch {
    /// Accumulated entities for this batch
    pub entities: Vec<EntityRenderData>,
    /// Current camera data
    pub camera: CameraRenderData,
    /// Statistics
    pub stats: BatchStats,
    /// Entity ID counter
    pub next_entity_id: u64,
}

impl RenderBatch {
    /// Begin a new batch - clears accumulated data and set camera for frustum culling
    pub fn new(
        view: &[f32; 16],
        projection: &[f32; 16],
        eye_x: f32,
        eye_y: f32,
        eye_z: f32,
    ) -> Self {
        let view_mat = Mat4::from_cols_array(view);
        let proj_mat = Mat4::from_cols_array(projection);
        let position = Vec3::new(eye_x, eye_y, eye_z);

        Self {
            entities: Vec::with_capacity(1024),
            camera: CameraRenderData::new(view_mat, proj_mat, position),
            stats: BatchStats::default(),
            next_entity_id: 1,
        }
    }

    /// Add an entity to the batch
    #[allow(clippy::too_many_arguments)]
    pub fn add_entity(
        &mut self,
        transform: &[f32; 16],
        bounds_center_x: f32,
        bounds_center_y: f32,
        bounds_center_z: f32,
        bounds_radius: f32,
        mesh_id: ResourceId,
        index_count: i32,
        shader_id: ResourceId,
        sort_key: u32,
    ) {
        self.entities.push(EntityRenderData {
            entity_id: self.next_entity_id,
            transform: Mat4::from_cols_array(transform),
            bounds_center: Vec3::new(bounds_center_x, bounds_center_y, bounds_center_z),
            bounds_radius,
            mesh_id,
            index_count,
            shader_id,
            sort_key,
        });

        self.next_entity_id += 1;
        self.stats.entities_submitted += 1;
    }

    /// Get current stats
    pub fn get_stats(&self) -> &BatchStats {
        &self.stats
    }
}
