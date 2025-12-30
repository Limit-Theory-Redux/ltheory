//! Batch rendering system for parallel entity processing.
//!
//! This module provides the bridge between Lua entity iteration and
//! the worker pool for parallel frustum culling and command generation.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use glam::{Mat4, Vec3};

use super::{
    CameraRenderData, EntityRenderData, WorkerPoolHandle, PrepareResult,
    RenderCommand, GpuHandle, CmdPrimitiveType, submit_command, is_command_mode,
};

// Cached uniform names for batch rendering - avoids allocation per entity
thread_local! {
    static UNIFORM_MVP: Arc<str> = Arc::from("mModelViewProj");
    static UNIFORM_MODEL: Arc<str> = Arc::from("mModel");
}

/// Statistics from batch rendering
#[derive(Debug, Clone, Default)]
pub struct BatchStats {
    /// Total entities submitted this frame
    pub entities_submitted: u32,
    /// Entities visible after culling
    pub entities_visible: u32,
    /// Entities culled
    pub entities_culled: u32,
    /// Commands generated
    pub commands_generated: u32,
    /// Batches processed
    pub batches_processed: u32,
}

/// Render batch collector - accumulates entities for worker processing
pub struct RenderBatch {
    /// Accumulated entities for this batch
    entities: Vec<EntityRenderData>,
    /// Current camera data
    camera: Option<CameraRenderData>,
    /// Statistics
    stats: BatchStats,
    /// Entity ID counter
    next_entity_id: AtomicU64,
}

impl RenderBatch {
    pub fn new() -> Self {
        Self {
            entities: Vec::with_capacity(1024),
            camera: None,
            stats: BatchStats::default(),
            next_entity_id: AtomicU64::new(1),
        }
    }

    /// Begin a new batch - clears accumulated data
    pub fn begin(&mut self) {
        self.entities.clear();
        self.stats = BatchStats::default();
    }

    /// Set camera for frustum culling
    pub fn set_camera(
        &mut self,
        view: &[f32; 16],
        projection: &[f32; 16],
        eye_x: f32, eye_y: f32, eye_z: f32,
    ) {
        let view_mat = Mat4::from_cols_array(view);
        let proj_mat = Mat4::from_cols_array(projection);
        let position = Vec3::new(eye_x, eye_y, eye_z);

        self.camera = Some(CameraRenderData::new(view_mat, proj_mat, position));
    }

    /// Add an entity to the batch
    pub fn add_entity(
        &mut self,
        transform: &[f32; 16],
        bounds_center_x: f32, bounds_center_y: f32, bounds_center_z: f32,
        bounds_radius: f32,
        mesh_vao: u32,
        index_count: i32,
        shader_handle: u32,
        sort_key: u32,
    ) {
        let entity_id = self.next_entity_id.fetch_add(1, Ordering::Relaxed);

        self.entities.push(EntityRenderData {
            entity_id,
            transform: Mat4::from_cols_array(transform),
            bounds_center: Vec3::new(bounds_center_x, bounds_center_y, bounds_center_z),
            bounds_radius,
            mesh_vao,
            index_count,
            shader_handle,
            mvp_location: -1, // Will use name-based uniforms
            model_location: -1,
            sort_key,
        });

        self.stats.entities_submitted += 1;
    }

    /// Process the batch using workers and submit commands
    pub fn flush(&mut self, worker_pool: Option<&WorkerPoolHandle>) -> BatchStats {
        if self.entities.is_empty() {
            return self.stats.clone();
        }

        let Some(camera) = self.camera.take() else {
            tracing::warn!("RenderBatch::flush called without camera set");
            return self.stats.clone();
        };

        // If we have workers, use parallel processing
        if let Some(pool) = worker_pool {
            let entities = std::mem::take(&mut self.entities);
            let _frame_id = pool.submit_entities(entities, camera.clone());

            // Collect results (blocking for now - could be async later)
            if let Some(result) = pool.recv_result() {
                self.apply_result(result);
            }
        } else {
            // No workers - process serially on main thread
            self.process_serial(&camera);
        }

        self.stats.batches_processed += 1;
        self.stats.clone()
    }

