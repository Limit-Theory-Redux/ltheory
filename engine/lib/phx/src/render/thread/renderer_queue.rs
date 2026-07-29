use glam::Mat4;
use tracing::error;

use crate::math::Matrix;
use crate::render::{
    BatchStats, BlendMode, CameraUboData, CmdPrimitiveType, CullFace, GpuHandle, LightUboData,
    MaterialUboData, RenderBatch, RenderCommand, Renderer,
};

// =============================================================================
// FFI-exposed Renderer API
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
        self.active_batch = Some(RenderBatch::new(view, projection, eye_x, eye_y, eye_z));
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
        if let Some(batch) = &mut self.active_batch {
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
        if let Some(batch) = &self.active_batch {
            Some(batch.get_stats())
        } else {
            error!("There is no active batch started. Use begin_batch() to start it.");
            None
        }
    }

    // === State Management ===

    /// Set the viewport
    pub fn set_viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        self.submit(RenderCommand::SetViewport {
            x,
            y,
            width,
            height,
        });
    }

    /// Set the scissor region
    pub fn set_scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        self.submit(RenderCommand::SetScissor {
            x,
            y,
            width,
            height,
        });
    }

    /// Enable or disable scissor test
    pub fn enable_scissor(&self, enable: bool) {
        self.submit(RenderCommand::EnableScissor(enable));
    }

    /// Set blend mode (0=Disabled, 1=Alpha, 2=Additive, 3=PreMultAlpha)
    pub fn set_blend_mode(&self, mode: i32) {
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
    pub fn set_cull_face(&self, face: i32) {
        let cull_face = match face {
            0 => CullFace::None,
            1 => CullFace::Back,
            2 => CullFace::Front,
            _ => CullFace::None,
        };
        self.submit(RenderCommand::SetCullFace(cull_face));
    }

    /// Enable or disable depth testing
    pub fn set_depth_test(&self, enable: bool) {
        self.submit(RenderCommand::SetDepthTest(enable));
    }

    /// Enable or disable depth writing
    pub fn set_depth_writable(&self, enable: bool) {
        self.submit(RenderCommand::SetDepthWritable(enable));
    }

    /// Set wireframe mode
    pub fn set_wireframe(&self, enable: bool) {
        self.submit(RenderCommand::SetWireframe(enable));
    }

    // === Shader Operations ===

    /// Bind a shader program
    pub fn bind_shader(&self, handle: u32) {
        self.submit(RenderCommand::BindShader {
            handle: GpuHandle(handle),
        });
    }

    /// Unbind the current shader
    pub fn unbind_shader(&self) {
        self.submit(RenderCommand::UnbindShader);
    }

    /// Set an integer uniform
    pub fn set_uniform_int(&self, location: i32, value: i32) {
        self.submit(RenderCommand::SetUniformInt { location, value });
    }

    /// Set a float uniform
    pub fn set_uniform_float(&self, location: i32, value: f32) {
        self.submit(RenderCommand::SetUniformFloat { location, value });
    }

    /// Set a vec2 uniform
    pub fn set_uniform_float2(&self, location: i32, x: f32, y: f32) {
        self.submit(RenderCommand::SetUniformFloat2 {
            location,
            value: [x, y],
        });
    }

    /// Set a vec3 uniform
    pub fn set_uniform_float3(&self, location: i32, x: f32, y: f32, z: f32) {
        self.submit(RenderCommand::SetUniformFloat3 {
            location,
            value: [x, y, z],
        });
    }

    /// Set a vec4 uniform
    pub fn set_uniform_float4(&self, location: i32, x: f32, y: f32, z: f32, w: f32) {
        self.submit(RenderCommand::SetUniformFloat4 {
            location,
            value: [x, y, z, w],
        });
    }

    // === Texture Operations ===

    /// Bind a 2D texture to a slot
    pub fn bind_texture_2d(&self, slot: u32, handle: u32) {
        self.submit(RenderCommand::BindTexture2D {
            slot,
            handle: GpuHandle(handle),
        });
    }

    /// Bind a 3D texture to a slot
    pub fn bind_texture_3d(&self, slot: u32, handle: u32) {
        self.submit(RenderCommand::BindTexture3D {
            slot,
            handle: GpuHandle(handle),
        });
    }

    /// Bind a cube texture to a slot
    pub fn bind_texture_cube(&self, slot: u32, handle: u32) {
        self.submit(RenderCommand::BindTextureCube {
            slot,
            handle: GpuHandle(handle),
        });
    }

    /// Unbind a texture from a slot
    pub fn unbind_texture(&self, slot: u32) {
        self.submit(RenderCommand::UnbindTexture { slot });
    }

    // === Framebuffer Operations ===

    /// Bind a framebuffer
    pub fn bind_framebuffer(&self, handle: u32) {
        self.submit(RenderCommand::BindFramebuffer {
            handle: GpuHandle(handle),
        });
    }

    /// Bind the default framebuffer
    pub fn bind_default_framebuffer(&self) {
        self.submit(RenderCommand::BindDefaultFramebuffer);
    }

    /// Clear color buffer
    pub fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        self.submit(RenderCommand::Clear {
            color: Some([r, g, b, a]),
            depth: None,
        });
    }

    /// Clear depth buffer
    pub fn clear_depth(&self, depth: f32) {
        self.submit(RenderCommand::Clear {
            color: None,
            depth: Some(depth),
        });
    }

    /// Clear both color and depth buffers
    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32, depth: f32) {
        self.submit(RenderCommand::Clear {
            color: Some([r, g, b, a]),
            depth: Some(depth),
        });
    }

    // === Drawing Operations ===

    /// Draw a mesh
    pub fn draw_mesh(&self, vao: u32, index_count: i32) {
        self.submit(RenderCommand::DrawMesh {
            vao: GpuHandle(vao),
            index_count,
            primitive: CmdPrimitiveType::Triangles,
        });
    }

    /// Draw a mesh with a specific primitive type
    pub fn draw_mesh_primitive(&self, vao: u32, index_count: i32, primitive: i32) {
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
    pub fn draw_mesh_instanced(&self, vao: u32, index_count: i32, instance_count: i32) {
        self.submit(RenderCommand::DrawMeshInstanced {
            vao: GpuHandle(vao),
            index_count,
            instance_count,
            primitive: CmdPrimitiveType::Triangles,
        });
    }

    // === Window Operations ===

    /// Signal resize
    pub fn resize(&self, width: u32, height: u32) {
        self.submit(RenderCommand::Resize { width, height });
    }

    /// Signal swap buffers (frame end)
    pub fn swap_buffers(&self) {
        self.submit(RenderCommand::SwapBuffers);
    }

    // === Camera UBO ===

    /// Create the camera UBO on the render thread
    #[bind(name = "CreateCameraUBO")]
    pub fn create_camera_ubo(&self) {
        self.submit(RenderCommand::CreateCameraUBO);
    }

    /// Update the camera UBO with new camera data
    /// Parameters are the matrices and vectors that make up the camera state.
    #[bind(name = "UpdateCameraUBO")]
    pub fn update_camera_ubo(
        &self,
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

        self.submit(RenderCommand::UpdateCameraUBO { data: boxed });
    }

    /// Create the material UBO on the render thread
    #[bind(name = "CreateMaterialUBO")]
    pub fn create_material_ubo(&self) {
        self.submit(RenderCommand::CreateMaterialUBO);
    }

    /// Update the material UBO with new material properties
    #[bind(name = "UpdateMaterialUBO")]
    pub fn update_material_ubo(
        &self,
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

        // Convert to boxed array for command
        self.submit(RenderCommand::UpdateMaterialUBO {
            data: *data.as_bytes(),
        });
    }

    /// Create the light UBO on the render thread
    #[bind(name = "CreateLightUBO")]
    pub fn create_light_ubo(&self) {
        self.submit(RenderCommand::CreateLightUBO);
    }

    /// Update the light UBO with light properties
    #[bind(name = "UpdateLightUBO")]
    pub fn update_light_ubo(
        &self,
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

        // Convert to boxed array for command
        self.submit(RenderCommand::UpdateLightUBO {
            data: *data.as_bytes(),
        });
    }
}
