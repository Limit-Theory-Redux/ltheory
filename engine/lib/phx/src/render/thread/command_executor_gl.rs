#![allow(unsafe_code)]

use tracing::{debug, error, info, warn};

use crate::render::gl::{self};
use crate::render::thread::{AttachKey, FboKey, GpuResource};
use crate::render::{
    BlockLayout, CommandCategory, CommandExecutor, CommandReply, LoadOp, MAX_COLOR_ATTACHMENTS,
    RenderPassDesc, RenderStats, ResourceId, ShaderLayout, ShaderReloadResult, TexFilter,
    TexFormat, TexWrapMode, VertexFormat, ViewDim,
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

            // The state pipelines start from (see `reset_pass_state`).
            gl::Disable(gl::DEPTH_TEST);
            gl::DepthMask(gl::TRUE);
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
            gl::BindVertexArray(0);
        }

        self.init_fullscreen_quad();
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
            self.note_mip_range(id, min_level, max_level);
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

            // Every uniform must live in a `#group` block or be a sampler: a loose
            // uniform has no place in the binding model (nothing could set it).
            let loose = Self::loose_uniforms(program);
            if !loose.is_empty() {
                gl::DeleteShader(vs);
                gl::DeleteShader(fs);
                gl::DeleteProgram(program);
                return Err(format!(
                    "loose uniform(s) {}: declare them as members of a `#group 2` uniform block (or as samplers in a `#group`)",
                    loose.join(", ")
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

    /// Names of the active uniforms of `program` that are neither in a uniform
    /// block nor samplers.
    fn loose_uniforms(program: u32) -> Vec<String> {
        let mut loose = Vec::new();
        unsafe {
            let mut count = 0;
            gl::GetProgramiv(program, gl::ACTIVE_UNIFORMS, &mut count);
            for i in 0..count.max(0) as u32 {
                let mut name = [0u8; 256];
                let (mut len, mut size, mut ty) = (0, 0, 0);
                gl::GetActiveUniform(
                    program,
                    i,
                    name.len() as i32,
                    &mut len,
                    &mut size,
                    &mut ty,
                    name.as_mut_ptr() as *mut _,
                );
                let name = String::from_utf8_lossy(&name[..len.max(0) as usize]).into_owned();
                if name.starts_with("gl_") {
                    continue;
                }
                let mut block = 0;
                gl::GetActiveUniformsiv(program, 1, &i, gl::UNIFORM_BLOCK_INDEX, &mut block);
                // sampler1D..sampler2DShadow, and the integer/array/rect sampler types
                let sampler = (0x8B5D..=0x8B62).contains(&ty) || (0x8DC0..=0x8DD7).contains(&ty);
                if block == -1 && !sampler {
                    loose.push(name);
                }
            }
        }
        loose
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
}
