//! Render queue with FFI bindings for Lua.
//!
//! This module provides the API that Lua uses to submit render commands.
//! Since Rust enums with data variants cannot be directly exposed via LuaJIT FFI,
//! we expose methods that internally create and queue the commands.

use std::cell::RefCell;
use std::sync::{OnceLock, RwLock};

use super::{
    BlendMode, CullFace, GpuHandle, ImmVertex, InstanceData, CmdPrimitiveType, Mesh, RenderCommand,
    RenderThreadHandle, ResourceId, next_resource_id as global_next_resource_id,
};
use crate::math::Matrix;

/// Global render queue instance with interior mutability for handle storage
static RENDER_QUEUE: OnceLock<RwLock<RenderQueue>> = OnceLock::new();

/// Thread-local command buffer for batching
thread_local! {
    static COMMAND_BUFFER: RefCell<Vec<RenderCommand>> = RefCell::new(Vec::with_capacity(1024));
}

/// Render queue that batches and submits commands to the render thread.
///
/// The queue supports two modes of operation:
/// 1. Immediate submission: Commands are sent directly to the render thread
/// 2. Batched submission: Commands are collected and sent together for efficiency
pub struct RenderQueue {
    /// Handle to the render thread
    render_handle: Option<RenderThreadHandle>,
}

impl RenderQueue {
    /// Create a new render queue
    pub fn new() -> Self {
        Self {
            render_handle: None,
        }
    }

    /// Set the render thread handle
    pub fn set_render_handle(&mut self, handle: RenderThreadHandle) {
        self.render_handle = Some(handle);
    }

    /// Get the global render queue instance (read access)
    ///
    /// Returns a guard that provides immutable access to the RenderQueue.
    /// For most operations, prefer using the FFI methods directly.
    pub fn global() -> std::sync::RwLockReadGuard<'static, RenderQueue> {
        RENDER_QUEUE
            .get_or_init(|| RwLock::new(RenderQueue::new()))
            .read()
            .expect("RenderQueue lock poisoned")
    }

    /// Initialize the global render queue with a render handle
    ///
    /// This must be called after the render thread is spawned to enable
    /// command submission. Without this, commands are buffered locally.
    pub fn init_global(handle: RenderThreadHandle) {
        let queue = RENDER_QUEUE.get_or_init(|| RwLock::new(RenderQueue::new()));
        let mut guard = queue.write().expect("RenderQueue lock poisoned");
        guard.set_render_handle(handle);
    }

    /// Clear the global render handle (called during shutdown)
    pub fn clear_global_handle() {
        if let Some(queue) = RENDER_QUEUE.get() {
            if let Ok(mut guard) = queue.write() {
                guard.render_handle = None;
            }
        }
    }

    /// Generate a new unique resource ID
    /// Uses the global counter from render_mode to avoid ID conflicts
    pub fn next_resource_id(&self) -> ResourceId {
        global_next_resource_id()
    }

    /// Submit a command immediately
    fn submit(&self, cmd: RenderCommand) {
        if let Some(ref handle) = self.render_handle {
            handle.submit(cmd);
        } else {
            // Queue in thread-local buffer if no handle
            COMMAND_BUFFER.with(|buf| {
                buf.borrow_mut().push(cmd);
            });
        }
    }

    /// Begin a new frame (clears the command buffer)
    fn begin_frame(&self) {
        COMMAND_BUFFER.with(|buf| {
            buf.borrow_mut().clear();
        });
    }

    /// Flush all queued commands to the render thread
    fn flush(&self) {
        if let Some(ref handle) = self.render_handle {
            COMMAND_BUFFER.with(|buf| {
                let commands: Vec<_> = buf.borrow_mut().drain(..).collect();
                handle.submit_batch(commands);
            });
        }
    }

    /// Synchronize with the render thread (wait for all commands to complete)
    fn sync(&self) -> bool {
        if let Some(ref handle) = self.render_handle {
            handle.sync()
        } else {
            false
        }
    }
}

impl Default for RenderQueue {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// FFI-exposed RenderQueue API
// =============================================================================

#[luajit_ffi_gen::luajit_ffi]
impl RenderQueue {
    // === Frame Management ===

    /// Begin a new frame
    #[bind(name = "BeginFrame")]
    pub fn ffi_begin_frame(&self) {
        self.begin_frame();
    }

    /// Flush all queued commands
    #[bind(name = "Flush")]
    pub fn ffi_flush(&self) {
        self.flush();
    }

    /// Wait for render thread to complete all commands
    #[bind(name = "Sync")]
    pub fn ffi_sync(&self) -> bool {
        self.sync()
    }

