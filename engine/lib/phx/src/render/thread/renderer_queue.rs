use glam::Mat4;
use tracing::error;

use crate::math::Matrix;
use crate::render::{
    BatchStats, BlendMode, CameraUboData, CmdPrimitiveType, CullFace, GpuHandle, LightUboData,
    MaterialUboData, RenderBatch, Renderer,
};

// =============================================================================
// FFI-exposed Renderer API
//
// These are thin Lua-facing wrappers: they convert Lua-friendly primitives
// (ints, separate floats, ...) into the native types the per-command methods
// on `Renderer` (defined in `renderer_immediate.rs`/`renderer_threaded.rs`)
// expect, then call those directly. They're renamed relative to the
// per-command methods purely to avoid clashing with them by name - the Lua
// name (see `#[bind(name = "...")]`) is unchanged from before this split.
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

    pub fn add_entity(
        &mut self,
        transform: &[f32; 16],
        bounds_center_x: f32,
        bounds_center_y: f32,
        bounds_center_z: f32,
        bounds_radius: f32,
        mesh_vao: u32,
        index_count: i32,
        shader_handle: u32,
        sort_key: u32,
    ) {
        if let Some(batch) = &mut self.data.active_batch {
            batch.add_entity(
                transform,
                bounds_center_x,
                bounds_center_y,
                bounds_center_z,
                bounds_radius,
                mesh_vao,
                index_count,
                shader_handle,
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
    #[bind(name = "setViewport")]
    pub fn set_viewport_lua(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.set_viewport(x, y, width, height);
    }

    /// Set the scissor region
    #[bind(name = "setScissor")]
    pub fn set_scissor_lua(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.set_scissor(x, y, width, height);
    }

    /// Enable or disable scissor test
    #[bind(name = "enableScissor")]
    pub fn enable_scissor_lua(&mut self, enable: bool) {
        self.enable_scissor(enable);
    }

    /// Set blend mode (0=Disabled, 1=Alpha, 2=Additive, 3=PreMultAlpha)
    #[bind(name = "setBlendMode")]
    pub fn set_blend_mode_lua(&mut self, mode: i32) {
        let blend_mode = match mode {
            0 => BlendMode::Disabled,
            1 => BlendMode::Alpha,
            2 => BlendMode::Additive,
            3 => BlendMode::PreMultAlpha,
            _ => BlendMode::Alpha,
        };
        self.set_blend_mode(blend_mode);
    }

    /// Set cull face (0=None, 1=Back, 2=Front)
    #[bind(name = "setCullFace")]
    pub fn set_cull_face_lua(&mut self, face: i32) {
        let cull_face = match face {
            0 => CullFace::None,
            1 => CullFace::Back,
            2 => CullFace::Front,
            _ => CullFace::None,
        };
        self.set_cull_face(cull_face);
    }

    /// Enable or disable depth testing
    #[bind(name = "setDepthTest")]
    pub fn set_depth_test_lua(&mut self, enable: bool) {
        self.set_depth_test(enable);
    }

    /// Enable or disable depth writing
    #[bind(name = "setDepthWritable")]
    pub fn set_depth_writable_lua(&mut self, enable: bool) {
        self.set_depth_writable(enable);
    }

    /// Set wireframe mode
    #[bind(name = "setWireframe")]
    pub fn set_wireframe_lua(&mut self, enable: bool) {
        self.set_wireframe(enable);
    }

    // === Shader Operations ===

    /// Bind a shader program
    #[bind(name = "bindShader")]
    pub fn bind_shader_lua(&mut self, handle: u32) {
        self.bind_shader(GpuHandle(handle));
    }

    /// Unbind the current shader
    #[bind(name = "unbindShader")]
    pub fn unbind_shader_lua(&mut self) {
        self.unbind_shader();
    }

    /// Set an integer uniform
    #[bind(name = "setUniformInt")]
    pub fn set_uniform_int_lua(&mut self, location: i32, value: i32) {
        self.set_uniform_int(location, value);
    }

    /// Set a float uniform
    #[bind(name = "setUniformFloat")]
    pub fn set_uniform_float_lua(&mut self, location: i32, value: f32) {
        self.set_uniform_float(location, value);
    }

    /// Set a vec2 uniform
    #[bind(name = "setUniformFloat2")]
    pub fn set_uniform_float2_lua(&mut self, location: i32, x: f32, y: f32) {
        self.set_uniform_float2(location, [x, y]);
    }

    /// Set a vec3 uniform
    #[bind(name = "setUniformFloat3")]
    pub fn set_uniform_float3_lua(&mut self, location: i32, x: f32, y: f32, z: f32) {
        self.set_uniform_float3(location, [x, y, z]);
    }

    /// Set a vec4 uniform
    #[bind(name = "setUniformFloat4")]
    pub fn set_uniform_float4_lua(&mut self, location: i32, x: f32, y: f32, z: f32, w: f32) {
        self.set_uniform_float4(location, [x, y, z, w]);
    }

    // === Texture Operations ===

    /// Bind a 2D texture to a slot
    #[bind(name = "bindTexture2D")]
    pub fn bind_texture_2d_lua(&mut self, slot: u32, handle: u32) {
        self.bind_texture_2d(slot, GpuHandle(handle));
    }

    /// Bind a 3D texture to a slot
    #[bind(name = "bindTexture3D")]
    pub fn bind_texture_3d_lua(&mut self, slot: u32, handle: u32) {
        self.bind_texture_3d(slot, GpuHandle(handle));
    }

    /// Bind a cube texture to a slot
    #[bind(name = "bindTextureCube")]
    pub fn bind_texture_cube_lua(&mut self, slot: u32, handle: u32) {
        self.bind_texture_cube(slot, GpuHandle(handle));
    }

    /// Unbind a texture from a slot
    #[bind(name = "unbindTexture")]
    pub fn unbind_texture_lua(&mut self, slot: u32) {
        self.unbind_texture(slot);
    }

    // === Framebuffer Operations ===

    /// Bind a framebuffer
    #[bind(name = "bindFramebuffer")]
    pub fn bind_framebuffer_lua(&mut self, handle: u32) {
        self.bind_framebuffer(GpuHandle(handle));
    }

    /// Bind the default framebuffer
    #[bind(name = "bindDefaultFramebuffer")]
    pub fn bind_default_framebuffer_lua(&mut self) {
        self.bind_default_framebuffer();
    }

    /// Clear color buffer
    #[bind(name = "clearColor")]
    pub fn clear_color_lua(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.clear(Some([r, g, b, a]), None);
    }

    /// Clear depth buffer
    #[bind(name = "clearDepth")]
    pub fn clear_depth_lua(&mut self, depth: f32) {
        self.clear(None, Some(depth));
    }

    /// Clear both color and depth buffers
    #[bind(name = "clear")]
    pub fn clear_lua(&mut self, r: f32, g: f32, b: f32, a: f32, depth: f32) {
        self.clear(Some([r, g, b, a]), Some(depth));
    }

    // === Drawing Operations ===

    /// Draw a mesh
    #[bind(name = "drawMesh")]
    pub fn draw_mesh_lua(&mut self, vao: u32, index_count: i32) {
        self.draw_mesh(GpuHandle(vao), index_count, CmdPrimitiveType::Triangles);
    }

    /// Draw a mesh with a specific primitive type
    #[bind(name = "drawMeshPrimitive")]
    pub fn draw_mesh_primitive_lua(&mut self, vao: u32, index_count: i32, primitive: i32) {
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
        self.draw_mesh(GpuHandle(vao), index_count, prim);
    }

    /// Draw instanced mesh
    #[bind(name = "drawMeshInstanced")]
    pub fn draw_mesh_instanced_lua(&mut self, vao: u32, index_count: i32, instance_count: i32) {
        self.draw_mesh_instanced(
            GpuHandle(vao),
            index_count,
            instance_count,
            CmdPrimitiveType::Triangles,
        );
    }

    // === Window Operations ===

    /// Signal resize
    #[bind(name = "resize")]
    pub fn resize_lua(&mut self, width: u32, height: u32) {
        self.resize(width, height);
    }

    /// Signal swap buffers (frame end)
    #[bind(name = "swapBuffers")]
    pub fn swap_buffers_lua(&mut self) {
        self.swap_buffers();
    }

    // === Camera UBO ===

    /// Create the camera UBO on the render thread
    #[bind(name = "CreateCameraUBO")]
    pub fn create_camera_ubo_lua(&mut self) {
        self.create_camera_ubo();
    }

    /// Update the camera UBO with new camera data
    /// Parameters are the matrices and vectors that make up the camera state.
    #[bind(name = "UpdateCameraUBO")]
    #[allow(clippy::too_many_arguments)]
    pub fn update_camera_ubo_lua(
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
        data.set_proj(&proj);
        data.set_eye(glam::vec3(eye_x, eye_y, eye_z));
        data.set_star_dir(glam::vec3(star_dir_x, star_dir_y, star_dir_z));

        // Convert to boxed array for command
        let bytes = data.as_bytes();
        let mut boxed: Box<[u8; 288]> = Box::new([0u8; 288]);
        boxed.copy_from_slice(bytes);

        self.update_camera_ubo(boxed);
    }

    /// Create the material UBO on the render thread
    #[bind(name = "CreateMaterialUBO")]
    pub fn create_material_ubo_lua(&mut self) {
        self.create_material_ubo();
    }

    /// Update the material UBO with new material properties
    #[bind(name = "UpdateMaterialUBO")]
    pub fn update_material_ubo_lua(
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

        self.update_material_ubo(*data.as_bytes());
    }

    /// Create the light UBO on the render thread
    #[bind(name = "CreateLightUBO")]
    pub fn create_light_ubo_lua(&mut self) {
        self.create_light_ubo();
    }

    /// Update the light UBO with light properties
    #[bind(name = "UpdateLightUBO")]
    #[allow(clippy::too_many_arguments)]
    pub fn update_light_ubo_lua(
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

        self.update_light_ubo(*data.as_bytes());
    }
}
