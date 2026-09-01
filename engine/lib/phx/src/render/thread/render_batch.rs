use glam::{Mat4, Vec3};

use crate::math::Matrix;
use crate::render::{BatchStats, CameraRenderData, EntityRenderData, ResourceId};

const ENTITIES_CAPACITY: usize = 1024;

/// Render batch collector - accumulates entities for worker processing
pub struct RenderBatch {
    /// Accumulated entities for this batch
    pub entities: Vec<EntityRenderData>,
    /// Indices into `entities` that survived the last `cull_and_sort`, in
    /// draw order. Sorting this instead of `entities` itself avoids moving
    /// ~100-byte elements around for what's ultimately a 4-byte reorder.
    visible: Vec<u32>,
    /// Current camera data
    pub camera: CameraRenderData,
    /// Statistics
    pub stats: BatchStats,
}

impl RenderBatch {
    /// Begin a new batch - clears accumulated data and set camera for frustum culling
    pub fn new(view: &Matrix, projection: &Matrix, eye: Vec3) -> Self {
        let view_mat = Mat4::from_cols_array(&view.to_cols_array());
        let proj_mat = Mat4::from_cols_array(&projection.to_cols_array());

        Self {
            entities: Vec::with_capacity(ENTITIES_CAPACITY),
            visible: Vec::with_capacity(ENTITIES_CAPACITY),
            camera: CameraRenderData::new(view_mat, proj_mat, eye),
            stats: BatchStats::default(),
        }
    }

    /// Begin a new batch in place, reusing the `entities`/`visible` backing
    /// storage instead of reallocating it (as a fresh `new()` per frame
    /// would, up to 3x/frame for the three blend-mode passes).
    pub fn reset(&mut self, view: &Matrix, projection: &Matrix, eye: Vec3) {
        let view_mat = Mat4::from_cols_array(&view.to_cols_array());
        let proj_mat = Mat4::from_cols_array(&projection.to_cols_array());

        self.entities.clear();
        self.visible.clear();
        self.camera = CameraRenderData::new(view_mat, proj_mat, eye);
        self.stats = BatchStats::default();
    }

    /// Add an entity to the batch
    #[allow(clippy::too_many_arguments)]
    pub fn add_entity(
        &mut self,
        transform: &Matrix,
        bounds_center: Vec3,
        bounds_radius: f32,
        mesh_id: ResourceId,
        index_count: i32,
        shader_id: ResourceId,
        sort_key: u32,
        user_id: u32,
    ) {
        self.entities.push(EntityRenderData {
            transform: Mat4::from_cols_array(&transform.to_cols_array()),
            bounds_center,
            bounds_radius,
            mesh_id,
            index_count,
            shader_id,
            sort_key,
            user_id,
        });

        self.stats.entities_submitted += 1;
    }

    /// Add a cull-only entity: no mesh/shader to draw, just bounds + ordering
    /// data. Used by callers that want frustum culling and sort ordering
    /// without going through the draw-emitting half of the batch (e.g. Lua's
    /// `RenderCoreSystem`, which still issues its own draws so it can apply
    /// per-entity material uniforms the batch draw path doesn't know about).
    pub fn add_cull_entity(
        &mut self,
        bounds_center: Vec3,
        bounds_radius: f32,
        sort_key: u32,
        user_id: u32,
    ) {
        self.entities.push(EntityRenderData {
            transform: Mat4::IDENTITY,
            bounds_center,
            bounds_radius,
            mesh_id: ResourceId(0),
            index_count: 0,
            shader_id: ResourceId(0),
            sort_key,
            user_id,
        });

        self.stats.entities_submitted += 1;
    }

    /// Frustum-cull `entities` and sort the survivors by `sort_key`, storing
    /// the result in `visible` (read back via `visible()`). A negative
    /// radius is a sentinel meaning "never cull" - used for entities with no
    /// bounds source (e.g. no rigid body).
    ///
    /// Sorts indices rather than `entities` itself, and sorts only the
    /// survivors, so this is cheap even though `entities` is not reordered.
    /// `sort_by_key` is stable: entities with equal `sort_key` keep their
    /// relative (insertion) order.
    pub fn cull_and_sort(&mut self) {
        self.stats.total_entities = self.entities.len() as u32;
        self.stats.entities_visible = 0;
        self.stats.entities_culled = 0;

        self.visible.clear();
        for (i, entity) in self.entities.iter().enumerate() {
            if entity.bounds_radius >= 0.0
                && !self
                    .camera
                    .sphere_in_frustum(entity.bounds_center, entity.bounds_radius)
            {
                self.stats.entities_culled += 1;
                continue;
            }
            self.visible.push(i as u32);
        }

        let entities = &self.entities;
        self.visible.sort_by_key(|&i| entities[i as usize].sort_key);

        self.stats.entities_visible = self.visible.len() as u32;
        self.stats.batches_processed += 1;
    }

