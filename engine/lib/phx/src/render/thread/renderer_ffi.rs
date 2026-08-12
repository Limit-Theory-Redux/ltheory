use glam::Mat4;
use tracing::error;

use crate::math::Matrix;
use crate::render::{
    BatchStats, BlendMode, CameraUboData, CmdPrimitiveType, CullFace, GpuHandle, LightUboData,
    MaterialUboData, RenderBatch, Renderer, ResourceId,
};

// =============================================================================
// FFI-exposed Renderer API
//
// These are thin Lua-facing wrappers: they convert Lua-friendly primitives
// (ints, separate floats, ...) into the native types the per-command methods
// on `Renderer` (defined in `renderer_immediate.rs`/`renderer_threaded.rs`)
// expect, then call those directly. The per-command methods carry an
// `_intern` suffix so these wrappers can keep the plain, Lua-facing name
// without clashing with them by name.
// =============================================================================

#[luajit_ffi_gen::luajit_ffi]
impl Renderer {
    // === Frame Management ===

    /// Begin a new frame
    pub fn begin_frame(&mut self) {
        self.begin_frame_intern();
    }

    /// Flush all queued commands to the render thread
    pub fn flush(&mut self) {
        self.flush_intern();
    }

    /// Synchronize with the render thread (wait for all commands to complete)
    pub fn sync(&mut self) -> bool {
        self.sync_intern()
    }

    // === Batch rendering ===

    pub fn begin_batch(
        &mut self,
        view: &[f32; 16],
        projection: &[f32; 16],
        eye_x: f32,
        eye_y: f32,
        eye_z: f32,
    ) {
        self.data.active_batch = Some(RenderBatch::new(view, projection, eye_x, eye_y, eye_z));
    }

    /// `mesh_id`/`shader_id` are `ResourceId`s as plain scalars - obtain them
    /// from `Mesh::resource_id`/`Shader::resource_id` (`mesh:resourceId(r)` /
    /// `shader:resourceId()` in Lua).
    #[allow(clippy::too_many_arguments)]
    pub fn add_entity(
        &mut self,
        transform: &[f32; 16],
        bounds_center_x: f32,
        bounds_center_y: f32,
        bounds_center_z: f32,
        bounds_radius: f32,
        mesh_id: u64,
        index_count: i32,
        shader_id: u64,
        sort_key: u32,
    ) {
        if let Some(batch) = &mut self.data.active_batch {
            batch.add_entity(
                transform,
                bounds_center_x,
                bounds_center_y,
                bounds_center_z,
                bounds_radius,
                ResourceId(mesh_id),
                index_count,
                ResourceId(shader_id),
                sort_key,
            );
        } else {
            error!("There is no active batch started. Use begin_batch() to start it.");
        }
    }

    pub fn flush_batch(&mut self) {
        self.process_batch();
    }

    pub fn get_batch_stats(&self) -> Option<&BatchStats> {
        if let Some(batch) = &self.data.active_batch {
            Some(batch.get_stats())
        } else {
            error!("There is no active batch started. Use begin_batch() to start it.");
            None
        }
    }

    // === State Management ===

