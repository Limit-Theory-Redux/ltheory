#![allow(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;

use tracing::{debug, error, info, warn};

use crate::render::gl::types::GLsizeiptr;
use crate::render::gl::{self};
use crate::render::thread::{
    AttachKey, FboKey, GpuResource, MAX_TEXTURE_SLOTS, TextureBinding, TextureType,
};
use crate::render::{
    BlendMode, BlockLayout, CmdPrimitiveType, CommandCategory, CommandExecutor, CommandReply,
    CullFace, ImmVertex, LoadOp, MAX_COLOR_ATTACHMENTS, PolygonMode, RenderPassDesc, RenderStats,
    ResourceId, ShaderLayout, ShaderReloadResult, TexFilter, TexFormat, TexWrapMode, VertexFormat,
    ViewDim,
};
use crate::window::{PresentMode, WindowGlContext};

const DRAW_BUFS: [u32; 4] = [
    gl::COLOR_ATTACHMENT0,
    gl::COLOR_ATTACHMENT1,
    gl::COLOR_ATTACHMENT2,
    gl::COLOR_ATTACHMENT3,
];

impl CommandExecutor {
    pub(super) fn init_gl_intern(&mut self) {
        unsafe {
            // Reset GL state to known defaults - context may have inherited state from main thread
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::BindVertexArray(0);
            gl::UseProgram(0);

            // =================================================================
            // CRITICAL: Match ALL GL state from glutin_render.rs init_renderer
            // Missing any of these causes rendering differences!
            // =================================================================

            // Disable multisampling (matches main thread)
            gl::Disable(gl::MULTISAMPLE);

            // Culling defaults
            gl::Disable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);

            // Pixel store alignment (1 byte for fonts with odd widths)
            gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            // Depth function
            gl::DepthFunc(gl::LEQUAL);

            // Blending - MUST be enabled for fonts!
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE, gl::ZERO);

