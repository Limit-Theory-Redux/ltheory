#![allow(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;

use tracing::{debug, error, info, warn};

use crate::render::gl::types::GLsizeiptr;
use crate::render::gl::{self};
use crate::render::thread::{
    FBO_STACK_DEPTH, FboEntry, GpuResource, MAX_TEXTURE_SLOTS, TextureBinding, TextureType,
};
use crate::render::{
    BlendMode, CmdPrimitiveType, CommandExecutor, CommandReply, CullFace, GenericUniformName,
    ImmVertex, InstanceData, RenderStats, ResourceId, ShaderReloadResult, TexFilter, TexFormat,
    TexWrapMode, VertexFormat,
};
use crate::window::WindowGlContext;

const DRAW_BUFS: [u32; 4] = [
    gl::COLOR_ATTACHMENT0,
    gl::COLOR_ATTACHMENT1,
    gl::COLOR_ATTACHMENT2,
    gl::COLOR_ATTACHMENT3,
];

impl CommandExecutor {
    pub(super) fn cmd_set_viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            gl::Viewport(x, y, width, height);
        }
    }

    pub(super) fn cmd_set_scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            gl::Scissor(x, y, width, height);
        }
    }

    pub(super) fn cmd_enable_scissor(&self, enable: bool) {
        unsafe {
            if enable {
                gl::Enable(gl::SCISSOR_TEST);
            } else {
                gl::Disable(gl::SCISSOR_TEST);
            }
        }
    }

    pub(super) fn cmd_set_depth_test(&self, enable: bool) {
        unsafe {
            if enable {
                gl::Enable(gl::DEPTH_TEST);
            } else {
                gl::Disable(gl::DEPTH_TEST);
            }
        }
    }

    pub(super) fn cmd_set_depth_writable(&self, enable: bool) {
        unsafe {
            gl::DepthMask(if enable { gl::TRUE } else { gl::FALSE });
        }
    }

    pub(super) fn cmd_set_wireframe(&self, enable: bool) {
        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, if enable { gl::LINE } else { gl::FILL });
        }
    }

    pub(super) fn cmd_set_line_width(&self, width: f32) {
        unsafe {
            gl::LineWidth(width);
        }
    }

    pub(super) fn cmd_set_point_size(&self, size: f32) {
        unsafe {
            gl::PointSize(size);
        }
    }

    pub(super) fn cmd_bind_shader(&mut self, handle: super::GpuHandle) {
        self.shader_bind_commands_this_frame += 1;
        if handle.0 == self.current_program {
            self.shader_redundant_binds_this_frame += 1;
        } else {
            self.shader_distinct_programs_this_frame += 1;
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

    pub(super) fn cmd_bind_shader_by_resource(
        &mut self,
        id: ResourceId,
        shader_key: Option<String>,
    ) {
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
            self.shader_bind_commands_this_frame += 1;
            if p == self.current_program {
                self.shader_redundant_binds_this_frame += 1;
            } else {
                self.shader_distinct_programs_this_frame += 1;
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

    pub(super) fn cmd_unbind_shader(&mut self) {
        // NOTE: deliberately do NOT invalidate the texture cache here.
        // glUseProgram(0) leaves texture bindings untouched; the cache
        // remains valid across program switches and self-corrects on any
        // real texture change. Previously this wiped the cache on every
        // shader stop (~2k/frame), destroying all reuse.
        self.texture_invalidations_on_shader_unbind_this_frame += 1;
        unsafe {
            gl::UseProgram(0);
        }
        self.current_program = 0;
    }

    pub(super) fn cmd_set_uniform_int(&self, location: i32, value: i32) {
        unsafe {
            gl::Uniform1i(location, value);
        }
    }

    pub(super) fn cmd_set_uniform_int2(&self, location: i32, value: [i32; 2]) {
        unsafe {
            gl::Uniform2i(location, value[0], value[1]);
        }
    }

    pub(super) fn cmd_set_uniform_int3(&self, location: i32, value: [i32; 3]) {
        unsafe {
            gl::Uniform3i(location, value[0], value[1], value[2]);
        }
    }

    pub(super) fn cmd_set_uniform_int4(&self, location: i32, value: [i32; 4]) {
        unsafe {
            gl::Uniform4i(location, value[0], value[1], value[2], value[3]);
        }
    }

    pub(super) fn cmd_set_uniform_float(&self, location: i32, value: f32) {
        unsafe {
            gl::Uniform1f(location, value);
        }
    }

    pub(super) fn cmd_set_uniform_float2(&self, location: i32, value: [f32; 2]) {
        unsafe {
            gl::Uniform2f(location, value[0], value[1]);
        }
    }

    pub(super) fn cmd_set_uniform_float3(&self, location: i32, value: [f32; 3]) {
        unsafe {
            gl::Uniform3f(location, value[0], value[1], value[2]);
        }
    }

    pub(super) fn cmd_set_uniform_float4(&self, location: i32, value: [f32; 4]) {
        unsafe {
            gl::Uniform4f(location, value[0], value[1], value[2], value[3]);
        }
    }

    pub(super) fn cmd_set_uniform_mat4(&self, location: i32, value: [f32; 16]) {
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, value.as_ptr());
        }
    }

    pub(super) fn cmd_set_uniform_int_by_name(&mut self, name: Arc<str>, value: i32) {
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform1i(loc, value);
            }
        }
    }

    pub(super) fn cmd_set_uniform_int2_by_name(&mut self, name: Arc<str>, value: [i32; 2]) {
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform2i(loc, value[0], value[1]);
            }
        }
    }

    pub(super) fn cmd_set_uniform_int3_by_name(&mut self, name: Arc<str>, value: [i32; 3]) {
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform3i(loc, value[0], value[1], value[2]);
            }
        }
    }

    pub(super) fn cmd_set_uniform_int4_by_name(&mut self, name: Arc<str>, value: [i32; 4]) {
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform4i(loc, value[0], value[1], value[2], value[3]);
            }
        }
    }

    pub(super) fn cmd_set_uniform_float_by_name(&mut self, name: Arc<str>, value: f32) {
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform1f(loc, value);
            }
        }
    }

    pub(super) fn cmd_set_uniform_float2_by_name(&mut self, name: Arc<str>, value: [f32; 2]) {
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform2f(loc, value[0], value[1]);
            }
        }
    }

    pub(super) fn cmd_set_uniform_float3_by_name(&mut self, name: Arc<str>, value: [f32; 3]) {
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform3f(loc, value[0], value[1], value[2]);
            }
        }
    }

    pub(super) fn cmd_set_uniform_float4_by_name(&mut self, name: Arc<str>, value: [f32; 4]) {
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::Uniform4f(loc, value[0], value[1], value[2], value[3]);
            }
        }
    }

    pub(super) fn cmd_set_uniform_mat4_by_name(&mut self, name: Arc<str>, value: [f32; 16]) {
        let loc = self.get_uniform_location_cached(&name);
        if loc >= 0 {
            unsafe {
                gl::UniformMatrix4fv(loc, 1, gl::FALSE, value.as_ptr());
            }
        }
    }

    pub(super) fn cmd_set_uniform_mat4_by_generic_name(
        &mut self,
        name: GenericUniformName,
        value: [f32; 16],
    ) {
        let loc = self.get_uniform_location_cached(name.as_str());
        if loc >= 0 {
            unsafe {
                gl::UniformMatrix4fv(loc, 1, gl::FALSE, value.as_ptr());
            }
        }
    }

    pub(super) fn cmd_bind_texture_2d(&mut self, slot: u32, handle: super::GpuHandle) {
        self.bind_texture_cached(slot, handle.0, TextureType::Texture2D);
    }

    pub(super) fn cmd_bind_texture_2d_by_resource(&mut self, slot: u32, id: ResourceId) {
        if let Some(GpuResource::Texture2D { handle }) = self.resources.get(&id) {
            self.bind_texture_cached(slot, *handle, TextureType::Texture2D);
        } else {
            warn!("BindTexture2DByResource: resource {:?} not found", id);
        }
    }

    pub(super) fn cmd_bind_texture_1d_by_resource(&mut self, slot: u32, id: ResourceId) {
        if let Some(GpuResource::Texture1D { handle }) = self.resources.get(&id) {
            self.bind_texture_cached(slot, *handle, TextureType::Texture1D);
        } else {
            warn!("BindTexture1DByResource: resource {:?} not found", id);
        }
    }

    pub(super) fn cmd_bind_texture_3d(&mut self, slot: u32, handle: super::GpuHandle) {
        self.bind_texture_cached(slot, handle.0, TextureType::Texture3D);
    }

    pub(super) fn cmd_bind_texture_3d_by_resource(&mut self, slot: u32, id: ResourceId) {
        if let Some(GpuResource::Texture3D { handle }) = self.resources.get(&id) {
            self.bind_texture_cached(slot, *handle, TextureType::Texture3D);
        } else {
            warn!("BindTexture3DByResource: resource {:?} not found", id);
        }
    }

    pub(super) fn cmd_bind_texture_cube(&mut self, slot: u32, handle: super::GpuHandle) {
        self.bind_texture_cached(slot, handle.0, TextureType::TextureCube);
    }

    pub(super) fn cmd_bind_texture_cube_by_resource(&mut self, slot: u32, id: ResourceId) {
        if let Some(GpuResource::TextureCube { handle }) = self.resources.get(&id) {
            self.bind_texture_cached(slot, *handle, TextureType::TextureCube);
        } else {
            warn!("BindTextureCubeByResource: resource {:?} not found", id);
        }
    }

    pub(super) fn cmd_unbind_texture(&mut self, slot: u32) {
        self.unbind_texture_cached(slot);
    }

    pub(super) fn cmd_set_texture_2d_mag_filter(
        &self,
        handle: super::GpuHandle,
        filter: TexFilter,
    ) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, handle.0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter as i32);
        }
        self.restore_active_unit_binding();
    }

    pub(super) fn cmd_set_texture_2d_min_filter(
        &self,
        handle: super::GpuHandle,
        filter: TexFilter,
    ) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, handle.0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as i32);
        }
        self.restore_active_unit_binding();
    }

    pub(super) fn cmd_set_texture_2d_wrap_mode(&self, handle: super::GpuHandle, mode: TexWrapMode) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, handle.0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, mode as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, mode as i32);
        }
        self.restore_active_unit_binding();
    }

    pub(super) fn cmd_set_texture_2d_mip_range(
        &self,
        handle: super::GpuHandle,
        min_level: i32,
        max_level: i32,
    ) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, handle.0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, min_level);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, max_level);
        }
        self.restore_active_unit_binding();
    }

    pub(super) fn cmd_generate_mipmap_2d(&self, handle: super::GpuHandle) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, handle.0);
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
        self.restore_active_unit_binding();
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn cmd_update_texture_2d_data(
        &self,
        handle: super::GpuHandle,
        width: i32,
        height: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    ) {
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

    pub(super) fn cmd_set_texture_2d_anisotropy(&self, handle: super::GpuHandle, factor: f32) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, handle.0);
            gl::TexParameterf(gl::TEXTURE_2D, gl::TEXTURE_MAX_ANISOTROPY_EXT, factor);
        }
        self.restore_active_unit_binding();
    }

    pub(super) fn cmd_set_texture_2d_anisotropy_by_resource(
        &mut self,
        id: ResourceId,
        factor: f32,
    ) {
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

    pub(super) fn cmd_set_texture_2d_mip_range_by_resource(
        &mut self,
        id: ResourceId,
        min_level: i32,
        max_level: i32,
    ) {
        if let Some((target, handle)) = self.texture_target_and_handle(id) {
            unsafe {
                gl::BindTexture(target, handle);
                gl::TexParameteri(target, gl::TEXTURE_BASE_LEVEL, min_level);
                gl::TexParameteri(target, gl::TEXTURE_MAX_LEVEL, max_level);
            }
            self.restore_active_unit_binding();
        } else {
            warn!(
                "SetTexture2DMipRangeByResource: resource {:?} not found",
                id
            );
        }
    }

    pub(super) fn cmd_set_texel_1d_by_resource(&mut self, id: ResourceId, x: i32, color: [f32; 4]) {
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

    pub(super) fn cmd_set_texel_2d_by_resource(
        &mut self,
        id: ResourceId,
        x: i32,
        y: i32,
        color: [f32; 4],
    ) {
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

    pub(super) fn cmd_set_texture_mag_filter_by_resource(
        &mut self,
        id: ResourceId,
        filter: TexFilter,
    ) {
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

    pub(super) fn cmd_set_texture_min_filter_by_resource(
        &mut self,
        id: ResourceId,
        filter: TexFilter,
    ) {
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

    pub(super) fn cmd_set_texture_wrap_mode_by_resource(
        &mut self,
        id: ResourceId,
        mode: TexWrapMode,
    ) {
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

    pub(super) fn cmd_generate_mipmap_by_resource(&mut self, id: ResourceId) {
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

    pub(super) fn cmd_update_texture_1d_data_by_resource(
        &mut self,
        id: ResourceId,
        width: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    ) {
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
                gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);
            }
        } else {
            warn!(
                "UpdateTextureCubeFaceDataByResource: resource {:?} not found",
                id
            );
        }
    }

    pub(super) fn cmd_copy_texture_2d_from_framebuffer_by_resource(
        &mut self,
        id: ResourceId,
        internal_format: i32,
        width: i32,
        height: i32,
    ) {
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

    pub(super) fn cmd_read_texture_1d_data(
        &mut self,
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
        reply_tx: crossbeam::channel::Sender<Vec<u8>>,
    ) {
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
        let _ = reply_tx.send(data);
    }

    pub(super) fn cmd_read_texture_2d_data(
        &mut self,
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
        reply_tx: crossbeam::channel::Sender<Vec<u8>>,
    ) {
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
        let _ = reply_tx.send(data);
    }

    pub(super) fn cmd_read_texture_3d_data(
        &mut self,
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
        reply_tx: crossbeam::channel::Sender<Vec<u8>>,
    ) {
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
        let _ = reply_tx.send(data);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn cmd_read_texture_cube_face_data(
        &mut self,
        id: ResourceId,
        face: u32,
        level: i32,
        pixel_format: u32,
        data_format: u32,
        reply_tx: crossbeam::channel::Sender<Vec<u8>>,
    ) {
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
        let _ = reply_tx.send(data);
    }

    pub(super) fn cmd_sample_pixel_2d_by_resource(
        &mut self,
        id: ResourceId,
        x: i32,
        y: i32,
        reply_tx: crossbeam::channel::Sender<[u8; 4]>,
    ) {
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
                // Restore whatever framebuffer the FBO stack says should be bound.
                gl::BindFramebuffer(
                    gl::FRAMEBUFFER,
                    self.fbo_stack.last().map_or(0, |fbo| fbo.handle),
                );
                gl::DeleteFramebuffers(1, &fbo);
            }
        } else {
            warn!("SamplePixel2DByResource: resource {:?} not found", id);
        }
        let _ = reply_tx.send(pixel);
    }

    pub(super) fn cmd_read_framebuffer_pixels(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        reply_tx: crossbeam::channel::Sender<Vec<u8>>,
    ) {
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
        let _ = reply_tx.send(data);
    }

    pub(super) fn cmd_framebuffer_attach_texture_2d(
        &mut self,
        attachment: u32,
        texture: super::GpuHandle,
        level: i32,
    ) {
        unsafe {
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                attachment,
                gl::TEXTURE_2D,
                texture.0,
                level,
            );
            // Update color index if this is a color attachment
            if (gl::COLOR_ATTACHMENT0..=gl::COLOR_ATTACHMENT3).contains(&attachment) {
                if let Some(fbo) = self.fbo_stack.last_mut() {
                    fbo.color_index = (attachment - gl::COLOR_ATTACHMENT0 + 1) as i32;
                    gl::DrawBuffers(fbo.color_index, DRAW_BUFS.as_ptr());
                }
            }
        }
    }

    pub(super) fn cmd_framebuffer_attach_texture_2d_by_resource(
        &mut self,
        attachment: u32,
        id: ResourceId,
        level: i32,
    ) {
        if let Some(GpuResource::Texture2D { handle }) = self.resources.get(&id) {
            unsafe {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    attachment,
                    gl::TEXTURE_2D,
                    *handle,
                    level,
                );
                // Update color index if this is a color attachment
                if (gl::COLOR_ATTACHMENT0..=gl::COLOR_ATTACHMENT3).contains(&attachment) {
                    if let Some(fbo) = self.fbo_stack.last_mut() {
                        fbo.color_index = (attachment - gl::COLOR_ATTACHMENT0 + 1) as i32;
                        gl::DrawBuffers(fbo.color_index, DRAW_BUFS.as_ptr());
                    }
                }
            }
        } else {
            warn!(
                "FramebufferAttachTexture2DByResource: resource {:?} not found",
                id
            );
        }
    }

    pub(super) fn cmd_framebuffer_attach_texture_3d(
        &mut self,
        attachment: u32,
        texture: super::GpuHandle,
        layer: i32,
        level: i32,
    ) {
        unsafe {
            gl::FramebufferTexture3D(
                gl::FRAMEBUFFER,
                attachment,
                gl::TEXTURE_3D,
                texture.0,
                level,
                layer,
            );
            if let Some(fbo) = self.fbo_stack.last_mut() {
                fbo.color_index = (attachment - gl::COLOR_ATTACHMENT0 + 1) as i32;
                gl::DrawBuffers(fbo.color_index, DRAW_BUFS.as_ptr());
            }
        }
    }

    pub(super) fn cmd_framebuffer_attach_texture_3d_by_resource(
        &mut self,
        attachment: u32,
        id: ResourceId,
        layer: i32,
        level: i32,
    ) {
        if let Some(GpuResource::Texture3D { handle }) = self.resources.get(&id) {
            unsafe {
                gl::FramebufferTexture3D(
                    gl::FRAMEBUFFER,
                    attachment,
                    gl::TEXTURE_3D,
                    *handle,
                    level,
                    layer,
                );
                if let Some(fbo) = self.fbo_stack.last_mut() {
                    fbo.color_index = (attachment - gl::COLOR_ATTACHMENT0 + 1) as i32;
                    gl::DrawBuffers(fbo.color_index, DRAW_BUFS.as_ptr());
                }
            }
        } else {
            warn!(
                "FramebufferAttachTexture3DByResource: resource {:?} not found",
                id
            );
        }
    }

    pub(super) fn cmd_framebuffer_attach_texture_cube(
        &mut self,
        attachment: u32,
        texture: super::GpuHandle,
        face: u32,
        level: i32,
    ) {
        unsafe {
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachment, face, texture.0, level);
            if let Some(fbo) = self.fbo_stack.last_mut() {
                fbo.color_index = (attachment - gl::COLOR_ATTACHMENT0 + 1) as i32;
                gl::DrawBuffers(fbo.color_index, DRAW_BUFS.as_ptr());
            }
        }
    }

    pub(super) fn cmd_framebuffer_attach_texture_cube_by_resource(
        &mut self,
        attachment: u32,
        id: ResourceId,
        face: u32,
        level: i32,
    ) {
        if let Some(GpuResource::TextureCube { handle }) = self.resources.get(&id) {
            unsafe {
                gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachment, face, *handle, level);
                if let Some(fbo) = self.fbo_stack.last_mut() {
                    fbo.color_index = (attachment - gl::COLOR_ATTACHMENT0 + 1) as i32;
                    gl::DrawBuffers(fbo.color_index, DRAW_BUFS.as_ptr());
                }
            }
        } else {
            warn!(
                "FramebufferAttachTextureCubeByResource: resource {:?} not found",
                id
            );
        }
    }

    pub(super) fn cmd_set_draw_buffers(&self, count: i32) {
        unsafe {
            gl::DrawBuffers(count, DRAW_BUFS.as_ptr());
        }
    }

    pub(super) fn cmd_bind_framebuffer(&self, handle: super::GpuHandle) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, handle.0);
        }
    }

    pub(super) fn cmd_bind_default_framebuffer(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub(super) fn cmd_clear(&self, color: Option<[f32; 4]>, depth: Option<f32>) {
        unsafe {
            let mut mask = 0;
            if let Some([r, g, b, a]) = color {
                gl::ClearColor(r, g, b, a);
                mask |= gl::COLOR_BUFFER_BIT;
            }
            if let Some(d) = depth {
                gl::ClearDepth(d as f64);
                mask |= gl::DEPTH_BUFFER_BIT;
            }
            if mask != 0 {
                gl::Clear(mask);
            }
        }
    }

    pub(super) fn cmd_bind_mesh(&self, vao: super::GpuHandle) {
        unsafe {
            gl::BindVertexArray(vao.0);
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::EnableVertexAttribArray(2);
        }
    }

    pub(super) fn cmd_unbind_mesh(&self) {
        unsafe {
            gl::DisableVertexAttribArray(0);
            gl::DisableVertexAttribArray(1);
            gl::DisableVertexAttribArray(2);
            gl::BindVertexArray(0);
        }
    }

    pub(super) fn cmd_draw_mesh(
        &mut self,
        vao: super::GpuHandle,
        index_count: i32,
        primitive: CmdPrimitiveType,
    ) {
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
        self.vertices_drawn_this_frame += index_count.max(0) as u64;
    }

    pub(super) fn cmd_draw_mesh_instanced(
        &mut self,
        vao: super::GpuHandle,
        index_count: i32,
        instance_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        unsafe {
            gl::BindVertexArray(vao.0);
            gl::DrawElementsInstanced(
                primitive.to_gl(),
                index_count,
                gl::UNSIGNED_INT,
                std::ptr::null(),
                instance_count,
            );
            gl::BindVertexArray(0);
        }
        self.vertices_drawn_this_frame +=
            (index_count.max(0) as u64) * (instance_count.max(0) as u64);
    }

    pub(super) fn cmd_draw_mesh_by_resource(
        &mut self,
        id: ResourceId,
        index_count: i32,
        primitive: CmdPrimitiveType,
    ) {
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
            self.vertices_drawn_this_frame += index_count.max(0) as u64;
        } else {
            warn!("DrawMeshByResource: resource {id:?} not found");
        }
    }

    pub(super) fn cmd_draw_mesh_instanced_by_resource(
        &mut self,
        id: ResourceId,
        index_count: i32,
        instance_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        if let Some(GpuResource::Mesh { vao, .. }) = self.resources.get(&id) {
            unsafe {
                gl::BindVertexArray(*vao);
                gl::DrawElementsInstanced(
                    primitive.to_gl(),
                    index_count,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                    instance_count,
                );
                gl::BindVertexArray(0);
            }
            self.vertices_drawn_this_frame +=
                (index_count.max(0) as u64) * (instance_count.max(0) as u64);
        } else {
            warn!("DrawMeshInstancedByResource: resource {id:?} not found");
        }
    }

    pub(super) fn cmd_draw_instanced_with_data(
        &mut self,
        mesh_id: ResourceId,
        index_count: i32,
        instances: Vec<InstanceData>,
        primitive: CmdPrimitiveType,
    ) {
        if instances.is_empty() {
            return; // Nothing to draw
        }

        let mesh_vao = if let Some(GpuResource::Mesh { vao, .. }) = self.resources.get(&mesh_id) {
            *vao
        } else {
            warn!(
                "DrawInstancedWithData: mesh resource {:?} not found",
                mesh_id
            );
            return;
        };

        unsafe {
            // Create or resize instance VBO if needed
            let instance_count = instances.len();
            let instance_size = std::mem::size_of::<InstanceData>();
            let data_size = instance_count * instance_size;

            if self.instance_vbo == 0 {
                gl::GenBuffers(1, &mut self.instance_vbo);
            }

            // Bind mesh VAO first
            gl::BindVertexArray(mesh_vao);

            // Bind instance VBO once - used for resize, upload, AND attribute setup
            // (GL_ARRAY_BUFFER is NOT part of VAO state, so this stays bound)
            gl::BindBuffer(gl::ARRAY_BUFFER, self.instance_vbo);

            // Resize buffer if needed (grow only, with some headroom)
            if instance_count > self.instance_vbo_capacity {
                let new_capacity = (instance_count * 3 / 2).max(64); // 50% headroom, min 64
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (new_capacity * instance_size) as isize,
                    std::ptr::null(),
                    gl::DYNAMIC_DRAW,
                );
                self.instance_vbo_capacity = new_capacity;
            }

            // Upload instance data
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                data_size as isize,
                instances.as_ptr() as *const _,
            );

            // Set up instance attributes (model matrix as 4 vec4 columns + color + scale)
            // InstanceData layout: model_matrix[16] + color[4] + scale = 84 bytes
            let stride = instance_size as i32;

            // Attribute 4-7: model matrix columns (mat4 = 4 x vec4)
            for col in 0..4u32 {
                let attrib = 4 + col;
                gl::EnableVertexAttribArray(attrib);
                gl::VertexAttribPointer(
                    attrib,
                    4, // 4 floats per column
                    gl::FLOAT,
                    gl::FALSE,
                    stride,
                    (col as usize * 16) as *const _, // offset: col * 4 floats * 4 bytes
                );
                gl::VertexAttribDivisor(attrib, 1); // Per-instance
            }

            // Attribute 8: color (vec4)
            gl::EnableVertexAttribArray(8);
            gl::VertexAttribPointer(
                8,
                4, // 4 floats (RGBA)
                gl::FLOAT,
                gl::FALSE,
                stride,
                64 as *const _, // offset: 16 floats * 4 bytes = 64
            );
            gl::VertexAttribDivisor(8, 1); // Per-instance

            // Attribute 9: per-instance scale (float)
            gl::EnableVertexAttribArray(9);
            gl::VertexAttribPointer(
                9,
                1, // 1 float (scale)
                gl::FLOAT,
                gl::FALSE,
                stride,
                80 as *const _, // offset: 20 floats * 4 bytes = 80
            );
            gl::VertexAttribDivisor(9, 1); // Per-instance

            // Draw instanced
            gl::DrawElementsInstanced(
                primitive.to_gl(),
                index_count,
                gl::UNSIGNED_INT,
                std::ptr::null(),
                instance_count as i32,
            );

            // Disable instance attributes and reset divisors
            for attrib in 4..=9 {
                gl::VertexAttribDivisor(attrib, 0);
                gl::DisableVertexAttribArray(attrib);
            }

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
        // Note: draw call counting is handled by is_draw_call() in execute()
        self.instanced_data_items_this_frame += instances.len() as u64;
        self.vertices_drawn_this_frame +=
            (index_count.max(0) as u64) * (instances.len() as u64);
    }

    /// Texture-fetch instancing: per-instance attribute is a u32 INDEX into
    /// a static data texture (uploaded once by the producer); the vertex
    /// shader pulls the transform via texelFetch. Upload per frame is
    /// 4 bytes/instance instead of an 84-byte InstanceData - this is what
    /// lets the producer scale to 100k+ asteroids on GL 3.3.
    pub(super) fn cmd_draw_instanced_indices(
        &mut self,
        mesh_id: ResourceId,
        index_count: i32,
        indices: Vec<u32>,
        primitive: CmdPrimitiveType,
    ) {
        if indices.is_empty() {
            return; // Nothing to draw
        }

        let mesh_vao = if let Some(GpuResource::Mesh { vao, .. }) = self.resources.get(&mesh_id) {
            *vao
        } else {
            warn!(
                "DrawInstancedIndices: mesh resource {:?} not found",
                mesh_id
            );
            return;
        };

        unsafe {
            // Reuse the instance VBO (it's just a buffer; the attribute
            // layout below reinterprets it as u32 indices)
            let instance_count = indices.len();
            let data_size = instance_count * std::mem::size_of::<u32>();

            if self.instance_vbo == 0 {
                gl::GenBuffers(1, &mut self.instance_vbo);
            }

            gl::BindVertexArray(mesh_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.instance_vbo);

            // Grow-only capacity (u32 elements)
            if instance_count > self.instance_vbo_capacity_u32 {
                let new_capacity = (instance_count * 3 / 2).max(64);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (new_capacity * std::mem::size_of::<u32>()) as isize,
                    std::ptr::null(),
                    gl::DYNAMIC_DRAW,
                );
                self.instance_vbo_capacity_u32 = new_capacity;
            }

            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                data_size as isize,
                indices.as_ptr() as *const _,
            );

            // Attribute 10: instance index (uint, divisor 1). MUST use
            // VertexAttribIPointer for integer attributes - the float
            // pointer path converts bits to floats and the shader's
            // `in uint` reads garbage -> out-of-bounds texelFetch.
            const ATTRIB: u32 = 10;
            gl::EnableVertexAttribArray(ATTRIB);
            gl::VertexAttribIPointer(
                ATTRIB,
                1,
                gl::UNSIGNED_INT,
                0, // tightly packed
                std::ptr::null(),
            );
            gl::VertexAttribDivisor(ATTRIB, 1);

            gl::DrawElementsInstanced(
                primitive.to_gl(),
                index_count,
                gl::UNSIGNED_INT,
                std::ptr::null(),
                instance_count as i32,
            );

            gl::VertexAttribDivisor(ATTRIB, 0);
            gl::DisableVertexAttribArray(ATTRIB);

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
        // Note: draw call counting is handled by is_draw_call() in execute()
        self.instanced_data_items_this_frame += indices.len() as u64;
        self.vertices_drawn_this_frame +=
            (index_count.max(0) as u64) * (indices.len() as u64);
    }

    pub(super) fn cmd_bind_mesh_by_resource(&mut self, id: ResourceId) {
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

    pub(super) fn cmd_create_shader(
        &mut self,
        id: ResourceId,
        vertex_src: String,
        fragment_src: String,
        reply_tx: crossbeam::channel::Sender<Option<String>>,
    ) {
        let error = match self.create_shader(&vertex_src, &fragment_src) {
            Ok(program) => {
                self.resources.insert(id, GpuResource::Shader { program });
                None
            }
            Err(e) => {
                error!("Failed to create shader {:?}: {}", id, e);
                Some(e)
            }
        };
        let _ = reply_tx.send(error);
    }

    pub(super) fn cmd_get_uniform_location_by_resource(
        &mut self,
        id: ResourceId,
        name: Arc<str>,
        reply_tx: crossbeam::channel::Sender<i32>,
    ) {
        let loc = if let Some(GpuResource::Shader { program }) = self.resources.get(&id) {
            let program = *program;
            self.get_uniform_location_for_program(program, &name)
        } else {
            warn!("GetUniformLocationByResource: resource {:?} not found", id);
            -1
        };
        let _ = reply_tx.send(loc);
    }

    pub(super) fn cmd_create_texture_1d(
        &mut self,
        id: ResourceId,
        width: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    ) {
        let handle = self.create_texture_1d(width, format, data.as_deref());
        self.resources.insert(id, GpuResource::Texture1D { handle });
    }

    pub(super) fn cmd_create_texture_2d(
        &mut self,
        id: ResourceId,
        width: u32,
        height: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    ) {
        let handle = self.create_texture_2d(width, height, format, data.as_deref());
        self.resources.insert(id, GpuResource::Texture2D { handle });
    }

    pub(super) fn cmd_create_texture_3d(
        &mut self,
        id: ResourceId,
        width: u32,
        height: u32,
        depth: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    ) {
        let handle = self.create_texture_3d(width, height, depth, format, data.as_deref());
        self.resources.insert(id, GpuResource::Texture3D { handle });
    }

    pub(super) fn cmd_create_texture_cube(&mut self, id: ResourceId, size: u32, format: TexFormat) {
        let handle = self.create_texture_cube(size, format);
        self.resources
            .insert(id, GpuResource::TextureCube { handle });
    }

    pub(super) fn cmd_create_mesh(
        &mut self,
        id: ResourceId,
        vertices: Vec<u8>,
        indices: Vec<u32>,
        vertex_format: VertexFormat,
    ) {
        let (vao, vbo, ebo) = self.create_mesh(&vertices, &indices, &vertex_format);
        self.resources
            .insert(id, GpuResource::Mesh { vao, vbo, ebo });
    }

    pub(super) fn cmd_destroy_resource(&mut self, ids: &[ResourceId]) {
        for id in ids {
            if let Some(resource) = self.resources.remove(id) {
                self.destroy_resource(resource);
            }
        }
    }

    pub(super) fn cmd_resize(&self, width: u32, height: u32) {
        // Only update the viewport - surface resize is handled by the window system.
        // Note: Calling ctx.resize() here was causing freezes during window resize,
        // likely due to synchronization issues with the window manager.
        // The viewport update is sufficient for correct rendering.
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }

    pub(super) fn cmd_reload_shader(
        &mut self,
        shader_key: &str,
        vertex_src: &str,
        fragment_src: &str,
    ) -> CommandReply {
        // Compile shader on render thread and send result back
        let result = match self.create_shader(vertex_src, fragment_src) {
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
                    success: true,
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
                    success: false,
                    error: Some(e),
                    program: 0,
                }
            }
        };
        CommandReply::ShaderReload(result)
    }

    pub(super) fn cmd_swap_buffers(&mut self) -> CommandReply {
        // Calculate frame time before swap
        let frame_time = self.frame_start.elapsed();
        let frame_time_us = frame_time.as_micros() as u64;

        self.stats.frame_count += 1;

        // Snapshot stats at end of frame; readable via `stats_snapshot()`
        // at any later point, and handed back as a reply so the
        // threaded backend can forward it to the main thread.
        self.last_stats = RenderStats {
            commands_processed: self.stats.commands_processed,
            draw_calls: self.stats.draw_calls,
            state_changes: self.stats.state_changes,
            frame_count: self.stats.frame_count,
            last_frame_time_us: frame_time_us,
            commands_last_frame: self.commands_this_frame,
            draw_calls_last_frame: self.draw_calls_this_frame,
            state_changes_last_frame: self.state_changes_this_frame,
            present_wait_us: 0,
            texture_bind_calls_last_frame: self.texture_bind_calls_this_frame,
            texture_binds_skipped_last_frame: self.texture_binds_skipped_this_frame,
            texture_cache_invalidations_last_frame: self.texture_cache_invalidations_this_frame,
            texture_invalidations_on_shader_bind_last_frame:
                self.texture_invalidations_on_shader_bind_this_frame,
            texture_invalidations_on_shader_unbind_last_frame:
                self.texture_invalidations_on_shader_unbind_this_frame,
            draw_mesh_calls_last_frame: self.draw_mesh_calls_this_frame,
            draw_immediate_calls_last_frame: self.draw_immediate_calls_this_frame,
            draw_instanced_calls_last_frame: self.draw_instanced_calls_this_frame,
            immediate_vertices_last_frame: self.immediate_vertices_this_frame,
            instanced_data_items_last_frame: self.instanced_data_items_this_frame,
            vertices_drawn_last_frame: self.vertices_drawn_this_frame,
            uniform_cache_hits_last_frame: self.uniform_cache_hits_this_frame,
            uniform_cache_misses_last_frame: self.uniform_cache_misses_this_frame,
            category_counts_last_frame: self.category_counts_this_frame,
            category_time_us_last_frame: self.category_time_us_this_frame,
            recv_wait_us_last_frame: self.recv_wait_us_this_frame,
            recv_wait_count_last_frame: self.recv_wait_count_this_frame,
            shader_bind_commands_last_frame: self.shader_bind_commands_this_frame,
            shader_redundant_binds_last_frame: self.shader_redundant_binds_this_frame,
            shader_distinct_programs_last_frame: self.shader_distinct_programs_this_frame,
            texture_binds_skipped: self.texture_binds_skipped,
        };

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
        self.commands_this_frame = 0;
        self.draw_calls_this_frame = 0;
        self.state_changes_this_frame = 0;
        self.texture_bind_calls_this_frame = 0;
        self.texture_binds_skipped_this_frame = 0;
        self.texture_cache_invalidations_this_frame = 0;
        self.texture_invalidations_on_shader_bind_this_frame = 0;
        self.texture_invalidations_on_shader_unbind_this_frame = 0;
        self.draw_mesh_calls_this_frame = 0;
        self.draw_immediate_calls_this_frame = 0;
        self.draw_instanced_calls_this_frame = 0;
        self.immediate_vertices_this_frame = 0;
        self.instanced_data_items_this_frame = 0;
        self.vertices_drawn_this_frame = 0;
        self.uniform_cache_hits_this_frame = 0;
        self.uniform_cache_misses_this_frame = 0;
        self.category_counts_this_frame = [0; 12];
        self.category_time_us_this_frame = [0; 12];
        self.recv_wait_us_this_frame = 0;
        self.recv_wait_count_this_frame = 0;
        self.shader_redundant_binds_this_frame = 0;
        self.shader_distinct_programs_this_frame = 0;
        self.shader_bind_commands_this_frame = 0;
        self.frame_start = std::time::Instant::now();

        CommandReply::Stats(self.last_stats.clone())
    }

    pub(super) fn cmd_flush(&self) {
        unsafe {
            gl::Finish();
        }
    }

    pub(super) fn cmd_fence(&self, fence_id: u64) -> CommandReply {
        CommandReply::Fence(fence_id)
    }

    pub(super) fn cmd_pacing_fence(&self, fence_id: u64) -> CommandReply {
        CommandReply::PacingFence(fence_id)
    }

    pub(super) fn cmd_set_blend_mode(&self, mode: BlendMode) {
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

    pub(super) fn cmd_set_cull_face(&self, face: CullFace) {
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

    pub(super) fn cmd_draw_immediate(&mut self, primitive: CmdPrimitiveType, vertices: &[ImmVertex]) {
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
        self.vertices_drawn_this_frame += vertices.len() as u64;
    }

    pub(super) fn create_shader(
        &self,
        vertex_src: &str,
        fragment_src: &str,
    ) -> Result<u32, String> {
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
            // Per-instance attributes for DrawInstancedWithData (res/shader/include/instanced.glsl);
            // must match cmd_draw_instanced_with_data's VertexAttribPointer setup: 4-7=mWorld
            // columns, 8=color. No-op (harmless) for shaders that don't declare these names.
            gl::BindAttribLocation(program, 4, c"instance_matrix_col0".as_ptr() as *const _);
            gl::BindAttribLocation(program, 5, c"instance_matrix_col1".as_ptr() as *const _);
            gl::BindAttribLocation(program, 6, c"instance_matrix_col2".as_ptr() as *const _);
            gl::BindAttribLocation(program, 7, c"instance_matrix_col3".as_ptr() as *const _);
            gl::BindAttribLocation(program, 8, c"instance_color".as_ptr() as *const _);

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

            // Bind CameraUBO to binding point 0 (if present in shader)
            let block_index = gl::GetUniformBlockIndex(program, c"CameraUBO".as_ptr() as *const _);
            if block_index != gl::INVALID_INDEX {
                gl::UniformBlockBinding(program, block_index, 0);
            }

            // Bind LightUBO to binding point 2 (if present in shader)
            let light_block_index =
                gl::GetUniformBlockIndex(program, c"LightUBO".as_ptr() as *const _);
            if light_block_index != gl::INVALID_INDEX {
                gl::UniformBlockBinding(program, light_block_index, 2);
            }

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            Ok(program)
        }
    }

    pub(super) fn create_texture_2d(
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

    pub(super) fn create_texture_1d(
        &self,
        width: u32,
        format: TexFormat,
        data: Option<&[u8]>,
    ) -> u32 {
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

    pub(super) fn create_texture_3d(
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
    pub(super) fn create_texture_cube(&self, size: u32, format: TexFormat) -> u32 {
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
    pub(super) fn texel_buffer_size(
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
    pub(super) fn texture_target_and_handle(
        &self,
        id: ResourceId,
    ) -> Option<(gl::types::GLenum, u32)> {
        match self.resources.get(&id) {
            Some(GpuResource::Texture1D { handle }) => Some((gl::TEXTURE_1D, *handle)),
            Some(GpuResource::Texture2D { handle }) => Some((gl::TEXTURE_2D, *handle)),
            Some(GpuResource::Texture3D { handle }) => Some((gl::TEXTURE_3D, *handle)),
            Some(GpuResource::TextureCube { handle }) => Some((gl::TEXTURE_CUBE_MAP, *handle)),
            _ => None,
        }
    }

    pub(super) fn create_mesh(
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

    pub(super) fn destroy_resource(&self, resource: GpuResource) {
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
                GpuResource::Framebuffer { fbo } => {
                    gl::DeleteFramebuffers(1, &fbo);
                }
            }
        }
    }

    /// Create the camera UBO (binding point 0)
    pub(super) fn cmd_create_camera_ubo(&mut self) {
        if self.camera_ubo != 0 {
            return; // Already created
        }

        unsafe {
            gl::GenBuffers(1, &mut self.camera_ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.camera_ubo);
            // Allocate 288 bytes (CameraUboData::SIZE)
            gl::BufferData(gl::UNIFORM_BUFFER, 288, std::ptr::null(), gl::DYNAMIC_DRAW);
            // Bind to binding point 0
            gl::BindBufferBase(gl::UNIFORM_BUFFER, 0, self.camera_ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
        debug!("Created camera UBO with handle {}", self.camera_ubo);
    }

    /// Update camera UBO data
    pub(super) fn cmd_update_camera_ubo(&mut self, data: &[u8; 288]) {
        if self.camera_ubo == 0 {
            self.cmd_create_camera_ubo();
        }

        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.camera_ubo);
            gl::BufferSubData(gl::UNIFORM_BUFFER, 0, 288, data.as_ptr() as *const _);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }

    /// Create material UBO
    pub(super) fn cmd_create_material_ubo(&mut self) {
        if self.material_ubo != 0 {
            return; // Already created
        }

        unsafe {
            gl::GenBuffers(1, &mut self.material_ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.material_ubo);
            // Allocate 32 bytes (MaterialUboData::SIZE)
            gl::BufferData(gl::UNIFORM_BUFFER, 32, std::ptr::null(), gl::DYNAMIC_DRAW);
            // Bind to binding point 1
            gl::BindBufferBase(gl::UNIFORM_BUFFER, 1, self.material_ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
        debug!("Created material UBO with handle {}", self.material_ubo);
    }

    /// Update material UBO data
    pub(super) fn cmd_update_material_ubo(&mut self, data: &[u8; 32]) {
        if self.material_ubo == 0 {
            self.cmd_create_material_ubo();
        }

        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.material_ubo);
            gl::BufferSubData(gl::UNIFORM_BUFFER, 0, 32, data.as_ptr() as *const _);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }

    /// Create light UBO
    pub(super) fn cmd_create_light_ubo(&mut self) {
        if self.light_ubo != 0 {
            return; // Already created
        }

        unsafe {
            gl::GenBuffers(1, &mut self.light_ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.light_ubo);
            // Allocate 32 bytes (LightUboData::SIZE)
            gl::BufferData(gl::UNIFORM_BUFFER, 32, std::ptr::null(), gl::DYNAMIC_DRAW);
            // Bind to binding point 2 (LIGHT_UBO_BINDING)
            gl::BindBufferBase(gl::UNIFORM_BUFFER, 2, self.light_ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
        debug!("Created light UBO with handle {}", self.light_ubo);
    }

    /// Update light UBO data
    pub(super) fn cmd_update_light_ubo(&mut self, data: &[u8; 32]) {
        if self.light_ubo == 0 {
            self.cmd_create_light_ubo();
        }

        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.light_ubo);
            gl::BufferSubData(gl::UNIFORM_BUFFER, 0, 32, data.as_ptr() as *const _);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }

    /// Push a new framebuffer onto the FBO stack
    pub(super) fn cmd_push_framebuffer(&mut self) {
        if self.fbo_stack.len() >= FBO_STACK_DEPTH {
            error!(
                "RenderThread: Maximum FBO stack depth {} exceeded",
                FBO_STACK_DEPTH
            );
            return;
        }

        unsafe {
            let mut handle = 0;
            gl::GenFramebuffers(1, &mut handle);
            gl::BindFramebuffer(gl::FRAMEBUFFER, handle);

            self.fbo_stack.push(FboEntry {
                handle,
                color_index: 0,
            });
        }
    }

    /// Pop the current framebuffer from the FBO stack
    pub(super) fn cmd_pop_framebuffer(&mut self) {
        if self.fbo_stack.is_empty() {
            error!("RenderThread: Attempting to pop an empty FBO stack");
            return;
        }

        unsafe {
            // Detach all color attachments
            for i in 0..4 {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0 + i,
                    gl::TEXTURE_2D,
                    0,
                    0,
                );
            }

            // Detach depth attachment
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, 0, 0);

            // Delete the FBO
            if let Some(fbo) = self.fbo_stack.pop() {
                gl::DeleteFramebuffers(1, &fbo.handle);
            }

            // Bind previous FBO or default framebuffer
            if let Some(prev) = self.fbo_stack.last() {
                gl::BindFramebuffer(gl::FRAMEBUFFER, prev.handle);
            } else {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
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

        // Cleanup any remaining FBOs in the stack
        unsafe {
            let fbo_count = self.fbo_stack.len();
            for fbo in self.fbo_stack.drain(..) {
                gl::DeleteFramebuffers(1, &fbo.handle);
            }
            if fbo_count > 0 {
                info!("Cleaned up {} FBOs from stack", fbo_count);
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
        unsafe {
            gl::Flush();
            gl::Finish();
        }
        info!("GL commands flushed");

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
        let slot_idx = slot as usize;
        if slot_idx >= MAX_TEXTURE_SLOTS {
            // Slot out of range, just bind directly
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + slot);
                gl::BindTexture(tex_type.to_gl_target(), handle);
                gl::ActiveTexture(gl::TEXTURE0);
            }
            self.texture_bind_calls_this_frame += 1;
            return true;
        }

        let new_binding = TextureBinding::new(handle, tex_type);
        let current = &self.texture_bindings[slot_idx];

        // Check if already bound
        if current.handle == handle && current.tex_type == Some(tex_type) {
            self.texture_binds_skipped += 1;
            self.texture_binds_skipped_this_frame += 1;
            return false;
        }

        // Different texture or type - need to bind
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(tex_type.to_gl_target(), handle);
            gl::ActiveTexture(gl::TEXTURE0);
        }

        self.texture_bind_calls_this_frame += 1;
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
                self.texture_binds_skipped_this_frame += 1;
                return;
            }

            // Unbind based on current type
            if let Some(tex_type) = current.tex_type {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0 + slot);
                    gl::BindTexture(tex_type.to_gl_target(), 0);
                    gl::ActiveTexture(gl::TEXTURE0);
                }
                self.texture_bind_calls_this_frame += 1;
            }

            self.texture_bindings[slot_idx] = TextureBinding::unbound();
        } else {
            // Slot out of range, can't track - just unbind 2D as fallback
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + slot);
                gl::BindTexture(gl::TEXTURE_2D, 0);
                gl::ActiveTexture(gl::TEXTURE0);
            }
            self.texture_bind_calls_this_frame += 1;
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
            self.uniform_cache_hits_this_frame += 1;
            return loc;
        }

        self.uniform_cache_misses_this_frame += 1;
        let c_name = std::ffi::CString::new(name).unwrap_or_default();
        let loc = unsafe { gl::GetUniformLocation(program, c_name.as_ptr()) };

        // Store in cache (even if -1 to avoid repeated lookups for non-existent uniforms)
        cache.insert(Arc::from(name), loc);
        loc
    }
}