    /// Apply worker results - submit commands to render thread
    fn apply_result(&mut self, result: PrepareResult) {
        self.stats.entities_visible = result.stats.visible_entities;
        self.stats.entities_culled = result.stats.culled_entities;
        self.stats.commands_generated = result.commands.len() as u32;

        // Submit all commands to render thread
        if is_command_mode() {
            for cmd in result.commands {
                submit_command(cmd);
            }
        }
    }

    /// Serial fallback when no workers available
    fn process_serial(&mut self, camera: &CameraRenderData) {
        let entities = std::mem::take(&mut self.entities);
        let mut visible = 0u32;
        let mut culled = 0u32;
        let mut commands = 0u32;

        // Sort by sort_key
        let mut sorted = entities;
        sorted.sort_by_key(|e| e.sort_key);

        let mut current_shader: Option<u32> = None;

        for entity in sorted {
            // Frustum culling
            if !camera.sphere_in_frustum(entity.bounds_center, entity.bounds_radius) {
                culled += 1;
                continue;
            }
            visible += 1;

            // Compute MVP
            let mvp = camera.view_projection * entity.transform;

            if is_command_mode() {
                // Bind shader if changed
                if current_shader != Some(entity.shader_handle) {
                    submit_command(RenderCommand::BindShader {
                        handle: GpuHandle(entity.shader_handle),
                    });
                    current_shader = Some(entity.shader_handle);
                    commands += 1;
                }

                // Set MVP uniform by name (Arc::clone is O(1))
                submit_command(RenderCommand::SetUniformMat4ByName {
                    name: UNIFORM_MVP.with(|n| n.clone()),
                    value: mvp.to_cols_array(),
                });
                commands += 1;

                // Set model matrix
                submit_command(RenderCommand::SetUniformMat4ByName {
                    name: UNIFORM_MODEL.with(|n| n.clone()),
                    value: entity.transform.to_cols_array(),
                });
                commands += 1;

                // Draw
                submit_command(RenderCommand::DrawMesh {
                    vao: GpuHandle(entity.mesh_vao),
                    index_count: entity.index_count,
                    primitive: CmdPrimitiveType::Triangles,
                });
                commands += 1;
            }
        }

        self.stats.entities_visible = visible;
        self.stats.entities_culled = culled;
        self.stats.commands_generated = commands;
    }

    /// Get current stats
    pub fn get_stats(&self) -> &BatchStats {
        &self.stats
    }
}

impl Default for RenderBatch {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// FFI-exposed RenderBatch API
// =============================================================================

use std::cell::RefCell;

thread_local! {
    static RENDER_BATCH: RefCell<RenderBatch> = RefCell::new(RenderBatch::new());
}

/// FFI wrapper for render batch operations
pub struct RenderBatchApi;

#[luajit_ffi_gen::luajit_ffi]
impl RenderBatchApi {
    /// Begin a new batch
    #[bind(name = "Begin")]
    pub fn begin() {
        RENDER_BATCH.with(|batch| batch.borrow_mut().begin());
    }

    /// Set the camera for frustum culling
    #[bind(name = "SetCamera")]
    pub fn set_camera(
        view_00: f32, view_01: f32, view_02: f32, view_03: f32,
        view_10: f32, view_11: f32, view_12: f32, view_13: f32,
        view_20: f32, view_21: f32, view_22: f32, view_23: f32,
        view_30: f32, view_31: f32, view_32: f32, view_33: f32,
        proj_00: f32, proj_01: f32, proj_02: f32, proj_03: f32,
        proj_10: f32, proj_11: f32, proj_12: f32, proj_13: f32,
        proj_20: f32, proj_21: f32, proj_22: f32, proj_23: f32,
        proj_30: f32, proj_31: f32, proj_32: f32, proj_33: f32,
        eye_x: f32, eye_y: f32, eye_z: f32,
    ) {
        let view = [
            view_00, view_01, view_02, view_03,
            view_10, view_11, view_12, view_13,
            view_20, view_21, view_22, view_23,
            view_30, view_31, view_32, view_33,
        ];
        let proj = [
            proj_00, proj_01, proj_02, proj_03,
            proj_10, proj_11, proj_12, proj_13,
            proj_20, proj_21, proj_22, proj_23,
            proj_30, proj_31, proj_32, proj_33,
        ];
        RENDER_BATCH.with(|batch| {
            batch.borrow_mut().set_camera(&view, &proj, eye_x, eye_y, eye_z);
        });
    }