            // Seamless cubemap filtering
            gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS);

            // Line rendering
            gl::Disable(gl::LINE_SMOOTH);
            gl::Hint(gl::LINE_SMOOTH_HINT, gl::FASTEST);
            #[cfg(not(target_os = "macos"))]
            gl::LineWidth(2.0f32);

            // =================================================================
            // Match RenderState::push_all_defaults() initial values
            // =================================================================

            // Depth test disabled by default (push_depth_test(false))
            gl::Disable(gl::DEPTH_TEST);

            // Depth writable true by default (push_depth_writable(true))
            gl::DepthMask(gl::TRUE);

            // Wireframe disabled by default (push_wireframe(false))
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);

            // Log initial state for debugging
            let mut current_fbo: i32 = 0;
            let mut current_vao: i32 = 0;
            let mut viewport: [i32; 4] = [0; 4];
            gl::GetIntegerv(gl::DRAW_FRAMEBUFFER_BINDING, &mut current_fbo);
            gl::GetIntegerv(gl::VERTEX_ARRAY_BINDING, &mut current_vao);
            gl::GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());
            info!(
                "Render thread GL state after reset: FBO={}, VAO={}, viewport={:?}",
                current_fbo, current_vao, viewport
            );

            // Create VAO/VBO for immediate mode rendering
            gl::GenVertexArrays(1, &mut self.imm_vao);
            gl::GenBuffers(1, &mut self.imm_vbo);

            gl::BindVertexArray(self.imm_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.imm_vbo);

            // Setup vertex attributes for ImmVertex: pos (3f), normal (3f), uv (2f), color (4f)
            // Attribute locations must match shader.rs BindAttribLocation calls:
            //   0 = vertex_position, 1 = vertex_normal, 2 = vertex_uv, 3 = vertex_color
            const STRIDE: i32 = std::mem::size_of::<ImmVertex>() as i32; // 12 floats = 48 bytes

            // Position attribute (location 0 = vertex_position)
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, STRIDE, std::ptr::null());

            // Normal attribute (location 1 = vertex_normal)
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, STRIDE, (3 * 4) as *const _);

            // UV attribute (location 2 = vertex_uv)
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, STRIDE, (6 * 4) as *const _);

            // Color attribute (location 3 = vertex_color)
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(3, 4, gl::FLOAT, gl::FALSE, STRIDE, (8 * 4) as *const _);

            gl::BindVertexArray(0);
        }

        self.init_fullscreen_quad();
    }

    #[inline(always)]
    pub(super) fn cmd_set_viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        let _sa = self.record_command(CommandCategory::State, false, true);
        unsafe {
            gl::Viewport(x, y, width, height);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_scissor(&mut self, x: i32, y: i32, width: i32, height: i32) {
        let _sa = self.record_command(CommandCategory::State, false, true);
        unsafe {
            gl::Scissor(x, y, width, height);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_enable_scissor(&mut self, enable: bool) {
        let _sa = self.record_command(CommandCategory::State, false, true);
        unsafe {
            if enable {
                gl::Enable(gl::SCISSOR_TEST);
            } else {
                gl::Disable(gl::SCISSOR_TEST);
            }
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_depth_test(&mut self, enable: bool) {
        let _sa = self.record_command(CommandCategory::State, false, true);
        // S6: remove the write-through (pipelines own this state).
        self.binding.gl_state.depth_test = Some(enable);
        self.invalidate_pipeline();
        unsafe {
            if enable {
                gl::Enable(gl::DEPTH_TEST);
            } else {
                gl::Disable(gl::DEPTH_TEST);
            }
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_depth_writable(&mut self, enable: bool) {
        let _sa = self.record_command(CommandCategory::State, false, true);
        self.binding.gl_state.depth_write = Some(enable); // S6: remove
        self.invalidate_pipeline();
        unsafe {
            gl::DepthMask(if enable { gl::TRUE } else { gl::FALSE });
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_wireframe(&mut self, enable: bool) {
        let _sa = self.record_command(CommandCategory::State, false, true);
        // S6: remove
        self.binding.gl_state.polygon = Some(if enable {
            PolygonMode::Line
        } else {
            PolygonMode::Fill
        });
        self.invalidate_pipeline();
        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, if enable { gl::LINE } else { gl::FILL });
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_line_width(&mut self, width: f32) {
        let _sa = self.record_command(CommandCategory::State, false, true);
        unsafe {
            gl::LineWidth(width);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_point_size(&mut self, size: f32) {
        let _sa = self.record_command(CommandCategory::State, false, true);
        unsafe {
            gl::PointSize(size);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_bind_shader(&mut self, handle: super::GpuHandle) {
        let _sa = self.record_command(CommandCategory::Shader, false, true);
        self.invalidate_pipeline(); // S6: remove
        self.this_frame_stats.shader_bind_commands += 1;
        if handle.0 == self.current_program {
            self.this_frame_stats.shader_redundant_binds += 1;
        } else {
            self.this_frame_stats.shader_distinct_programs += 1;
            // NOTE: deliberately do NOT invalidate the texture cache here.
            // glUseProgram does not touch texture bindings; the cache keys
            // on (slot, handle, type) and self-corrects when a different
            // texture is bound. Invalidating on every shader switch was
            // wiping the cache ~2k times/frame, keeping the hit rate at
            // ~0% and forcing redundant glBindTexture calls.
            unsafe {
                gl::UseProgram(handle.0);
            }
            self.current_program = handle.0;
        }
    }

    #[inline(always)]
    pub(super) fn cmd_bind_shader_by_resource(
        &mut self,
        id: ResourceId,
        shader_key: Option<String>,
    ) {
        let _sa = self.record_command(CommandCategory::Shader, false, false);
        self.invalidate_pipeline(); // S6: remove
        // First check if there's a hot-reloaded version of this shader
        let program = if let Some(ref key) = shader_key {
            self.hot_reloaded_shaders.get(key).copied()
        } else {
            None
        };

        // Fall back to resource if no hot-reload version
        let program = program.or_else(|| {
            if let Some(GpuResource::Shader { program }) = self.resources.get(&id) {
                Some(*program)
            } else {
                None
            }
        });

        if let Some(p) = program {
            self.this_frame_stats.shader_bind_commands += 1;
            if p == self.current_program {
                self.this_frame_stats.shader_redundant_binds += 1;
            } else {
                self.this_frame_stats.shader_distinct_programs += 1;
                // NOTE: no texture-cache invalidation here either - see
                // cmd_bind_shader: glUseProgram does not affect bindings.
                unsafe {
                    gl::UseProgram(p);
                }
                self.current_program = p;
            }
        } else {
            error!("BindShaderByResource: resource {:?} not found!", id);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_unbind_shader(&mut self) {
        let _sa = self.record_command(CommandCategory::Shader, false, true);
        // NOTE: deliberately do NOT invalidate the texture cache here.
        // glUseProgram(0) leaves texture bindings untouched; the cache
        // remains valid across program switches and self-corrects on any
        // real texture change. Previously this wiped the cache on every
        // shader stop (~2k/frame), destroying all reuse.
        self.this_frame_stats.texture_invalidations_on_shader_unbind += 1;
        self.invalidate_pipeline(); // S6: remove
        unsafe {
            gl::UseProgram(0);
        }
        self.current_program = 0;
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_int(&mut self, location: i32, value: i32) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        unsafe {
            gl::Uniform1i(location, value);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_int2(&mut self, location: i32, value: [i32; 2]) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        unsafe {
            gl::Uniform2i(location, value[0], value[1]);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_int3(&mut self, location: i32, value: [i32; 3]) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        unsafe {
            gl::Uniform3i(location, value[0], value[1], value[2]);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_int4(&mut self, location: i32, value: [i32; 4]) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        unsafe {
            gl::Uniform4i(location, value[0], value[1], value[2], value[3]);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_float(&mut self, location: i32, value: f32) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        unsafe {
            gl::Uniform1f(location, value);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_float2(&mut self, location: i32, value: [f32; 2]) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        unsafe {
            gl::Uniform2f(location, value[0], value[1]);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_float3(&mut self, location: i32, value: [f32; 3]) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        unsafe {
            gl::Uniform3f(location, value[0], value[1], value[2]);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_float4(&mut self, location: i32, value: [f32; 4]) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        unsafe {
            gl::Uniform4f(location, value[0], value[1], value[2], value[3]);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_mat4(&mut self, location: i32, value: [f32; 16]) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, value.as_ptr());
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_int_by_name(&mut self, name: Arc<str>, value: i32) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform1i(loc, value);
            }
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_int2_by_name(&mut self, name: Arc<str>, value: [i32; 2]) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform2i(loc, value[0], value[1]);
            }
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_int3_by_name(&mut self, name: Arc<str>, value: [i32; 3]) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform3i(loc, value[0], value[1], value[2]);
            }
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_int4_by_name(&mut self, name: Arc<str>, value: [i32; 4]) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform4i(loc, value[0], value[1], value[2], value[3]);
            }
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_float_by_name(&mut self, name: Arc<str>, value: f32) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform1f(loc, value);
            }
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_float2_by_name(&mut self, name: Arc<str>, value: [f32; 2]) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform2f(loc, value[0], value[1]);
            }
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_float3_by_name(&mut self, name: Arc<str>, value: [f32; 3]) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform3f(loc, value[0], value[1], value[2]);
            }
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_float4_by_name(&mut self, name: Arc<str>, value: [f32; 4]) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform4f(loc, value[0], value[1], value[2], value[3]);
            }
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_uniform_mat4_by_name(&mut self, name: Arc<str>, value: [f32; 16]) {
        let _sa = self.record_command(CommandCategory::Uniform, false, false);
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::UniformMatrix4fv(loc, 1, gl::FALSE, value.as_ptr());
            }
        }
    }

    #[inline(always)]
    pub(super) fn cmd_bind_texture_2d(&mut self, slot: u32, handle: super::GpuHandle) {
        let _sa = self.record_command(CommandCategory::Texture, false, true);
        self.bind_texture_cached(slot, handle.0, TextureType::Texture2D);
    }

    #[inline(always)]
    pub(super) fn cmd_bind_texture_2d_by_resource(&mut self, slot: u32, id: ResourceId) {
        let _sa = self.record_command(CommandCategory::Texture, false, false);
        if let Some(GpuResource::Texture2D { handle }) = self.resources.get(&id) {
            self.bind_texture_cached(slot, *handle, TextureType::Texture2D);
        } else {
            warn!("BindTexture2DByResource: resource {:?} not found", id);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_bind_texture_1d_by_resource(&mut self, slot: u32, id: ResourceId) {
        let _sa = self.record_command(CommandCategory::Texture, false, false);
        if let Some(GpuResource::Texture1D { handle }) = self.resources.get(&id) {
            self.bind_texture_cached(slot, *handle, TextureType::Texture1D);
        } else {
            warn!("BindTexture1DByResource: resource {:?} not found", id);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_bind_texture_3d(&mut self, slot: u32, handle: super::GpuHandle) {
        let _sa = self.record_command(CommandCategory::Texture, false, true);
        self.bind_texture_cached(slot, handle.0, TextureType::Texture3D);
    }

    #[inline(always)]
    pub(super) fn cmd_bind_texture_3d_by_resource(&mut self, slot: u32, id: ResourceId) {
        let _sa = self.record_command(CommandCategory::Texture, false, false);
        if let Some(GpuResource::Texture3D { handle }) = self.resources.get(&id) {
            self.bind_texture_cached(slot, *handle, TextureType::Texture3D);
        } else {
            warn!("BindTexture3DByResource: resource {:?} not found", id);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_bind_texture_cube(&mut self, slot: u32, handle: super::GpuHandle) {
        let _sa = self.record_command(CommandCategory::Texture, false, true);
        self.bind_texture_cached(slot, handle.0, TextureType::TextureCube);
    }

    #[inline(always)]
    pub(super) fn cmd_bind_texture_cube_by_resource(&mut self, slot: u32, id: ResourceId) {
        let _sa = self.record_command(CommandCategory::Texture, false, false);
        if let Some(GpuResource::TextureCube { handle }) = self.resources.get(&id) {
            self.bind_texture_cached(slot, *handle, TextureType::TextureCube);
        } else {
            warn!("BindTextureCubeByResource: resource {:?} not found", id);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_unbind_texture(&mut self, slot: u32) {
        let _sa = self.record_command(CommandCategory::Texture, false, false);
        self.unbind_texture_cached(slot);
    }

    #[inline(always)]
    pub(super) fn cmd_set_texture_2d_mag_filter(
        &mut self,
        handle: super::GpuHandle,
        filter: TexFilter,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, handle.0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter as i32);
        }
        self.restore_active_unit_binding();
    }

    #[inline(always)]
    pub(super) fn cmd_set_texture_2d_min_filter(
        &mut self,
        handle: super::GpuHandle,
        filter: TexFilter,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, handle.0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as i32);
        }
        self.restore_active_unit_binding();
    }

    #[inline(always)]
    pub(super) fn cmd_set_texture_2d_wrap_mode(
        &mut self,
        handle: super::GpuHandle,
        mode: TexWrapMode,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, handle.0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, mode as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, mode as i32);
        }
        self.restore_active_unit_binding();
    }

    #[inline(always)]
    pub(super) fn cmd_set_texture_2d_mip_range(
        &mut self,
        handle: super::GpuHandle,
        min_level: i32,
        max_level: i32,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, handle.0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, min_level);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, max_level);
        }
        self.restore_active_unit_binding();
    }

    #[inline(always)]
    pub(super) fn cmd_generate_mipmap_2d(&mut self, handle: super::GpuHandle) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, handle.0);
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
        self.restore_active_unit_binding();
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn cmd_update_texture_2d_data(
        &mut self,
        handle: super::GpuHandle,
        width: i32,
        height: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, handle.0);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format,
                width,
                height,
                0,
                pixel_format,
                data_format,
                data.as_ptr() as *const _,
            );
            // Re-apply texture parameters after TexImage2D to ensure consistent state
            // (some drivers may reset parameters on texture reallocation)
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        }
        self.restore_active_unit_binding();
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn cmd_update_texture_2d_data_by_resource(
        &mut self,
        id: ResourceId,
        width: i32,
        height: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        if let Some(GpuResource::Texture2D { handle }) = self.resources.get(&id) {
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, *handle);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    internal_format,
                    width,
                    height,
                    0,
                    pixel_format,
                    data_format,
                    data.as_ptr() as *const _,
                );
                // Re-apply texture parameters after TexImage2D to ensure consistent state
                // (some drivers may reset parameters on texture reallocation)
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            }
            self.restore_active_unit_binding();
        } else {
            warn!("UpdateTexture2DDataByResource: resource {:?} not found", id);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn cmd_update_texture_2d_rect(
        &mut self,
        id: ResourceId,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        if let Some(GpuResource::Texture2D { handle }) = self.resources.get(&id) {
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, *handle);
                // Rows are tightly packed (R8 atlas rows are not 4-byte aligned in general).
                gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                gl::TexSubImage2D(
                    gl::TEXTURE_2D,
                    0,
                    x,
                    y,
                    width,
                    height,
                    pixel_format,
                    data_format,
                    data.as_ptr() as *const _,
                );
                gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
            }
            self.restore_active_unit_binding();
        } else {
            warn!("UpdateTexture2DRect: resource {:?} not found", id);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_texture_2d_anisotropy(&mut self, handle: super::GpuHandle, factor: f32) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, handle.0);
            gl::TexParameterf(gl::TEXTURE_2D, gl::TEXTURE_MAX_ANISOTROPY_EXT, factor);
        }
        self.restore_active_unit_binding();
    }

    #[inline(always)]
    pub(super) fn cmd_set_texture_2d_anisotropy_by_resource(
        &mut self,
        id: ResourceId,
        factor: f32,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        if let Some((target, handle)) = self.texture_target_and_handle(id) {
            unsafe {
                gl::BindTexture(target, handle);
                gl::TexParameterf(target, gl::TEXTURE_MAX_ANISOTROPY_EXT, factor);
            }
            self.restore_active_unit_binding();
        } else {
            warn!(
                "SetTexture2DAnisotropyByResource: resource {:?} not found",
                id
            );
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_texture_2d_mip_range_by_resource(
        &mut self,
        id: ResourceId,
        min_level: i32,
        max_level: i32,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        if let Some((target, handle)) = self.texture_target_and_handle(id) {
            unsafe {
                gl::BindTexture(target, handle);
                gl::TexParameteri(target, gl::TEXTURE_BASE_LEVEL, min_level);
                gl::TexParameteri(target, gl::TEXTURE_MAX_LEVEL, max_level);
            }
            self.note_mip_range(id, min_level, max_level); // S6: remove
            self.restore_active_unit_binding();
        } else {
            warn!(
                "SetTexture2DMipRangeByResource: resource {:?} not found",
                id
            );
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_texel_1d_by_resource(&mut self, id: ResourceId, x: i32, color: [f32; 4]) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        if let Some((target, handle)) = self.texture_target_and_handle(id) {
            unsafe {
                gl::BindTexture(target, handle);
                gl::TexSubImage1D(
                    target,
                    0,
                    x,
                    1,
                    gl::RGBA,
                    gl::FLOAT,
                    color.as_ptr() as *const _,
                );
            }
            self.restore_active_unit_binding();
        } else {
            warn!("SetTexel1DByResource: resource {:?} not found", id);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_texel_2d_by_resource(
        &mut self,
        id: ResourceId,
        x: i32,
        y: i32,
        color: [f32; 4],
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        if let Some((target, handle)) = self.texture_target_and_handle(id) {
            unsafe {
                gl::BindTexture(target, handle);
                gl::TexSubImage2D(
                    target,
                    0,
                    x,
                    y,
                    1,
                    1,
                    gl::RGBA,
                    gl::FLOAT,
                    color.as_ptr() as *const _,
                );
            }
            self.restore_active_unit_binding();
        } else {
            warn!("SetTexel2DByResource: resource {:?} not found", id);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_texture_mag_filter_by_resource(
        &mut self,
        id: ResourceId,
        filter: TexFilter,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        if let Some((target, handle)) = self.texture_target_and_handle(id) {
            unsafe {
                gl::BindTexture(target, handle);
                gl::TexParameteri(target, gl::TEXTURE_MAG_FILTER, filter as i32);
            }
            self.restore_active_unit_binding();
        } else {
            warn!("SetTextureMagFilterByResource: resource {:?} not found", id);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_texture_min_filter_by_resource(
        &mut self,
        id: ResourceId,
        filter: TexFilter,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        if let Some((target, handle)) = self.texture_target_and_handle(id) {
            unsafe {
                gl::BindTexture(target, handle);
                gl::TexParameteri(target, gl::TEXTURE_MIN_FILTER, filter as i32);
            }
            self.restore_active_unit_binding();
        } else {
            warn!("SetTextureMinFilterByResource: resource {:?} not found", id);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_texture_wrap_mode_by_resource(
        &mut self,
        id: ResourceId,
        mode: TexWrapMode,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        if let Some((target, handle)) = self.texture_target_and_handle(id) {
            unsafe {
                gl::BindTexture(target, handle);
                gl::TexParameteri(target, gl::TEXTURE_WRAP_S, mode as i32);
                if target != gl::TEXTURE_1D {
                    gl::TexParameteri(target, gl::TEXTURE_WRAP_T, mode as i32);
                }
                if target == gl::TEXTURE_3D {
                    gl::TexParameteri(target, gl::TEXTURE_WRAP_R, mode as i32);
                }
            }
            self.restore_active_unit_binding();
        } else {
            warn!("SetTextureWrapModeByResource: resource {:?} not found", id);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_generate_mipmap_by_resource(&mut self, id: ResourceId) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        if let Some((target, handle)) = self.texture_target_and_handle(id) {
            unsafe {
                gl::BindTexture(target, handle);
                gl::GenerateMipmap(target);
            }
            self.restore_active_unit_binding();
        } else {
            warn!("GenerateMipmapByResource: resource {:?} not found", id);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_update_texture_1d_data_by_resource(
        &mut self,
        id: ResourceId,
        width: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        if let Some(GpuResource::Texture1D { handle }) = self.resources.get(&id) {
            unsafe {
                gl::BindTexture(gl::TEXTURE_1D, *handle);
                gl::TexImage1D(
                    gl::TEXTURE_1D,
                    0,
                    internal_format,
                    width,
                    0,
                    pixel_format,
                    data_format,
                    data.as_ptr() as *const _,
                );
                gl::TexParameteri(gl::TEXTURE_1D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                gl::TexParameteri(gl::TEXTURE_1D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                gl::TexParameteri(gl::TEXTURE_1D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::BindTexture(gl::TEXTURE_1D, 0);
            }
        } else {
            warn!("UpdateTexture1DDataByResource: resource {:?} not found", id);
        }
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn cmd_update_texture_3d_data_by_resource(
        &mut self,
        id: ResourceId,
        width: i32,
        height: i32,
        depth: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        if let Some(GpuResource::Texture3D { handle }) = self.resources.get(&id) {
            unsafe {
                gl::BindTexture(gl::TEXTURE_3D, *handle);
                gl::TexImage3D(
                    gl::TEXTURE_3D,
                    0,
                    internal_format,
                    width,
                    height,
                    depth,
                    0,
                    pixel_format,
                    data_format,
                    data.as_ptr() as *const _,
                );
                gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
                gl::BindTexture(gl::TEXTURE_3D, 0);
            }
        } else {
            warn!("UpdateTexture3DDataByResource: resource {:?} not found", id);
        }
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn cmd_update_texture_cube_face_data_by_resource(
        &mut self,
        id: ResourceId,
        face: u32,
        level: i32,
        size: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        if let Some(GpuResource::TextureCube { handle }) = self.resources.get(&id) {
            unsafe {
                gl::BindTexture(gl::TEXTURE_CUBE_MAP, *handle);
                gl::TexImage2D(
                    face,
                    level,
                    internal_format,
                    size,
                    size,
                    0,
                    pixel_format,
                    data_format,
                    data.as_ptr() as *const _,
                );
            }
            // Not `BindTexture(.., 0)`: unit 0 holds the environment cube map.
            self.restore_active_unit_binding();
        } else {
            warn!(
                "UpdateTextureCubeFaceDataByResource: resource {:?} not found",
                id
            );
        }
    }

    #[inline(always)]
    pub(super) fn cmd_copy_texture_2d_from_framebuffer_by_resource(
        &mut self,
        id: ResourceId,
        internal_format: i32,
        width: i32,
        height: i32,
    ) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        if let Some(GpuResource::Texture2D { handle }) = self.resources.get(&id) {
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, *handle);
                gl::CopyTexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    internal_format as u32,
                    0,
                    0,
                    width,
                    height,
                    0,
                );
            }
            self.restore_active_unit_binding();
        } else {
            warn!(
                "CopyTexture2DFromFramebufferByResource: resource {:?} not found",
                id
            );
        }
    }

    #[inline(always)]
    pub(super) fn cmd_read_texture_1d_data(
        &mut self,
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
    ) -> Vec<u8> {
        let _sa = self.record_command(CommandCategory::Readback, false, false);
        let mut data = Vec::new();
        if let Some(GpuResource::Texture1D { handle }) = self.resources.get(&id) {
            unsafe {
                let mut width = 0;
                gl::BindTexture(gl::TEXTURE_1D, *handle);
                gl::GetTexLevelParameteriv(gl::TEXTURE_1D, 0, gl::TEXTURE_WIDTH, &mut width);
                data = vec![0u8; self.texel_buffer_size(width, 1, 1, pixel_format, data_format)];
                gl::GetTexImage(
                    gl::TEXTURE_1D,
                    0,
                    pixel_format,
                    data_format,
                    data.as_mut_ptr() as *mut _,
                );
                gl::BindTexture(gl::TEXTURE_1D, 0);
            }
        } else {
            warn!("ReadTexture1DData: resource {:?} not found", id);
        }
        data
    }

    #[inline(always)]
    pub(super) fn cmd_read_texture_2d_data(
        &mut self,
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
    ) -> Vec<u8> {
        let _sa = self.record_command(CommandCategory::Readback, false, false);
        let mut data = Vec::new();
        if let Some(GpuResource::Texture2D { handle }) = self.resources.get(&id) {
            unsafe {
                let (mut width, mut height) = (0, 0);
                gl::BindTexture(gl::TEXTURE_2D, *handle);
                gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_WIDTH, &mut width);
                gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_HEIGHT, &mut height);
                data =
                    vec![0u8; self.texel_buffer_size(width, height, 1, pixel_format, data_format)];
                gl::GetTexImage(
                    gl::TEXTURE_2D,
                    0,
                    pixel_format,
                    data_format,
                    data.as_mut_ptr() as *mut _,
                );
            }
            self.restore_active_unit_binding();
        } else {
            warn!("ReadTexture2DData: resource {:?} not found", id);
        }
        data
    }

    #[inline(always)]
    pub(super) fn cmd_read_texture_3d_data(
        &mut self,
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
    ) -> Vec<u8> {
        let _sa = self.record_command(CommandCategory::Readback, false, false);
        let mut data = Vec::new();
        if let Some(GpuResource::Texture3D { handle }) = self.resources.get(&id) {
            unsafe {
                let (mut width, mut height, mut depth) = (0, 0, 0);
                gl::BindTexture(gl::TEXTURE_3D, *handle);
                gl::GetTexLevelParameteriv(gl::TEXTURE_3D, 0, gl::TEXTURE_WIDTH, &mut width);
                gl::GetTexLevelParameteriv(gl::TEXTURE_3D, 0, gl::TEXTURE_HEIGHT, &mut height);
                gl::GetTexLevelParameteriv(gl::TEXTURE_3D, 0, gl::TEXTURE_DEPTH, &mut depth);
                data = vec![
                    0u8;
                    self.texel_buffer_size(width, height, depth, pixel_format, data_format)
                ];
                gl::GetTexImage(
                    gl::TEXTURE_3D,
                    0,
                    pixel_format,
                    data_format,
                    data.as_mut_ptr() as *mut _,
                );
                gl::BindTexture(gl::TEXTURE_3D, 0);
            }
        } else {
            warn!("ReadTexture3DData: resource {:?} not found", id);
        }
        data
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn cmd_read_texture_cube_face_data(
        &mut self,
        id: ResourceId,
        face: u32,
        level: i32,
        pixel_format: u32,
        data_format: u32,
    ) -> Vec<u8> {
        let _sa = self.record_command(CommandCategory::Readback, false, false);
        let mut data = Vec::new();
        if let Some(GpuResource::TextureCube { handle }) = self.resources.get(&id) {
            unsafe {
                let mut size = 0;
                gl::BindTexture(gl::TEXTURE_CUBE_MAP, *handle);
                gl::GetTexLevelParameteriv(face, level, gl::TEXTURE_WIDTH, &mut size);
                data = vec![0u8; self.texel_buffer_size(size, size, 1, pixel_format, data_format)];
                gl::GetTexImage(
                    face,
                    level,
                    pixel_format,
                    data_format,
                    data.as_mut_ptr() as *mut _,
                );
                gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);
            }
        } else {
            warn!("ReadTextureCubeFaceData: resource {:?} not found", id);
        }
        data
    }

    #[inline(always)]
    pub(super) fn cmd_sample_pixel_2d_by_resource(
        &mut self,
        id: ResourceId,
        x: i32,
        y: i32,
    ) -> [u8; 4] {
        let _sa = self.record_command(CommandCategory::Readback, false, false);
        let mut pixel = [0u8; 4];
        if let Some(GpuResource::Texture2D { handle }) = self.resources.get(&id) {
            unsafe {
                let mut fbo = 0;
                gl::GenFramebuffers(1, &mut fbo);
                gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    *handle,
                    0,
                );
                if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE {
                    gl::ReadPixels(
                        x,
                        y,
                        1,
                        1,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        pixel.as_mut_ptr() as *mut _,
                    );
                } else {
                    warn!("SamplePixel2DByResource: incomplete framebuffer");
                }
                // Restore the framebuffer of the open pass (or the default one).
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.bound_fbo);
                gl::DeleteFramebuffers(1, &fbo);
            }
        } else {
            warn!("SamplePixel2DByResource: resource {:?} not found", id);
        }
        pixel
    }

    #[inline(always)]
    pub(super) fn cmd_read_framebuffer_pixels(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Vec<u8> {
        let _sa = self.record_command(CommandCategory::Readback, false, false);
        let mut data = vec![0u8; (width * height * 4) as usize];
        unsafe {
            gl::ReadPixels(
                x,
                y,
                width,
                height,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_mut_ptr() as *mut _,
            );
        }
        data
    }

    /// Framebuffer for `desc`'s attachments, created on first use and cached
    /// until one of its textures is destroyed. Returns `None` if a texture is
    /// missing or the framebuffer is incomplete.
    fn framebuffer_for_pass(&mut self, desc: &RenderPassDesc) -> Option<u32> {
        let key = FboKey {
            color: std::array::from_fn(|i| {
                desc.color[i].as_ref().map(|c| AttachKey {
                    id: c.view.tex,
                    dim: c.view.dim,
                    mip: c.view.base_mip,
                })
            }),
            depth: desc.depth.as_ref().map(|d| AttachKey {
                id: d.view.tex,
                dim: d.view.dim,
                mip: d.view.base_mip,
            }),
        };
        if let Some(&fbo) = self.fbo_cache.get(&key) {
            return Some(fbo);
        }

        let mut fbo = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
        }
        let mut ok = true;
        let mut color_count = 0;
        for (i, att) in key.color.iter().enumerate() {
            if let Some(att) = att {
                ok &= self.attach_to_bound_fbo(gl::COLOR_ATTACHMENT0 + i as u32, att);
                color_count += 1;
            }
        }
        if let Some(att) = &key.depth {
            ok &= self.attach_to_bound_fbo(gl::DEPTH_ATTACHMENT, att);
        }
        unsafe {
            if color_count > 0 {
                gl::DrawBuffers(color_count, DRAW_BUFS.as_ptr());
            } else {
                gl::DrawBuffer(gl::NONE);
                gl::ReadBuffer(gl::NONE);
            }
            if ok {
                let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
                if status != gl::FRAMEBUFFER_COMPLETE {
                    error!(
                        "RenderPass '{}': incomplete framebuffer (status {status:#x})",
                        desc.label
                    );
                    ok = false;
                }
            }
            if !ok {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                gl::DeleteFramebuffers(1, &fbo);
                return None;
            }
        }

        self.fbo_cache.insert(key, fbo);
        let mut ids: Vec<ResourceId> = key.color.iter().flatten().map(|a| a.id).collect();
        ids.extend(key.depth.map(|a| a.id));
        for id in ids {
            self.texture_fbos.entry(id).or_default().push(key);
        }
        Some(fbo)
    }

    /// Attach one view to the currently bound framebuffer.
    fn attach_to_bound_fbo(&self, attachment: u32, att: &AttachKey) -> bool {
        let level = att.mip as i32;
        unsafe {
            match (self.resources.get(&att.id), att.dim) {
                (Some(GpuResource::Texture2D { handle }), ViewDim::D2) => {
                    gl::FramebufferTexture2D(
                        gl::FRAMEBUFFER,
                        attachment,
                        gl::TEXTURE_2D,
                        *handle,
                        level,
                    );
                }
                (Some(GpuResource::Texture3D { handle }), ViewDim::D2Layer(layer)) => {
                    gl::FramebufferTextureLayer(
                        gl::FRAMEBUFFER,
                        attachment,
                        *handle,
                        level,
                        layer as i32,
                    );
                }
                (Some(GpuResource::TextureCube { handle }), ViewDim::CubeFace(face)) => {
                    gl::FramebufferTexture2D(
                        gl::FRAMEBUFFER,
                        attachment,
                        face as u32,
                        *handle,
                        level,
                    );
                }
                (resource, dim) => {
                    error!(
                        "RenderPass: cannot attach {:?} ({dim:?}) as a render target (resource: {})",
                        att.id,
                        if resource.is_some() {
                            "wrong texture kind"
                        } else {
                            "not found"
                        }
                    );
                    return false;
                }
            }
        }
        true
    }

    pub(super) fn cmd_begin_render_pass(&mut self, desc: &RenderPassDesc) {
        let _sa = self.record_command(CommandCategory::Framebuffer, false, false);

        // If a texture is missing or the FBO is incomplete the error was
        // logged at creation; fall back to the default framebuffer so the
        // rest of the pass cannot touch the textures.
        let fbo = if desc.backbuffer {
            0
        } else {
            self.framebuffer_for_pass(desc).unwrap_or(0)
        };
        self.bound_fbo = fbo;

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::Viewport(0, 0, desc.extent[0] as i32, desc.extent[1] as i32);
        }
        self.reset_pass_state();

        // Load ops. `DontCare` needs no work on GL 3.3.
        let mut color_ops: [Option<[f32; 4]>; MAX_COLOR_ATTACHMENTS] =
            [None; MAX_COLOR_ATTACHMENTS];
        let depth_op: Option<f32>;
        if desc.backbuffer {
            if desc.back_color.0 == LoadOp::Clear {
                color_ops[0] = Some(desc.back_color.1);
            }
            depth_op = (desc.back_depth.0 == LoadOp::Clear).then_some(desc.back_depth.1);
        } else {
            for (i, c) in desc.color.iter().enumerate() {
                if let Some(c) = c.as_ref().filter(|c| c.load == LoadOp::Clear) {
                    color_ops[i] = Some(c.clear);
                }
            }
            depth_op = desc
                .depth
                .as_ref()
                .filter(|d| d.load == LoadOp::Clear)
                .map(|d| d.clear);
        }
        if color_ops.iter().all(|c| c.is_none()) && depth_op.is_none() {
            return;
        }

        unsafe {
            // Clears honour the write masks and the scissor, so force them
            // open for the clear and put them back afterwards.
            let mut depth_mask: u8 = gl::TRUE;
            gl::GetBooleanv(gl::DEPTH_WRITEMASK, &mut depth_mask);
            let scissor = gl::IsEnabled(gl::SCISSOR_TEST) == gl::TRUE;
            if depth_mask != gl::TRUE {
                gl::DepthMask(gl::TRUE);
            }
            if scissor {
                gl::Disable(gl::SCISSOR_TEST);
            }

            for (index, color) in color_ops.iter().enumerate() {
                if let Some(color) = color {
                    gl::ClearBufferfv(gl::COLOR, index as i32, color.as_ptr());
                }
            }
            if let Some(depth) = depth_op {
                gl::ClearBufferfv(gl::DEPTH, 0, &depth);
            }

            if scissor {
                gl::Enable(gl::SCISSOR_TEST);
            }
            if depth_mask != gl::TRUE {
                gl::DepthMask(depth_mask);
            }
        }
    }

    pub(super) fn cmd_end_render_pass(&mut self) {
        let _sa = self.record_command(CommandCategory::Framebuffer, false, false);
        self.bound_fbo = 0;
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_bind_mesh(&mut self, vao: super::GpuHandle) {
        let _sa = self.record_command(CommandCategory::Mesh, false, false);
        unsafe {
            gl::BindVertexArray(vao.0);
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::EnableVertexAttribArray(2);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_unbind_mesh(&mut self) {
        let _sa = self.record_command(CommandCategory::Mesh, false, false);
        unsafe {
            gl::DisableVertexAttribArray(0);
            gl::DisableVertexAttribArray(1);
            gl::DisableVertexAttribArray(2);
            gl::BindVertexArray(0);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_draw_mesh(
        &mut self,
        vao: super::GpuHandle,
        index_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        let _sa = self.record_command(CommandCategory::Draw, true, false);
        self.this_frame_stats.draw_mesh_calls += 1;
        unsafe {
            gl::BindVertexArray(vao.0);
            gl::DrawElements(
                primitive.to_gl(),
                index_count,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
            gl::BindVertexArray(0);
        }
        self.this_frame_stats.vertices_drawn += index_count.max(0) as u64;
    }

    #[inline(always)]
    pub(super) fn cmd_draw_mesh_by_resource(
        &mut self,
        id: ResourceId,
        index_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        let _sa = self.record_command(CommandCategory::Draw, true, false);
        self.this_frame_stats.draw_mesh_calls += 1;
        if let Some(GpuResource::Mesh { vao, .. }) = self.resources.get(&id) {
            unsafe {
                gl::BindVertexArray(*vao);
                gl::DrawElements(
                    primitive.to_gl(),
                    index_count,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
                gl::BindVertexArray(0);
            }
            self.this_frame_stats.vertices_drawn += index_count.max(0) as u64;
        } else {
            warn!("DrawMeshByResource: resource {id:?} not found");
        }
    }

    #[inline(always)]
    pub(super) fn cmd_bind_mesh_by_resource(&mut self, id: ResourceId) {
        let _sa = self.record_command(CommandCategory::Mesh, false, false);
        if let Some(GpuResource::Mesh { vao, .. }) = self.resources.get(&id) {
            unsafe {
                gl::BindVertexArray(*vao);
                gl::EnableVertexAttribArray(0);
                gl::EnableVertexAttribArray(1);
                gl::EnableVertexAttribArray(2);
            }
        } else {
            warn!("BindMeshByResource: resource {:?} not found", id);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_create_shader(
        &mut self,
        id: ResourceId,
        vertex_src: String,
        fragment_src: String,
        layout: &ShaderLayout,
    ) -> Result<Vec<BlockLayout>, String> {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        if !self.has_gl_context() {
            // No-op: leave the resource untracked. By-resource commands
            // (e.g. `GetUniformLocationByResource`) already handle a
            // missing id gracefully, so nothing downstream needs a real
            // GL program.
            return Ok(Vec::new());
        }
        match self.create_shader(&vertex_src, &fragment_src, layout) {
            Ok((program, blocks)) => {
                self.resources.insert(id, GpuResource::Shader { program });
                Ok(blocks)
            }
            Err(e) => {
                error!("Failed to create shader {:?}: {}", id, e);
                Err(e)
            }
        }
    }

    #[inline(always)]
    pub(super) fn cmd_get_uniform_location_by_resource(
        &mut self,
        id: ResourceId,
        name: Arc<str>,
    ) -> i32 {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        if let Some(GpuResource::Shader { program }) = self.resources.get(&id) {
            let program = *program;
            self.get_uniform_location_for_program(program, &name)
        } else {
            warn!("GetUniformLocationByResource: resource {:?} not found", id);
            -1
        }
    }

    #[inline(always)]
    pub(super) fn cmd_create_texture_1d(
        &mut self,
        id: ResourceId,
        width: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    ) {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        let handle = self.create_texture_1d(width, format, data.as_deref());
        self.resources.insert(id, GpuResource::Texture1D { handle });
    }

    #[inline(always)]
    pub(super) fn cmd_create_texture_2d(
        &mut self,
        id: ResourceId,
        width: u32,
        height: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    ) {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        let handle = self.create_texture_2d(width, height, format, data.as_deref());
        self.resources.insert(id, GpuResource::Texture2D { handle });
    }

    #[inline(always)]
    pub(super) fn cmd_create_texture_3d(
        &mut self,
        id: ResourceId,
        width: u32,
        height: u32,
        depth: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    ) {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        let handle = self.create_texture_3d(width, height, depth, format, data.as_deref());
        self.resources.insert(id, GpuResource::Texture3D { handle });
    }

    #[inline(always)]
    pub(super) fn cmd_create_texture_cube(&mut self, id: ResourceId, size: u32, format: TexFormat) {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        let handle = self.create_texture_cube(size, format);
        self.resources
            .insert(id, GpuResource::TextureCube { handle });
    }

    #[inline(always)]
    pub(super) fn cmd_create_mesh(
        &mut self,
        id: ResourceId,
        vertices: Vec<u8>,
        indices: Vec<u32>,
        vertex_format: VertexFormat,
    ) {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        let (vao, vbo, ebo) = self.create_mesh(&vertices, &indices, &vertex_format);
        self.resources
            .insert(id, GpuResource::Mesh { vao, vbo, ebo });
    }

    #[inline(always)]
    pub(super) fn cmd_destroy_resource(&mut self, ids: &[ResourceId]) {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        for id in ids {
            self.forget_texture(*id);
            // Evict every framebuffer that attaches this texture.
            if let Some(keys) = self.texture_fbos.remove(id) {
                for key in keys {
                    if let Some(fbo) = self.fbo_cache.remove(&key) {
                        if fbo == self.bound_fbo {
                            self.bound_fbo = 0;
                        }
                        unsafe {
                            gl::DeleteFramebuffers(1, &fbo);
                        }
                    }
                }
            }
            if let Some(resource) = self.resources.remove(id) {
                self.destroy_resource(resource);
            }
        }
    }

    #[inline(always)]
    pub(super) fn cmd_resize(&mut self, width: u32, height: u32) {
        let _sa = self.record_command(CommandCategory::Sync, false, false);
        // Only update the viewport - surface resize is handled by the window system.
        // Note: Calling ctx.resize() here was causing freezes during window resize,
        // likely due to synchronization issues with the window manager.
        // The viewport update is sufficient for correct rendering.
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_present_mode(&mut self, mode: PresentMode) {
        let _sa = self.record_command(CommandCategory::Sync, false, false);
        let Some(ref ctx) = self.gl_context else {
            return;
        };
        match ctx.set_swap_interval(mode) {
            Ok(()) => info!("Present mode set to {mode:?}"),
            Err(e) => warn!("Unable to set present mode {mode:?}: {e}"),
        }
    }

    #[inline(always)]
    pub(super) fn cmd_reload_shader(
        &mut self,
        shader_key: &str,
        vertex_src: &str,
        fragment_src: &str,
    ) -> CommandReply {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        // Compile shader on render thread and send result back
        // The legacy reload command carries no layout; `Shader::reload` (a
        // fresh `CreateShader`) is what hot reload uses.
        let result = match self
            .create_shader(vertex_src, fragment_src, &ShaderLayout::default())
            .map(|(program, _)| program)
        {
            Ok(program) => {
                // Delete old hot-reloaded shader if exists
                if let Some(old_program) = self.hot_reloaded_shaders.remove(shader_key) {
                    // Clear uniform cache for the old program to prevent stale lookups
                    // (GL may reuse the program ID for a new shader)
                    self.uniform_caches.remove(&old_program);
                    unsafe {
                        gl::DeleteProgram(old_program);
                    }
                    debug!("Deleted previous hot-reloaded shader for '{shader_key}'",);
                }

                // Store the new program for this shader_key
                self.hot_reloaded_shaders
                    .insert(shader_key.to_string(), program);
                info!(
                    "Shader '{shader_key}' reloaded successfully on render thread (program={program})",
                );

                ShaderReloadResult {
                    shader_key: shader_key.into(),
                    error: None,
                    program,
                }
            }
            Err(e) => {
                warn!("Shader '{shader_key}' reload failed: {e}");
                // Push error to global queue for UI overlay
                // push_shader_error(&shader_key, "compile", &e);
                ShaderReloadResult {
                    shader_key: shader_key.into(),
                    error: Some(e),
                    program: 0,
                }
            }
        };
        CommandReply::ShaderReload(result)
    }

    #[inline(always)]
    pub(super) fn cmd_swap_buffers(&mut self) -> CommandReply {
        // Calculate frame time before swap
        let frame_time = self.frame_start.elapsed();
        let frame_time_us = frame_time.as_micros() as u64;

        self.stats.frame_count += 1;

        // Snapshot stats at end of frame; readable via `stats_snapshot()`
        // at any later point, and handed back as a reply so the
        // threaded backend can forward it to the main thread. The executor's
        // per-frame counters already live in `RenderStats` field shape, so
        // they transfer verbatim - only the cross-frame/cumulative fields
        // are filled in here.
        self.last_stats = RenderStats {
            commands_processed: self.stats.commands_processed,
            draw_calls: self.stats.draw_calls,
            state_changes: self.stats.state_changes,
            frame_count: self.stats.frame_count,
            last_frame_time_us: frame_time_us,
            present_wait_us: 0,
            texture_binds_skipped_cumulative: self.texture_binds_skipped,
            ..self.this_frame_stats.clone()
        };

        // Fence the frame's commands (uniform ring slot reuse waits on this).
        self.insert_slot_fence();

        // Perform actual buffer swap if we have a GL context
        let present_start = std::time::Instant::now();
        if let Some(ref ctx) = self.gl_context {
            if let Err(e) = ctx.swap_buffers() {
                error!("Failed to swap buffers: {}", e);
            }
            // Measure vsync/present wait (may block until the next vblank)
            self.last_stats.present_wait_us = present_start.elapsed().as_micros() as u64;
        } else if self.stats.frame_count == 1 {
            error!("SwapBuffers: no GL context available!");
        }

        // Reset per-frame counters and start new frame timing
        self.this_frame_stats = RenderStats::default();
        self.frame_start = std::time::Instant::now();

        CommandReply::Stats(Box::new(self.last_stats.clone()))
    }

    #[inline(always)]
    pub(super) fn cmd_flush(&mut self) {
        let _sa = self.record_command(CommandCategory::Sync, false, false);
        unsafe {
            gl::Finish();
        }
    }

    #[inline(always)]
    pub(super) fn cmd_fence(&mut self, fence_id: u64) -> CommandReply {
        let _sa = self.record_command(CommandCategory::Sync, false, false);
        CommandReply::Fence(fence_id)
    }

    #[inline(always)]
    pub(super) fn cmd_pacing_fence(&mut self, fence_id: u64) -> CommandReply {
        let _sa = self.record_command(CommandCategory::Sync, false, false);
        CommandReply::PacingFence(fence_id)
    }

    #[inline(always)]
    pub(super) fn cmd_set_blend_mode(&mut self, mode: BlendMode) {
        let _sa = self.record_command(CommandCategory::State, false, true);
        self.binding.gl_state.blend = Some(mode); // S6: remove
        self.invalidate_pipeline();
        unsafe {
            match mode {
                BlendMode::Disabled => {
                    gl::Disable(gl::BLEND);
                    gl::BlendFunc(gl::ONE, gl::ZERO);
                }
                BlendMode::Alpha => {
                    gl::Enable(gl::BLEND);
                    gl::BlendFuncSeparate(
                        gl::SRC_ALPHA,
                        gl::ONE_MINUS_SRC_ALPHA,
                        gl::ONE,
                        gl::ONE_MINUS_SRC_ALPHA,
                    );
                }
                BlendMode::Additive => {
                    gl::Enable(gl::BLEND);
                    gl::BlendFuncSeparate(gl::ONE, gl::ONE, gl::ONE, gl::ONE);
                }
                BlendMode::PreMultAlpha => {
                    gl::Enable(gl::BLEND);
                    gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
                }
            }
        }
    }

    #[inline(always)]
    pub(super) fn cmd_set_cull_face(&mut self, face: CullFace) {
        let _sa = self.record_command(CommandCategory::State, false, true);
        self.binding.gl_state.cull = Some(face); // S6: remove
        self.invalidate_pipeline();
        unsafe {
            match face {
                CullFace::None => {
                    gl::Disable(gl::CULL_FACE);
                }
                CullFace::Back => {
                    gl::Enable(gl::CULL_FACE);
                    gl::CullFace(gl::BACK);
                }
                CullFace::Front => {
                    gl::Enable(gl::CULL_FACE);
                    gl::CullFace(gl::FRONT);
                }
            }
        }
    }

    #[inline(always)]
    pub(super) fn cmd_draw_immediate(
        &mut self,
        primitive: CmdPrimitiveType,
        vertices: &[ImmVertex],
    ) {
        let _sa = self.record_command(CommandCategory::Draw, true, false);
        self.this_frame_stats.draw_immediate_calls += 1;
        self.this_frame_stats.immediate_vertices += vertices.len() as u64;

        if vertices.is_empty() {
            return;
        }

        unsafe {
            gl::BindVertexArray(self.imm_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.imm_vbo);

            // Use BufferData with STREAM_DRAW for per-frame updates.
            // This "orphans" the old buffer, allowing the driver to reuse memory
            // without GPU stalls (vs BufferSubData which can block).
            let size = std::mem::size_of_val(vertices) as GLsizeiptr;
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size,
                vertices.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );

            // Handle quads by drawing as triangle fans (4 vertices per quad)
            if matches!(primitive, CmdPrimitiveType::Quads) {
                let quad_count = vertices.len() / 4;
                for i in 0..quad_count {
                    gl::DrawArrays(gl::TRIANGLE_FAN, (i * 4) as i32, 4);
                }
            } else {
                gl::DrawArrays(primitive.to_gl(), 0, vertices.len() as i32);
            }

            gl::BindVertexArray(0);
        }
        self.this_frame_stats.vertices_drawn += vertices.len() as u64;
    }

    fn create_shader(
        &self,
        vertex_src: &str,
        fragment_src: &str,
        layout: &ShaderLayout,
    ) -> Result<(u32, Vec<BlockLayout>), String> {
        unsafe {
            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            let vs_src = std::ffi::CString::new(vertex_src).unwrap();
            gl::ShaderSource(vs, 1, &vs_src.as_ptr(), std::ptr::null());
            gl::CompileShader(vs);

            let mut success = 0;
            gl::GetShaderiv(vs, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetShaderiv(vs, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl::GetShaderInfoLog(vs, len, std::ptr::null_mut(), buffer.as_mut_ptr() as *mut _);
                gl::DeleteShader(vs);
                return Err(format!(
                    "Vertex shader error: {}",
                    String::from_utf8_lossy(&buffer)
                ));
            }

            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            let fs_src = std::ffi::CString::new(fragment_src).unwrap();
            gl::ShaderSource(fs, 1, &fs_src.as_ptr(), std::ptr::null());
            gl::CompileShader(fs);

            gl::GetShaderiv(fs, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetShaderiv(fs, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl::GetShaderInfoLog(fs, len, std::ptr::null_mut(), buffer.as_mut_ptr() as *mut _);
                gl::DeleteShader(vs);
                gl::DeleteShader(fs);
                return Err(format!(
                    "Fragment shader error: {}",
                    String::from_utf8_lossy(&buffer)
                ));
            }

            let program = gl::CreateProgram();
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);

            // CRITICAL: Bind attribute locations BEFORE linking!
            // Must match the VAO setup: 0=position, 1=normal, 2=uv, 3=color
            gl::BindAttribLocation(program, 0, c"vertex_position".as_ptr() as *const _);
            gl::BindAttribLocation(program, 1, c"vertex_normal".as_ptr() as *const _);
            gl::BindAttribLocation(program, 2, c"vertex_uv".as_ptr() as *const _);
            gl::BindAttribLocation(program, 3, c"vertex_color".as_ptr() as *const _);
            // Per-instance attributes of `PassCmd::DrawMeshInstanced` (res/shader/include/instanced.glsl);
            // must match its VertexAttribPointer setup: 4-7=mWorld columns, 8=color. No-op
            // (harmless) for shaders that don't declare these names.
            gl::BindAttribLocation(program, 4, c"instance_matrix_col0".as_ptr() as *const _);
            gl::BindAttribLocation(program, 5, c"instance_matrix_col1".as_ptr() as *const _);
            gl::BindAttribLocation(program, 6, c"instance_matrix_col2".as_ptr() as *const _);
            gl::BindAttribLocation(program, 7, c"instance_matrix_col3".as_ptr() as *const _);
            gl::BindAttribLocation(program, 8, c"instance_color".as_ptr() as *const _);
            // Shape parameters of `Imm2DVertex` (`vertex/imm2d.glsl`).
            gl::BindAttribLocation(program, 11, c"imm_params".as_ptr() as *const _);
            gl::BindAttribLocation(program, 12, c"imm_params2".as_ptr() as *const _);

            gl::LinkProgram(program);

            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl::GetProgramInfoLog(
                    program,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut _,
                );
                gl::DeleteShader(vs);
                gl::DeleteShader(fs);
                gl::DeleteProgram(program);
                return Err(format!(
                    "Shader link error: {}",
                    String::from_utf8_lossy(&buffer)
                ));
            }

            // Block binding points and sampler units from the `#group` layout.
            Self::apply_layout(program, layout);
            let blocks = Self::reflect_blocks(program);

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            Ok((program, blocks))
        }
    }

    fn create_texture_2d(
        &self,
        width: u32,
        height: u32,
        format: TexFormat,
        data: Option<&[u8]>,
    ) -> u32 {
        unsafe {
            let mut handle = 0;
            gl::GenTextures(1, &mut handle);
            gl::BindTexture(gl::TEXTURE_2D, handle);

            let (internal_format, gl_format, gl_type) = format.to_gl_formats();

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format as i32,
                width as i32,
                height as i32,
                0,
                gl_format,
                gl_type,
                data.map_or(std::ptr::null(), |d| d.as_ptr() as *const _),
            );

            // Use NEAREST filtering to match direct mode behavior (important for fonts/crisp textures)
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            self.restore_active_unit_binding();
            handle
        }
    }

    fn create_texture_1d(&self, width: u32, format: TexFormat, data: Option<&[u8]>) -> u32 {
        unsafe {
            let mut handle = 0;
            gl::GenTextures(1, &mut handle);
            gl::BindTexture(gl::TEXTURE_1D, handle);

            let (internal_format, gl_format, gl_type) = format.to_gl_formats();

            gl::TexImage1D(
                gl::TEXTURE_1D,
                0,
                internal_format as i32,
                width as i32,
                0,
                gl_format,
                gl_type,
                data.map_or(std::ptr::null(), |d| d.as_ptr() as *const _),
            );

            gl::TexParameteri(gl::TEXTURE_1D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_1D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_1D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);

            self.restore_active_unit_binding();
            handle
        }
    }

    fn create_texture_3d(
        &self,
        width: u32,
        height: u32,
        depth: u32,
        format: TexFormat,
        data: Option<&[u8]>,
    ) -> u32 {
        unsafe {
            let mut handle = 0;
            gl::GenTextures(1, &mut handle);
            gl::BindTexture(gl::TEXTURE_3D, handle);

            let (internal_format, gl_format, gl_type) = format.to_gl_formats();

            gl::TexImage3D(
                gl::TEXTURE_3D,
                0,
                internal_format as i32,
                width as i32,
                height as i32,
                depth as i32,
                0,
                gl_format,
                gl_type,
                data.map_or(std::ptr::null(), |d| d.as_ptr() as *const _),
            );

            gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);

            self.restore_active_unit_binding();
            handle
        }
    }

    /// Creates a cube texture with 6 empty faces (matches `TexCube::new`'s
    /// old direct-GL behavior: null data, `GL_RED`/`GL_BYTE` placeholder
    /// format regardless of `format`, since nothing is actually uploaded yet)
    fn create_texture_cube(&self, size: u32, format: TexFormat) -> u32 {
        unsafe {
            let mut handle = 0;
            gl::GenTextures(1, &mut handle);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, handle);

            const FACES: [gl::types::GLenum; 6] = [
                gl::TEXTURE_CUBE_MAP_POSITIVE_X,
                gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
                gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
                gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
                gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
                gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
            ];
            for face in FACES {
                gl::TexImage2D(
                    face,
                    0,
                    format as i32,
                    size as i32,
                    size as i32,
                    0,
                    gl::RED,
                    gl::BYTE,
                    std::ptr::null(),
                );
            }

            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MAG_FILTER,
                gl::NEAREST as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as i32,
            );

            self.restore_active_unit_binding();
            handle
        }
    }

    /// Byte size of a `w*h*d` block of texels in the given GL pixel/data
    /// format - used to size the buffer for a `glGetTexImage` readback.
    fn texel_buffer_size(
        &self,
        w: i32,
        h: i32,
        d: i32,
        pixel_format: gl::types::GLenum,
        data_format: gl::types::GLenum,
    ) -> usize {
        let components: i32 = match pixel_format {
            gl::RED | gl::DEPTH_COMPONENT => 1,
            gl::RG => 2,
            gl::RGB | gl::BGR => 3,
            gl::RGBA | gl::BGRA => 4,
            _ => 4,
        };
        let element_size: i32 = match data_format {
            gl::BYTE | gl::UNSIGNED_BYTE => 1,
            gl::SHORT | gl::UNSIGNED_SHORT => 2,
            gl::INT | gl::UNSIGNED_INT | gl::FLOAT => 4,
            _ => 1,
        };
        (w * h * d * components * element_size).max(0) as usize
    }

    /// Look up the GL target and handle for any texture-kind resource,
    /// dispatching on which `GpuResource` variant it is.
    fn texture_target_and_handle(&self, id: ResourceId) -> Option<(gl::types::GLenum, u32)> {
        match self.resources.get(&id) {
            Some(GpuResource::Texture1D { handle }) => Some((gl::TEXTURE_1D, *handle)),
            Some(GpuResource::Texture2D { handle }) => Some((gl::TEXTURE_2D, *handle)),
            Some(GpuResource::Texture3D { handle }) => Some((gl::TEXTURE_3D, *handle)),
            Some(GpuResource::TextureCube { handle }) => Some((gl::TEXTURE_CUBE_MAP, *handle)),
            _ => None,
        }
    }

    fn create_mesh(
        &self,
        vertices: &[u8],
        indices: &[u32],
        format: &VertexFormat,
    ) -> (u32, u32, u32) {
        unsafe {
            let mut vao = 0;
            let mut vbo = 0;
            let mut ebo = 0;

            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                vertices.len() as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * 4) as isize,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let stride = format.stride as i32;
            let mut offset = 0;
            let mut location = 0;

            if format.has_position {
                gl::EnableVertexAttribArray(location);
                gl::VertexAttribPointer(
                    location,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    stride,
                    offset as *const _,
                );
                offset += 12; // 3 floats
                location += 1;
            }

            if format.has_normal {
                gl::EnableVertexAttribArray(location);
                gl::VertexAttribPointer(
                    location,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    stride,
                    offset as *const _,
                );
                offset += 12; // 3 floats
                location += 1;
            }

            if format.has_uv {
                gl::EnableVertexAttribArray(location);
                gl::VertexAttribPointer(
                    location,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    stride,
                    offset as *const _,
                );
                offset += 8; // 2 floats
                location += 1;
            }

            if format.has_color {
                gl::EnableVertexAttribArray(location);
                gl::VertexAttribPointer(
                    location,
                    4,
                    gl::FLOAT,
                    gl::FALSE,
                    stride,
                    offset as *const _,
                );
            }

            gl::BindVertexArray(0);

            (vao, vbo, ebo)
        }
    }

    fn destroy_resource(&self, resource: GpuResource) {
        unsafe {
            match resource {
                GpuResource::Shader { program } => {
                    gl::DeleteProgram(program);
                }
                GpuResource::Texture1D { handle }
                | GpuResource::Texture2D { handle }
                | GpuResource::Texture3D { handle }
                | GpuResource::TextureCube { handle } => {
                    gl::DeleteTextures(1, &handle);
                }
                GpuResource::Mesh { vao, vbo, ebo } => {
                    gl::DeleteVertexArrays(1, &vao);
                    gl::DeleteBuffers(1, &vbo);
                    gl::DeleteBuffers(1, &ebo);
                }
            }
        }
    }

    /// Destroy every GPU resource and give the GL context back.
    ///
    /// Returns the released context, or `None` if there was none or the
    /// platform could not release it (macOS leaks it to avoid a dispatch_sync
    /// deadlock). The caller decides what to do with it.
    pub(super) fn cleanup(&mut self) -> Option<WindowGlContext> {
        info!(
            "Cleaning up render thread resources ({} resources to clean)",
            self.resources.len()
        );

        // Destroy all remaining resources
        let resources: Vec<_> = self.resources.drain().collect();
        for (_, resource) in resources {
            self.destroy_resource(resource);
        }
        info!("Resources cleaned up");

        // Cleanup cached FBOs
        unsafe {
            let fbo_count = self.fbo_cache.len();
            for (_, fbo) in self.fbo_cache.drain() {
                gl::DeleteFramebuffers(1, &fbo);
            }
            self.texture_fbos.clear();
            if fbo_count > 0 {
                info!("Cleaned up {} cached FBOs", fbo_count);
            }
        }

        // Binding model objects
        unsafe {
            for buffer in self.binding.ring_buffers.iter().flatten() {
                gl::DeleteBuffers(1, buffer);
            }
            for sampler in self.binding.samplers.values() {
                gl::DeleteSamplers(1, sampler);
            }
            if self.binding.copy_fbos[0] != 0 {
                gl::DeleteFramebuffers(2, self.binding.copy_fbos.as_ptr());
            }
            if self.binding.fullscreen_vao != 0 {
                gl::DeleteVertexArrays(1, &self.binding.fullscreen_vao);
                gl::DeleteBuffers(1, &self.binding.fullscreen_vbo);
            }
            for vao in [self.binding.imm2d_vao, self.binding.imm3d_vao] {
                if vao != 0 {
                    gl::DeleteVertexArrays(1, &vao);
                }
            }
        }

        // Cleanup immediate mode resources
        unsafe {
            if self.imm_vao != 0 {
                gl::DeleteVertexArrays(1, &self.imm_vao);
            }
            if self.imm_vbo != 0 {
                gl::DeleteBuffers(1, &self.imm_vbo);
            }
        }
        info!("Immediate mode resources cleaned up");

        // Flush and finish all pending GL commands before releasing context
        if self.has_gl_context() {
            unsafe {
                gl::Flush();
                gl::Finish();
            }
            info!("GL commands flushed");
        }

        // Release GL context back to main thread (platform-specific)
        // WindowActiveGlContext::release_for_main_thread() handles:
        // - macOS: Uses mem::forget to avoid dispatch_sync deadlock, returns Err
        // - Linux/Windows: Properly releases and returns context + surface
        let Some(gl_ctx) = self.gl_context.take() else {
            info!("No GL context to return");
            return None;
        };

        info!("Releasing GL context...");
        match gl_ctx.release_for_main_thread() {
            Ok((not_current_ctx, surface)) => {
                info!("GL context released");
                Some(WindowGlContext {
                    context: not_current_ctx,
                    surface,
                })
            }
            Err(e) => {
                // On macOS, this is expected - context was leaked to avoid deadlock
                warn!("Could not release GL context: {e} - marking unavailable");
                None
            }
        }
    }

    fn bind_texture_cached(&mut self, slot: u32, handle: u32, tex_type: TextureType) -> bool {
        // S6: remove. A legacy bind samples with the parameters stored in
        // the texture, not with a sampler object a pass left on the unit.
        self.drop_unit_sampler(slot as usize);
        let slot_idx = slot as usize;
        if slot_idx >= MAX_TEXTURE_SLOTS {
            // Slot out of range, just bind directly
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + slot);
                gl::BindTexture(tex_type.to_gl_target(), handle);
                gl::ActiveTexture(gl::TEXTURE0);
            }
            self.this_frame_stats.texture_bind_calls += 1;
            return true;
        }

        let new_binding = TextureBinding::new(handle, tex_type);
        let current = &self.texture_bindings[slot_idx];

        // Check if already bound
        if current.handle == handle && current.tex_type == Some(tex_type) {
            self.texture_binds_skipped += 1;
            self.this_frame_stats.texture_binds_skipped += 1;
            return false;
        }

        // Different texture or type - need to bind
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(tex_type.to_gl_target(), handle);
            gl::ActiveTexture(gl::TEXTURE0);
        }

        self.this_frame_stats.texture_bind_calls += 1;
        self.texture_bindings[slot_idx] = new_binding;
        true
    }

    /// Unbind texture from slot (bind 0)
    fn unbind_texture_cached(&mut self, slot: u32) {
        let slot_idx = slot as usize;
        if slot_idx < MAX_TEXTURE_SLOTS {
            let current = &self.texture_bindings[slot_idx];
            if current.handle == 0 {
                // Already unbound
                self.texture_binds_skipped += 1;
                self.this_frame_stats.texture_binds_skipped += 1;
                return;
            }

            // Unbind based on current type
            if let Some(tex_type) = current.tex_type {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0 + slot);
                    gl::BindTexture(tex_type.to_gl_target(), 0);
                    gl::ActiveTexture(gl::TEXTURE0);
                }
                self.this_frame_stats.texture_bind_calls += 1;
            }

            self.texture_bindings[slot_idx] = TextureBinding::unbound();
        } else {
            // Slot out of range, can't track - just unbind 2D as fallback
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + slot);
                gl::BindTexture(gl::TEXTURE_2D, 0);
                gl::ActiveTexture(gl::TEXTURE0);
            }
            self.this_frame_stats.texture_bind_calls += 1;
        }
    }

    /// Re-bind the cached texture for the active unit (slot 0 by the same
    /// invariant as `mark_active_unit_unbound`) after a direct-bind setter
    /// finished its param change. Keeps the cache authoritative: the setter
    /// temporarily bound a texture on unit 0 and left 0 bound; restoring the
    /// cached texture means the next `bind_texture_cached` for that slot can
    /// still legitimately skip. Read-only so it can be called from `&self`
    /// setters without signature churn.
    fn restore_active_unit_binding(&self) {
        let binding = &self.texture_bindings[0];
        unsafe {
            if binding.handle != 0 {
                if let Some(tex_type) = binding.tex_type {
                    gl::BindTexture(tex_type.to_gl_target(), binding.handle);
                    return;
                }
            }
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    /// Get uniform location with per-shader caching to avoid repeated gl::GetUniformLocation calls.
    /// Cache is keyed by (program, name) - preserves locations across shader switches.
    /// Takes `&str` so callers never need to own an `Arc<str>` just to do a
    /// lookup; an `Arc<str>` is only allocated internally on a cache miss.
    /// Returns -1 if uniform not found (matches OpenGL behavior).
    fn get_uniform_location_cached(&mut self, name: &str) -> i32 {
        if self.current_program == 0 {
            return -1;
        }
        self.get_uniform_location_for_program(self.current_program, name)
    }

    /// Same caching as `get_uniform_location_cached`, but for an explicitly
    /// named program rather than whichever one is currently bound - used to
    /// resolve a uniform's location for a shader that may not be bound yet
    /// (e.g. right after `CreateShader`, or from `GetUniformLocationByResource`).
    fn get_uniform_location_for_program(&mut self, program: u32, name: &str) -> i32 {
        let cache = self
            .uniform_caches
            .entry(program)
            .or_insert_with(|| HashMap::with_capacity(32));

        if let Some(&loc) = cache.get(name) {
            self.this_frame_stats.uniform_cache_hits += 1;
            return loc;
        }

        self.this_frame_stats.uniform_cache_misses += 1;
        let c_name = std::ffi::CString::new(name).unwrap_or_default();
        let loc = unsafe { gl::GetUniformLocation(program, c_name.as_ptr()) };

        // Store in cache (even if -1 to avoid repeated lookups for non-existent uniforms)
        cache.insert(Arc::from(name), loc);
        loc
    }
}
