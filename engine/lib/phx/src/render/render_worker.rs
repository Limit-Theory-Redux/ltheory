//! Worker pool for parallel render data preparation.
//!
//! Workers perform CPU-only operations like frustum culling, sorting,
//! transform computation, and command buffer encoding. They do NOT
//! make any GL calls - all GPU work happens on the render thread.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crossbeam::channel::{Receiver, Sender, bounded};
use glam::{Mat4, Vec3, Vec4};
use tracing::{debug, error, info};

use super::{RenderCommand, GpuHandle, CmdPrimitiveType};

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

/// Camera data for culling and matrix computation
#[derive(Clone, Debug)]
pub struct CameraRenderData {
    /// View matrix
    pub view: Mat4,
    /// Projection matrix
    pub projection: Mat4,
    /// View-projection matrix (cached)
    pub view_projection: Mat4,
    /// Camera position in world space
    pub position: Vec3,
    /// Frustum planes for culling (6 planes: left, right, bottom, top, near, far)
    pub frustum_planes: [Vec4; 6],
}

impl CameraRenderData {
    /// Create camera data from view and projection matrices
    pub fn new(view: Mat4, projection: Mat4, position: Vec3) -> Self {
        let view_projection = projection * view;
        let frustum_planes = Self::extract_frustum_planes(&view_projection);
        Self {
            view,
            projection,
            view_projection,
            position,
            frustum_planes,
        }
    }

    /// Extract frustum planes from view-projection matrix
    fn extract_frustum_planes(vp: &Mat4) -> [Vec4; 6] {
        let row0 = Vec4::new(vp.x_axis.x, vp.y_axis.x, vp.z_axis.x, vp.w_axis.x);
        let row1 = Vec4::new(vp.x_axis.y, vp.y_axis.y, vp.z_axis.y, vp.w_axis.y);
        let row2 = Vec4::new(vp.x_axis.z, vp.y_axis.z, vp.z_axis.z, vp.w_axis.z);
        let row3 = Vec4::new(vp.x_axis.w, vp.y_axis.w, vp.z_axis.w, vp.w_axis.w);

        [
            (row3 + row0).normalize(), // Left
            (row3 - row0).normalize(), // Right
            (row3 + row1).normalize(), // Bottom
            (row3 - row1).normalize(), // Top
            (row3 + row2).normalize(), // Near
            (row3 - row2).normalize(), // Far
        ]
    }

    /// Test if a bounding sphere is inside the frustum
    pub fn sphere_in_frustum(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.frustum_planes {
            let distance = plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w;
            if distance < -radius {
                return false;
            }
        }
        true
    }
}

/// Task submitted to worker pool
#[derive(Clone)]
pub enum PrepareTask {
    /// Prepare a batch of entities for rendering
    PrepareEntities {
        /// Entities to process
        entities: Vec<EntityRenderData>,
        /// Camera for culling and matrix computation
        camera: CameraRenderData,
        /// Frame ID for synchronization
        frame_id: u64,
    },
    /// Shutdown the worker
    Shutdown,
}

/// Statistics from culling/preparation
#[derive(Clone, Debug, Default)]
pub struct CullStats {
    /// Total entities submitted
    pub total_entities: u32,
    /// Entities that passed frustum culling
    pub visible_entities: u32,
    /// Entities culled
    pub culled_entities: u32,
}

/// Result from worker preparation
#[derive(Clone)]
pub struct PrepareResult {
    /// Frame ID this result is for
    pub frame_id: u64,
    /// Render commands ready to submit
    pub commands: Vec<RenderCommand>,
    /// Culling statistics
    pub stats: CullStats,
}

/// Configuration for worker pool
#[derive(Clone, Debug)]
pub struct WorkerPoolConfig {
    /// Number of worker threads (0 = auto-detect)
    pub num_workers: usize,
    /// Task queue capacity
    pub task_queue_size: usize,
    /// Result queue capacity
    pub result_queue_size: usize,
}

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        Self {
            num_workers: 0, // Auto-detect
            task_queue_size: 64,
            result_queue_size: 64,
        }
    }
}