    /// Add an entity to the batch
    #[bind(name = "AddEntity")]
    pub fn add_entity(
        // Transform matrix (column-major)
        t00: f32, t01: f32, t02: f32, t03: f32,
        t10: f32, t11: f32, t12: f32, t13: f32,
        t20: f32, t21: f32, t22: f32, t23: f32,
        t30: f32, t31: f32, t32: f32, t33: f32,
        // Bounding sphere
        bounds_x: f32, bounds_y: f32, bounds_z: f32, bounds_radius: f32,
        // Mesh data
        mesh_vao: u32, index_count: i32,
        // Shader
        shader_handle: u32,
        // Sort key (0 = first)
        sort_key: u32,
    ) {
        let transform = [
            t00, t01, t02, t03,
            t10, t11, t12, t13,
            t20, t21, t22, t23,
            t30, t31, t32, t33,
        ];
        RENDER_BATCH.with(|batch| {
            batch.borrow_mut().add_entity(
                &transform,
                bounds_x, bounds_y, bounds_z, bounds_radius,
                mesh_vao, index_count, shader_handle, sort_key,
            );
        });
    }

    /// Flush the batch (serial processing, no workers)
    #[bind(name = "Flush")]
    pub fn flush() {
        RENDER_BATCH.with(|batch| {
            batch.borrow_mut().flush(None);
        });
    }

    /// Get number of entities in current batch
    #[bind(name = "GetEntityCount")]
    pub fn get_entity_count() -> u32 {
        RENDER_BATCH.with(|batch| batch.borrow().entities.len() as u32)
    }

    /// Get stats from last flush
    #[bind(name = "GetEntitiesSubmitted")]
    pub fn get_entities_submitted() -> u32 {
        RENDER_BATCH.with(|batch| batch.borrow().stats.entities_submitted)
    }

    #[bind(name = "GetEntitiesVisible")]
    pub fn get_entities_visible() -> u32 {
        RENDER_BATCH.with(|batch| batch.borrow().stats.entities_visible)
    }

    #[bind(name = "GetEntitiesCulled")]
    pub fn get_entities_culled() -> u32 {
        RENDER_BATCH.with(|batch| batch.borrow().stats.entities_culled)
    }

    #[bind(name = "GetCommandsGenerated")]
    pub fn get_commands_generated() -> u32 {
        RENDER_BATCH.with(|batch| batch.borrow().stats.commands_generated)
    }
}

/// Flush the render batch with a worker pool (called from Engine).
/// Returns batch stats for this flush.
pub fn flush_render_batch_with_workers(worker_pool: Option<&WorkerPoolHandle>) -> BatchStats {
    RENDER_BATCH.with(|batch| batch.borrow_mut().flush(worker_pool))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_batch_new() {
        let batch = RenderBatch::new();
        assert!(batch.entities.is_empty());
        assert!(batch.camera.is_none());
    }

    #[test]
    fn test_render_batch_add_entity() {
        let mut batch = RenderBatch::new();
        batch.begin();

        let transform = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];

        batch.add_entity(&transform, 0.0, 0.0, 0.0, 1.0, 1, 36, 1, 0);
        assert_eq!(batch.entities.len(), 1);
        assert_eq!(batch.stats.entities_submitted, 1);
    }
}