    // === State Management ===

    /// Set the viewport
    #[bind(name = "SetViewport")]
    pub fn ffi_set_viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        self.submit(RenderCommand::SetViewport { x, y, width, height });
    }

    /// Set the scissor region
    #[bind(name = "SetScissor")]
    pub fn ffi_set_scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        self.submit(RenderCommand::SetScissor { x, y, width, height });
    }

    /// Enable or disable scissor test
    #[bind(name = "EnableScissor")]
    pub fn ffi_enable_scissor(&self, enable: bool) {
        self.submit(RenderCommand::EnableScissor(enable));
    }

    /// Set blend mode (0=Disabled, 1=Alpha, 2=Additive, 3=PreMultAlpha)
    #[bind(name = "SetBlendMode")]
    pub fn ffi_set_blend_mode(&self, mode: i32) {
        let blend_mode = match mode {
            0 => BlendMode::Disabled,
            1 => BlendMode::Alpha,
            2 => BlendMode::Additive,
            3 => BlendMode::PreMultAlpha,
            _ => BlendMode::Alpha,
        };
        self.submit(RenderCommand::SetBlendMode(blend_mode));
    }

    /// Set cull face (0=None, 1=Back, 2=Front)
    #[bind(name = "SetCullFace")]
    pub fn ffi_set_cull_face(&self, face: i32) {
        let cull_face = match face {
            0 => CullFace::None,
            1 => CullFace::Back,
            2 => CullFace::Front,
            _ => CullFace::None,
        };
        self.submit(RenderCommand::SetCullFace(cull_face));
    }

    /// Enable or disable depth testing
    #[bind(name = "SetDepthTest")]
    pub fn ffi_set_depth_test(&self, enable: bool) {
        self.submit(RenderCommand::SetDepthTest(enable));
    }

    /// Enable or disable depth writing
    #[bind(name = "SetDepthWritable")]
    pub fn ffi_set_depth_writable(&self, enable: bool) {
        self.submit(RenderCommand::SetDepthWritable(enable));
    }

    /// Set wireframe mode
    #[bind(name = "SetWireframe")]
    pub fn ffi_set_wireframe(&self, enable: bool) {
        self.submit(RenderCommand::SetWireframe(enable));
    }

    // === Shader Operations ===

    /// Bind a shader program
    #[bind(name = "BindShader")]
    pub fn ffi_bind_shader(&self, handle: u32) {
        self.submit(RenderCommand::BindShader { handle: GpuHandle(handle) });
    }

    /// Unbind the current shader
    #[bind(name = "UnbindShader")]
    pub fn ffi_unbind_shader(&self) {
        self.submit(RenderCommand::UnbindShader);
    }

    /// Set an integer uniform
    #[bind(name = "SetUniformInt")]
    pub fn ffi_set_uniform_int(&self, location: i32, value: i32) {
        self.submit(RenderCommand::SetUniformInt { location, value });
    }

    /// Set a float uniform
    #[bind(name = "SetUniformFloat")]
    pub fn ffi_set_uniform_float(&self, location: i32, value: f32) {
        self.submit(RenderCommand::SetUniformFloat { location, value });
    }

    /// Set a vec2 uniform
    #[bind(name = "SetUniformFloat2")]
    pub fn ffi_set_uniform_float2(&self, location: i32, x: f32, y: f32) {
        self.submit(RenderCommand::SetUniformFloat2 { location, value: [x, y] });
    }

    /// Set a vec3 uniform
    #[bind(name = "SetUniformFloat3")]
    pub fn ffi_set_uniform_float3(&self, location: i32, x: f32, y: f32, z: f32) {
        self.submit(RenderCommand::SetUniformFloat3 { location, value: [x, y, z] });
    }

    /// Set a vec4 uniform
    #[bind(name = "SetUniformFloat4")]
    pub fn ffi_set_uniform_float4(&self, location: i32, x: f32, y: f32, z: f32, w: f32) {
        self.submit(RenderCommand::SetUniformFloat4 { location, value: [x, y, z, w] });
    }

    // === Texture Operations ===

    /// Bind a 2D texture to a slot
    #[bind(name = "BindTexture2D")]
    pub fn ffi_bind_texture_2d(&self, slot: u32, handle: u32) {
        self.submit(RenderCommand::BindTexture2D { slot, handle: GpuHandle(handle) });
    }

    /// Bind a 3D texture to a slot
    #[bind(name = "BindTexture3D")]
    pub fn ffi_bind_texture_3d(&self, slot: u32, handle: u32) {
        self.submit(RenderCommand::BindTexture3D { slot, handle: GpuHandle(handle) });
    }

    /// Bind a cube texture to a slot
    #[bind(name = "BindTextureCube")]
    pub fn ffi_bind_texture_cube(&self, slot: u32, handle: u32) {
        self.submit(RenderCommand::BindTextureCube { slot, handle: GpuHandle(handle) });
    }

    /// Unbind a texture from a slot
    #[bind(name = "UnbindTexture")]
    pub fn ffi_unbind_texture(&self, slot: u32) {
        self.submit(RenderCommand::UnbindTexture { slot });
    }

    // === Framebuffer Operations ===

    /// Bind a framebuffer
    #[bind(name = "BindFramebuffer")]
    pub fn ffi_bind_framebuffer(&self, handle: u32) {
        self.submit(RenderCommand::BindFramebuffer { handle: GpuHandle(handle) });
    }

    /// Bind the default framebuffer
    #[bind(name = "BindDefaultFramebuffer")]
    pub fn ffi_bind_default_framebuffer(&self) {
        self.submit(RenderCommand::BindDefaultFramebuffer);
    }

    /// Clear color buffer
    #[bind(name = "ClearColor")]
    pub fn ffi_clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        self.submit(RenderCommand::Clear {
            color: Some([r, g, b, a]),
            depth: None,
        });
    }

    /// Clear depth buffer
    #[bind(name = "ClearDepth")]
    pub fn ffi_clear_depth(&self, depth: f32) {
        self.submit(RenderCommand::Clear {
            color: None,
            depth: Some(depth),
        });
    }

    /// Clear both color and depth buffers
    #[bind(name = "Clear")]
    pub fn ffi_clear(&self, r: f32, g: f32, b: f32, a: f32, depth: f32) {
        self.submit(RenderCommand::Clear {
            color: Some([r, g, b, a]),
            depth: Some(depth),
        });
    }

    // === Drawing Operations ===

    /// Draw a mesh
    #[bind(name = "DrawMesh")]
    pub fn ffi_draw_mesh(&self, vao: u32, index_count: i32) {
        self.submit(RenderCommand::DrawMesh {
            vao: GpuHandle(vao),
            index_count,
            primitive: CmdPrimitiveType::Triangles,
        });
    }

    /// Draw a mesh with a specific primitive type
    #[bind(name = "DrawMeshPrimitive")]
    pub fn ffi_draw_mesh_primitive(&self, vao: u32, index_count: i32, primitive: i32) {
        let prim = match primitive {
            0 => CmdPrimitiveType::Points,
            1 => CmdPrimitiveType::Lines,
            2 => CmdPrimitiveType::LineStrip,
            3 => CmdPrimitiveType::Triangles,
            4 => CmdPrimitiveType::TriangleStrip,
            5 => CmdPrimitiveType::TriangleFan,
            6 => CmdPrimitiveType::Quads,
            _ => CmdPrimitiveType::Triangles,
        };
        self.submit(RenderCommand::DrawMesh {
            vao: GpuHandle(vao),
            index_count,
            primitive: prim,
        });
    }

    /// Draw instanced mesh
    #[bind(name = "DrawMeshInstanced")]
    pub fn ffi_draw_mesh_instanced(&self, vao: u32, index_count: i32, instance_count: i32) {
        self.submit(RenderCommand::DrawMeshInstanced {
            vao: GpuHandle(vao),
            index_count,
            instance_count,
            primitive: CmdPrimitiveType::Triangles,
        });
    }

    // === Window Operations ===

    /// Signal resize
    #[bind(name = "Resize")]
    pub fn ffi_resize(&self, width: u32, height: u32) {
        self.submit(RenderCommand::Resize { width, height });
    }

    /// Signal swap buffers (frame end)
    #[bind(name = "SwapBuffers")]
    pub fn ffi_swap_buffers(&self) {
        self.submit(RenderCommand::SwapBuffers);
    }

    // === Camera UBO ===

    /// Create the camera UBO on the render thread
    #[bind(name = "CreateCameraUBO")]
    pub fn ffi_create_camera_ubo(&self) {
        self.submit(RenderCommand::CreateCameraUBO);
    }

    /// Update the camera UBO with new camera data
    /// Parameters are the matrices and vectors that make up the camera state.
    #[bind(name = "UpdateCameraUBO")]
    pub fn ffi_update_camera_ubo(
        &self,
        m_view: &Matrix,
        m_proj: &Matrix,
        eye_x: f32, eye_y: f32, eye_z: f32,
        star_dir_x: f32, star_dir_y: f32, star_dir_z: f32,
    ) {
        use super::CameraUboData;
        use glam::Mat4;

        let mut data = CameraUboData::new();
        // Convert Matrix to Mat4 via column array
        let view = Mat4::from_cols_array(&m_view.to_cols_array());
        let proj = Mat4::from_cols_array(&m_proj.to_cols_array());
        data.set_view(&view);
        data.set_proj(&proj);
        data.set_eye(glam::vec3(eye_x, eye_y, eye_z));
        data.set_star_dir(glam::vec3(star_dir_x, star_dir_y, star_dir_z));

        // Convert to boxed array for command
        let bytes = data.as_bytes();
        let mut boxed: Box<[u8; 288]> = Box::new([0u8; 288]);
        boxed.copy_from_slice(bytes);

        self.submit(RenderCommand::UpdateCameraUBO { data: boxed });
    }

    /// Create the material UBO on the render thread
    #[bind(name = "CreateMaterialUBO")]
    pub fn ffi_create_material_ubo(&self) {
        self.submit(RenderCommand::CreateMaterialUBO);
    }

    /// Update the material UBO with new material properties
    #[bind(name = "UpdateMaterialUBO")]
    pub fn ffi_update_material_ubo(
        &self,
        r: f32, g: f32, b: f32, a: f32,
        metallic: f32, roughness: f32, emission: f32,
    ) {
        use super::MaterialUboData;

        let mut data = MaterialUboData::new();
        data.set_color(r, g, b, a);
        data.set_metallic(metallic);
        data.set_roughness(roughness);
        data.set_emission(emission);

        // Convert to boxed array for command
        let boxed: Box<[u8; 32]> = Box::new(*data.as_bytes());

        self.submit(RenderCommand::UpdateMaterialUBO { data: boxed });
    }

    /// Create the light UBO on the render thread
    #[bind(name = "CreateLightUBO")]
    pub fn ffi_create_light_ubo(&self) {
        self.submit(RenderCommand::CreateLightUBO);
    }

    /// Update the light UBO with light properties
    #[bind(name = "UpdateLightUBO")]
    pub fn ffi_update_light_ubo(
        &self,
        pos_x: f32, pos_y: f32, pos_z: f32, radius: f32,
        r: f32, g: f32, b: f32, intensity: f32,
    ) {
        use super::LightUboData;

        let mut data = LightUboData::new();
        data.set_position(pos_x, pos_y, pos_z);
        data.set_radius(radius);
        data.set_color(r, g, b);
        data.set_intensity(intensity);

        // Convert to boxed array for command
        let boxed: Box<[u8; 32]> = Box::new(*data.as_bytes());

        self.submit(RenderCommand::UpdateLightUBO { data: boxed });
    }
}