    /// Indices into `entities`, in draw order, as computed by the last
    /// `cull_and_sort` call.
    pub fn visible(&self) -> &[u32] {
        &self.visible
    }

    /// Get current stats
    pub fn get_stats(&self) -> &BatchStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_view_proj() -> (Matrix, Matrix) {
        // Looking down -Z from the origin; a wide-open frustum so tests can
        // reason about culling purely via bounds_center.z sign.
        let view = Matrix::look_at(&Vec3::ZERO, &Vec3::new(0.0, 0.0, -1.0), &Vec3::Y);
        let proj = Matrix::perspective(90.0, 1.0, 0.1, 1000.0);
        (view, proj)
    }

    fn identity_camera() -> RenderBatch {
        let (view, proj) = test_view_proj();
        RenderBatch::new(&view, &proj, Vec3::ZERO)
    }

    #[test]
    fn cull_and_sort_orders_survivors_by_sort_key_stably() {
        let mut batch = identity_camera();
        // All in front of the camera, interleaved keys: 2, 1, 2, 1.
        let front = Vec3::new(0.0, 0.0, -10.0);
        batch.add_cull_entity(front, 1.0, 2, 100);
        batch.add_cull_entity(front, 1.0, 1, 101);
        batch.add_cull_entity(front, 1.0, 2, 102);
        batch.add_cull_entity(front, 1.0, 1, 103);

        batch.cull_and_sort();

        let order: Vec<u32> = batch
            .visible()
            .iter()
            .map(|&i| batch.entities[i as usize].user_id)
            .collect();
        assert_eq!(order, vec![101, 103, 100, 102]);
    }

    #[test]
    fn cull_and_sort_drops_entities_outside_frustum() {
        let mut batch = identity_camera();
        batch.add_cull_entity(Vec3::new(0.0, 0.0, -10.0), 1.0, 0, 1); // in front: visible
        batch.add_cull_entity(Vec3::new(0.0, 0.0, 10.0), 1.0, 0, 2); // behind: culled

        batch.cull_and_sort();

        assert_eq!(batch.visible().len(), 1);
        assert_eq!(batch.entities[batch.visible()[0] as usize].user_id, 1);
        assert_eq!(batch.stats.entities_culled, 1);
        assert_eq!(batch.stats.entities_visible, 1);
    }

    #[test]
    fn negative_radius_is_never_culled() {
        let mut batch = identity_camera();
        // Far behind the camera, would fail the frustum test, but the
        // negative radius sentinel means "never cull".
        batch.add_cull_entity(Vec3::new(0.0, 0.0, 1_000_000.0), -1.0, 0, 42);

        batch.cull_and_sort();

        assert_eq!(batch.visible().len(), 1);
        assert_eq!(batch.stats.entities_culled, 0);
    }

    #[test]
    fn reset_preserves_capacity() {
        let mut batch = identity_camera();
        for i in 0..10 {
            batch.add_cull_entity(Vec3::new(0.0, 0.0, -10.0), 1.0, 0, i);
        }
        let cap_before = batch.entities.capacity();

        let (view, proj) = test_view_proj();
        batch.reset(&view, &proj, Vec3::new(1.0, 2.0, 3.0));

        assert_eq!(batch.entities.capacity(), cap_before);
        assert_eq!(batch.entities.len(), 0);
        assert_eq!(batch.visible().len(), 0);
        assert_eq!(batch.stats.entities_submitted, 0);
        assert_eq!(batch.camera.position, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn cull_batch_user_ids_are_not_insertion_indices() {
        let mut batch = identity_camera();
        // user_id deliberately differs from insertion index.
        batch.add_cull_entity(Vec3::new(0.0, 0.0, -10.0), 1.0, 0, 999);

        batch.cull_and_sort();

        let idx = batch.visible()[0];
        assert_eq!(batch.entities[idx as usize].user_id, 999);
        assert_ne!(batch.entities[idx as usize].user_id, idx);
    }
}