/// Handle to the worker pool
pub struct WorkerPoolHandle {
    /// Channel to send tasks to workers
    task_tx: Sender<PrepareTask>,
    /// Channel to receive results from workers
    result_rx: Receiver<PrepareResult>,
    /// Worker thread handles
    workers: Vec<JoinHandle<()>>,
    /// Running flag
    running: Arc<AtomicBool>,
    /// Next frame ID
    next_frame_id: AtomicU64,
    /// Number of workers
    num_workers: usize,
}

impl WorkerPoolHandle {
    /// Submit entities for parallel preparation
    pub fn submit_entities(&self, entities: Vec<EntityRenderData>, camera: CameraRenderData) -> u64 {
        let frame_id = self.next_frame_id.fetch_add(1, Ordering::Relaxed);

        if let Err(e) = self.task_tx.send(PrepareTask::PrepareEntities {
            entities,
            camera,
            frame_id,
        }) {
            error!("Failed to submit task to worker pool: {}", e);
        }

        frame_id
    }

    /// Try to receive a result (non-blocking)
    pub fn try_recv_result(&self) -> Option<PrepareResult> {
        self.result_rx.try_recv().ok()
    }

    /// Receive a result (blocking)
    pub fn recv_result(&self) -> Option<PrepareResult> {
        self.result_rx.recv().ok()
    }

    /// Get number of workers
    pub fn num_workers(&self) -> usize {
        self.num_workers
    }

    /// Shutdown the worker pool
    pub fn shutdown(self) {
        self.running.store(false, Ordering::Release);

        // Send shutdown signal to all workers
        for _ in 0..self.workers.len() {
            let _ = self.task_tx.send(PrepareTask::Shutdown);
        }

        // Wait for workers to finish
        for worker in self.workers {
            let _ = worker.join();
        }

        info!("Worker pool shut down");
    }
}

/// Worker thread state
struct Worker {
    /// Worker ID
    id: usize,
    /// Task receiver
    task_rx: Receiver<PrepareTask>,
    /// Result sender
    result_tx: Sender<PrepareResult>,
    /// Running flag
    running: Arc<AtomicBool>,
}

impl Worker {
    fn run(self) {
        debug!("Worker {} started", self.id);

        while self.running.load(Ordering::Acquire) {
            match self.task_rx.recv() {
                Ok(PrepareTask::PrepareEntities { entities, camera, frame_id }) => {
                    let result = self.prepare_entities(entities, camera, frame_id);
                    if let Err(e) = self.result_tx.send(result) {
                        error!("Worker {} failed to send result: {}", self.id, e);
                    }
                }
                Ok(PrepareTask::Shutdown) => {
                    debug!("Worker {} received shutdown", self.id);
                    break;
                }
                Err(_) => {
                    // Channel closed
                    break;
                }
            }
        }

        debug!("Worker {} stopped", self.id);
    }

    fn prepare_entities(
        &self,
        entities: Vec<EntityRenderData>,
        camera: CameraRenderData,
        frame_id: u64,
    ) -> PrepareResult {
        let mut commands = Vec::with_capacity(entities.len() * 3);
        let mut stats = CullStats {
            total_entities: entities.len() as u32,
            visible_entities: 0,
            culled_entities: 0,
        };

        // Sort entities by sort key for better batching
        let mut sorted_entities = entities;
        sorted_entities.sort_by_key(|e| e.sort_key);

        let mut current_shader: Option<u32> = None;

        for entity in sorted_entities {
            // Frustum culling
            if !camera.sphere_in_frustum(entity.bounds_center, entity.bounds_radius) {
                stats.culled_entities += 1;
                continue;
            }

            stats.visible_entities += 1;

            // Compute MVP matrix
            let mvp = camera.view_projection * entity.transform;
            let mvp_array = mvp.to_cols_array();

            // Bind shader if changed
            if current_shader != Some(entity.shader_handle) {
                commands.push(RenderCommand::BindShader {
                    handle: GpuHandle(entity.shader_handle),
                });
                current_shader = Some(entity.shader_handle);
            }

            // Set MVP uniform
            commands.push(RenderCommand::SetUniformMat4 {
                location: entity.mvp_location,
                value: mvp_array,
            });

            // Set model matrix uniform if needed
            if entity.model_location >= 0 {
                commands.push(RenderCommand::SetUniformMat4 {
                    location: entity.model_location,
                    value: entity.transform.to_cols_array(),
                });
            }

            // Draw call
            commands.push(RenderCommand::DrawMesh {
                vao: GpuHandle(entity.mesh_vao),
                index_count: entity.index_count,
                primitive: CmdPrimitiveType::Triangles,
            });
        }

        PrepareResult {
            frame_id,
            commands,
            stats,
        }
    }
}