// =============================================================================
// Immediate Mode Drawing API
// =============================================================================

/// Immediate mode vertex builder for the render queue
pub struct ImmediateBuilder {
    vertices: Vec<ImmVertex>,
    current_color: [f32; 4],
    current_normal: [f32; 3],
    current_uv: [f32; 2],
    primitive: CmdPrimitiveType,
}

impl ImmediateBuilder {
    /// Create a new immediate mode builder
    pub fn new(primitive: CmdPrimitiveType) -> Self {
        Self {
            vertices: Vec::with_capacity(64),
            current_color: [1.0, 1.0, 1.0, 1.0],
            current_normal: [0.0, 0.0, 0.0],
            current_uv: [0.0, 0.0],
            primitive,
        }
    }

    /// Set the current color for subsequent vertices
    pub fn color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.current_color = [r, g, b, a];
    }

    /// Set the current normal for subsequent vertices
    pub fn normal(&mut self, x: f32, y: f32, z: f32) {
        self.current_normal = [x, y, z];
    }

    /// Set the current UV for subsequent vertices
    pub fn uv(&mut self, u: f32, v: f32) {
        self.current_uv = [u, v];
    }

    /// Add a vertex
    pub fn vertex(&mut self, x: f32, y: f32, z: f32) {
        self.vertices.push(ImmVertex {
            pos: [x, y, z],
            normal: self.current_normal,
            uv: self.current_uv,
            color: self.current_color,
        });
    }

    /// Build the draw command
    pub fn build(self) -> RenderCommand {
        RenderCommand::DrawImmediate {
            primitive: self.primitive,
            vertices: self.vertices,
        }
    }
}