    /// Set the viewport
    pub fn set_viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.set_viewport_intern(x, y, width, height);
    }

    /// Set the scissor region
    pub fn set_scissor(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.set_scissor_intern(x, y, width, height);
    }

    /// Enable or disable scissor test
    pub fn enable_scissor(&mut self, enable: bool) {
        self.enable_scissor_intern(enable);
    }

    /// Set blend mode (0=Disabled, 1=Alpha, 2=Additive, 3=PreMultAlpha)
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.set_blend_mode_intern(mode);
    }

    /// Set cull face (0=None, 1=Back, 2=Front)
    pub fn set_cull_face(&mut self, face: CullFace) {
        self.set_cull_face_intern(face);
    }

    /// Enable or disable depth testing
    pub fn set_depth_test(&mut self, enable: bool) {
        self.set_depth_test_intern(enable);
    }

    /// Enable or disable depth writing
    pub fn set_depth_writable(&mut self, enable: bool) {
        self.set_depth_writable_intern(enable);
    }

    /// Set wireframe mode
    pub fn set_wireframe(&mut self, enable: bool) {
        self.set_wireframe_intern(enable);
    }

    // === Shader Operations ===

    /// Bind a shader program
    pub fn bind_shader(&mut self, handle: u32) {
        self.bind_shader_intern(GpuHandle(handle));
    }

    /// Unbind the current shader
    pub fn unbind_shader(&mut self) {
        self.unbind_shader_intern();
    }

    /// Set an integer uniform
    pub fn set_uniform_int(&mut self, location: i32, value: i32) {
        self.set_uniform_int_intern(location, value);
    }

    /// Set a float uniform
    pub fn set_uniform_float(&mut self, location: i32, value: f32) {
        self.set_uniform_float_intern(location, value);
    }

    /// Set a vec2 uniform
    pub fn set_uniform_float2(&mut self, location: i32, x: f32, y: f32) {
        self.set_uniform_float2_intern(location, [x, y]);
    }

    /// Set a vec3 uniform
    pub fn set_uniform_float3(&mut self, location: i32, x: f32, y: f32, z: f32) {
        self.set_uniform_float3_intern(location, [x, y, z]);
    }

    /// Set a vec4 uniform
    pub fn set_uniform_float4(&mut self, location: i32, x: f32, y: f32, z: f32, w: f32) {
        self.set_uniform_float4_intern(location, [x, y, z, w]);
    }

    // === Texture Operations ===

    /// Bind a 2D texture to a slot
    pub fn bind_texture_2d(&mut self, slot: u32, handle: u32) {
        self.bind_texture_2d_intern(slot, GpuHandle(handle));
    }

    /// Bind a 3D texture to a slot
    pub fn bind_texture_3d(&mut self, slot: u32, handle: u32) {
        self.bind_texture_3d_intern(slot, GpuHandle(handle));
    }

    /// Bind a cube texture to a slot
    pub fn bind_texture_cube(&mut self, slot: u32, handle: u32) {
        self.bind_texture_cube_intern(slot, GpuHandle(handle));
    }

    /// Unbind a texture from a slot
    pub fn unbind_texture(&mut self, slot: u32) {
        self.unbind_texture_intern(slot);
    }

    // === Framebuffer Operations ===

    /// Bind a framebuffer
    pub fn bind_framebuffer(&mut self, handle: u32) {
        self.bind_framebuffer_intern(GpuHandle(handle));
    }

    /// Bind the default framebuffer
    pub fn bind_default_framebuffer(&mut self) {
        self.bind_default_framebuffer_intern();
    }

    /// Clear color buffer
    pub fn clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.clear_intern(Some([r, g, b, a]), None);
    }

    /// Clear depth buffer
    pub fn clear_depth(&mut self, depth: f32) {
        self.clear_intern(None, Some(depth));
    }

    /// Clear both color and depth buffers
    pub fn clear(&mut self, r: f32, g: f32, b: f32, a: f32, depth: f32) {
        self.clear_intern(Some([r, g, b, a]), Some(depth));
    }

    // === Drawing Operations ===

    /// Draw a mesh
    pub fn draw_mesh(&mut self, vao: u32, index_count: i32) {
        self.draw_mesh_intern(GpuHandle(vao), index_count, CmdPrimitiveType::Triangles);
    }

    /// Draw a mesh with a specific primitive type
    pub fn draw_mesh_primitive(&mut self, vao: u32, index_count: i32, primitive: CmdPrimitiveType) {
        self.draw_mesh_intern(GpuHandle(vao), index_count, primitive);
    }

    /// Draw instanced mesh
    pub fn draw_mesh_instanced(&mut self, vao: u32, index_count: i32, instance_count: i32) {
        self.draw_mesh_instanced_intern(
            GpuHandle(vao),
            index_count,
            instance_count,
            CmdPrimitiveType::Triangles,
        );
    }

    // === Window Operations ===

    /// Signal resize
    pub fn resize(&mut self, width: u32, height: u32) {
        self.resize_intern(width, height);
    }

    /// Signal swap buffers (frame end)
    pub fn swap_buffers(&mut self) {
        self.swap_buffers_intern();
    }

    // === Camera UBO ===

    /// Create the camera UBO on the render thread
    pub fn create_camera_ubo(&mut self) {
        self.create_camera_ubo_intern();
    }

    /// Update the camera UBO with new camera data
    /// Parameters are the matrices and vectors that make up the camera state.
    #[allow(clippy::too_many_arguments)]
    pub fn update_camera_ubo(
        &mut self,
        m_view: &Matrix,
        m_proj: &Matrix,
        eye_x: f32,
        eye_y: f32,
        eye_z: f32,
        star_dir_x: f32,
        star_dir_y: f32,
        star_dir_z: f32,
    ) {
        let mut data = CameraUboData::new();
        // Convert Matrix to Mat4 via column array
        let view = Mat4::from_cols_array(&m_view.to_cols_array());
        let proj = Mat4::from_cols_array(&m_proj.to_cols_array());
        data.set_view(&view);
        // Derived rather than passed in: the two Lua camera paths
        // (CameraManager vs. legacy Camera) disagree on whether mViewInv's
        // translation should be the real world-space position or zero, but
        // both leave its rotation the exact inverse of `view`'s - which is
        // all worldray.glsl (via mat3(mViewInv)) actually needs.
        data.set_view_inv(&view.inverse());
        data.set_proj(&proj);
        data.set_eye(glam::vec3(eye_x, eye_y, eye_z));
        data.set_star_dir(glam::vec3(star_dir_x, star_dir_y, star_dir_z));

        // Convert to boxed array for command
        let bytes = data.as_bytes();
        let mut boxed: Box<[u8; 288]> = Box::new([0u8; 288]);
        boxed.copy_from_slice(bytes);

        self.update_camera_ubo_intern(boxed);
    }

    /// Create the material UBO on the render thread
    pub fn create_material_ubo(&mut self) {
        self.create_material_ubo_intern();
    }

    /// Update the material UBO with new material properties
    pub fn update_material_ubo(
        &mut self,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
        metallic: f32,
        roughness: f32,
        emission: f32,
    ) {
        let mut data = MaterialUboData::new();
        data.set_color(r, g, b, a);
        data.set_metallic(metallic);
        data.set_roughness(roughness);
        data.set_emission(emission);

        self.update_material_ubo_intern(*data.as_bytes());
    }

    /// Create the light UBO on the render thread
    pub fn create_light_ubo(&mut self) {
        self.create_light_ubo_intern();
    }

    /// Update the light UBO with light properties
    #[allow(clippy::too_many_arguments)]
    pub fn update_light_ubo(
        &mut self,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        radius: f32,
        r: f32,
        g: f32,
        b: f32,
        intensity: f32,
    ) {
        let mut data = LightUboData::new();
        data.set_position(pos_x, pos_y, pos_z);
        data.set_radius(radius);
        data.set_color(r, g, b);
        data.set_intensity(intensity);

        self.update_light_ubo_intern(*data.as_bytes());
    }
}