/// Spawn the worker pool
pub fn spawn_worker_pool(config: WorkerPoolConfig) -> WorkerPoolHandle {
    let num_workers = if config.num_workers == 0 {
        // Auto-detect: CPU cores - 2 (reserve for main + render thread)
        let cores = num_cpus::get();
        (cores.saturating_sub(2)).max(1)
    } else {
        config.num_workers
    };

    info!("Spawning worker pool with {} workers", num_workers);

    let (task_tx, task_rx) = bounded(config.task_queue_size);
    let (result_tx, result_rx) = bounded(config.result_queue_size);
    let running = Arc::new(AtomicBool::new(true));

    let mut workers = Vec::with_capacity(num_workers);

    for id in 0..num_workers {
        let worker = Worker {
            id,
            task_rx: task_rx.clone(),
            result_tx: result_tx.clone(),
            running: Arc::clone(&running),
        };

        let handle = thread::Builder::new()
            .name(format!("RenderWorker-{}", id))
            .spawn(move || worker.run())
            .expect("Failed to spawn worker thread");

        workers.push(handle);
    }

    WorkerPoolHandle {
        task_tx,
        result_rx,
        workers,
        running,
        next_frame_id: AtomicU64::new(0),
        num_workers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_frustum_extraction() {
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh_gl(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 100.0);
        let camera = CameraRenderData::new(view, proj, Vec3::new(0.0, 0.0, 5.0));

        // Point at origin should be visible
        assert!(camera.sphere_in_frustum(Vec3::ZERO, 1.0));

        // Point far behind camera should not be visible
        assert!(!camera.sphere_in_frustum(Vec3::new(0.0, 0.0, 200.0), 1.0));
    }

    #[test]
    fn test_worker_pool_spawn_shutdown() {
        let config = WorkerPoolConfig {
            num_workers: 2,
            task_queue_size: 16,
            result_queue_size: 16,
        };

        let pool = spawn_worker_pool(config);
        assert_eq!(pool.num_workers(), 2);
        pool.shutdown();
    }

    #[test]
    fn test_worker_pool_process_entities() {
        let config = WorkerPoolConfig {
            num_workers: 1,
            task_queue_size: 16,
            result_queue_size: 16,
        };

        let pool = spawn_worker_pool(config);

        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh_gl(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 100.0);
        let camera = CameraRenderData::new(view, proj, Vec3::new(0.0, 0.0, 5.0));

        let entities = vec![
            EntityRenderData {
                entity_id: 1,
                transform: Mat4::IDENTITY,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                mesh_vao: 1,
                index_count: 36,
                shader_handle: 1,
                mvp_location: 0,
                model_location: 1,
                sort_key: 0,
            },
            EntityRenderData {
                entity_id: 2,
                transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 200.0)), // Behind camera
                bounds_center: Vec3::new(0.0, 0.0, 200.0),
                bounds_radius: 1.0,
                mesh_vao: 2,
                index_count: 36,
                shader_handle: 1,
                mvp_location: 0,
                model_location: 1,
                sort_key: 1,
            },
        ];

        let frame_id = pool.submit_entities(entities, camera);
        let result = pool.recv_result().expect("Should receive result");

        assert_eq!(result.frame_id, frame_id);
        assert_eq!(result.stats.total_entities, 2);
        assert_eq!(result.stats.visible_entities, 1); // Only one visible
        assert_eq!(result.stats.culled_entities, 1);  // One culled

        pool.shutdown();
    }
}