// =============================================================================
// Instanced Drawing API
// =============================================================================

/// Instance batch builder for instanced rendering.
/// Collects instance data (transforms + colors) and submits as a single draw call.
///
/// Usage from Lua:
/// ```lua
/// local batch = InstanceBatch.Create(mesh)
/// for each object do
///     batch:addInstance(transform, r, g, b, a)
/// end
/// batch:draw()
/// ```
pub struct InstanceBatch {
    mesh_resource_id: ResourceId,
    index_count: i32,
    instances: Vec<InstanceData>,
    primitive: CmdPrimitiveType,
}

#[luajit_ffi_gen::luajit_ffi]
impl InstanceBatch {
    /// Create a new instance batch for a mesh.
    /// The mesh must have been created in command mode (has a ResourceId).
    #[bind(name = "Create")]
    pub fn new(mesh: &mut super::Mesh, primitive: i32) -> Option<InstanceBatch> {
        // Ensure mesh has a resource ID (triggers lazy creation if needed)
        let resource_id = mesh.ensure_resource_id()?;
        let index_count = mesh.get_index_count();

        Some(InstanceBatch {
            mesh_resource_id: resource_id,
            index_count,
            instances: Vec::with_capacity(64),
            primitive: match primitive {
                0 => CmdPrimitiveType::Points,
                1 => CmdPrimitiveType::Lines,
                2 => CmdPrimitiveType::LineStrip,
                3 => CmdPrimitiveType::Triangles,
                4 => CmdPrimitiveType::TriangleStrip,
                5 => CmdPrimitiveType::TriangleFan,
                _ => CmdPrimitiveType::Triangles,
            },
        })
    }

    /// Add an instance with a 4x4 transform matrix and RGBA color.
    /// Matrix is in column-major order (OpenGL convention).
    #[bind(name = "AddInstance")]
    pub fn add_instance(
        &mut self,
        // Matrix elements (column-major)
        m00: f32, m01: f32, m02: f32, m03: f32,
        m10: f32, m11: f32, m12: f32, m13: f32,
        m20: f32, m21: f32, m22: f32, m23: f32,
        m30: f32, m31: f32, m32: f32, m33: f32,
        // Color
        r: f32, g: f32, b: f32, a: f32,
    ) {
        self.instances.push(InstanceData {
            model_matrix: [
                m00, m01, m02, m03,
                m10, m11, m12, m13,
                m20, m21, m22, m23,
                m30, m31, m32, m33,
            ],
            color: [r, g, b, a],
        });
    }

    /// Add an instance using a Matrix object and color values.
    #[bind(name = "AddInstanceMatrix")]
    pub fn add_instance_matrix(&mut self, matrix: &crate::math::Matrix, r: f32, g: f32, b: f32, a: f32) {
        self.instances.push(InstanceData {
            model_matrix: matrix.to_cols_array(),
            color: [r, g, b, a],
        });
    }

    /// Get the current number of instances in the batch
    #[bind(name = "GetInstanceCount")]
    pub fn get_instance_count(&self) -> i32 {
        self.instances.len() as i32
    }

    /// Clear all instances (reuse batch for next frame)
    #[bind(name = "Clear")]
    pub fn clear(&mut self) {
        self.instances.clear();
    }

    /// Submit the batch for drawing.
    /// This sends a DrawInstancedWithData command to the render thread.
    #[bind(name = "Draw")]
    pub fn draw(&self) {
        if self.instances.is_empty() {
            return;
        }

        super::submit_command(RenderCommand::DrawInstancedWithData {
            mesh_id: self.mesh_resource_id,
            index_count: self.index_count,
            instances: self.instances.clone(),
            primitive: self.primitive,
        });
    }

    /// Submit and clear in one call (common pattern for per-frame batches)
    #[bind(name = "Flush")]
    pub fn flush(&mut self) {
        self.draw();
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_queue_new() {
        let queue = RenderQueue::new();
        assert!(queue.render_handle.is_none());
    }

    #[test]
    fn test_resource_id_generation() {
        let queue = RenderQueue::new();
        let id1 = queue.next_resource_id();
        let id2 = queue.next_resource_id();
        assert_ne!(id1.0, id2.0);
    }

    #[test]
    fn test_immediate_builder() {
        let mut builder = ImmediateBuilder::new(CmdPrimitiveType::Triangles);
        builder.color(1.0, 0.0, 0.0, 1.0);
        builder.vertex(0.0, 0.0, 0.0);
        builder.vertex(1.0, 0.0, 0.0);
        builder.vertex(0.5, 1.0, 0.0);

        let cmd = builder.build();
        if let RenderCommand::DrawImmediate { vertices, primitive } = cmd {
            assert_eq!(vertices.len(), 3);
            assert!(matches!(primitive, CmdPrimitiveType::Triangles));
        } else {
            panic!("Expected DrawImmediate command");
        }
    }
}
