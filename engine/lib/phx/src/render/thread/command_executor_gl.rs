#![allow(unsafe_code)]

use std::sync::Arc;

use tracing::{error, info, warn};

use crate::render::gl::{self};
use crate::render::thread::{AttachKey, FboKey, GlPendingReadback, GpuResource};
use crate::render::{
    BlockLayout, CommandCategory, CommandExecutor, CommandReply, LoadOp, MAX_COLOR_ATTACHMENTS,
    ReadSource, ReadbackSlot, RenderPassDesc, RenderStats, ResourceId, ShaderLayout, TexDesc,
    TexDim, TexFormat, TexRegion, VertexFormat, ViewDim,
};
use crate::window::{PresentMode, WindowGlContext};

const DRAW_BUFS: [u32; 4] = [
    gl::COLOR_ATTACHMENT0,
    gl::COLOR_ATTACHMENT1,
    gl::COLOR_ATTACHMENT2,
    gl::COLOR_ATTACHMENT3,
];

impl CommandExecutor {
    /// The `key=value` lines `Renderer::backend_info` returns for the GL
    /// backend. Needs the context current on this thread, after `init_gl`.
    pub fn gl_backend_info(&self) -> String {
        let get = |name: u32| unsafe {
            let p = gl::GetString(name);
            if p.is_null() {
                "n/a".to_string()
            } else {
                std::ffi::CStr::from_ptr(p as *const _).to_string_lossy().into_owned()
            }
        };
        format!(
            "backend=OpenGL 3.3
gl_renderer={}
gl_vendor={}
gl_version={}
glsl_version={}
",
            get(gl::RENDERER),
            get(gl::VENDOR),
            get(gl::VERSION),
            get(gl::SHADING_LANGUAGE_VERSION)
        )
    }

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
    pub(super) fn cmd_copy_texture_2d_from_framebuffer_by_resource(
        &mut self,
        id: ResourceId,
        format: TexFormat,
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
                    format.to_gl_formats().0,
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

    /// Attach layer `layer` of mip `level` of texture `id` to the scratch read
    /// framebuffer (`layer` is the z slice of a 3D texture or the face of a
    /// cube). False if `id` is not a texture.
    fn attach_read_layer(&self, id: ResourceId, level: i32, layer: u32) -> bool {
        unsafe {
            match self.resources.get(&id) {
                Some(GpuResource::Texture1D { handle }) => gl::FramebufferTexture1D(
                    gl::READ_FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_1D,
                    *handle,
                    level,
                ),
                Some(GpuResource::Texture2D { handle }) => gl::FramebufferTexture2D(
                    gl::READ_FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    *handle,
                    level,
                ),
                Some(GpuResource::Texture3D { handle }) => gl::FramebufferTextureLayer(
                    gl::READ_FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    *handle,
                    level,
                    layer as i32,
                ),
                Some(GpuResource::TextureCube { handle }) => gl::FramebufferTexture2D(
                    gl::READ_FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + layer,
                    *handle,
                    level,
                ),
                _ => return false,
            }
        }
        true
    }

    /// `glReadPixels` of `region` of `src` as `format`, through the scratch
    /// read framebuffer (or the default framebuffer for the backbuffer). GL
    /// converts from the texture's own format. Rows come out tightly packed,
    /// row 0 first, one layer after the other, at `dest` (a pointer, or an
    /// offset into the bound pixel pack buffer). Returns false if the source
    /// cannot be read; the pass framebuffer binding is restored either way.
    fn gl_read_region(
        &mut self,
        src: ReadSource,
        region: &TexRegion,
        format: TexFormat,
        dest: *mut u8,
    ) -> bool {
        if TexFormat::is_depth(format) {
            warn!("ReadTexture: depth formats cannot be read back ({format:?})");
            return false;
        }
        let (_, gl_format, gl_type) = format.to_gl_formats();
        let [x, y, z] = region.origin;
        let [w, h, depth] = region.size;
        let layer_bytes = w as usize * h as usize * TexFormat::get_size(format) as usize;
        let mut ok = true;
        unsafe {
            gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
            if self.binding.copy_fbos[0] == 0 {
                gl::GenFramebuffers(2, self.binding.copy_fbos.as_mut_ptr());
            }
            let read_fbo = self.binding.copy_fbos[0];
            match src {
                ReadSource::Backbuffer => {
                    gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
                    gl::ReadBuffer(gl::BACK);
                }
                ReadSource::Texture(_) => {
                    gl::BindFramebuffer(gl::READ_FRAMEBUFFER, read_fbo);
                    gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
                }
            }
            for layer in 0..depth {
                if let ReadSource::Texture(id) = src {
                    if !self.attach_read_layer(id, region.level as i32, z + layer) {
                        warn!("ReadTexture: resource {id:?} is not a texture");
                        ok = false;
                        break;
                    }
                    if layer == 0 {
                        let status = gl::CheckFramebufferStatus(gl::READ_FRAMEBUFFER);
                        if status != gl::FRAMEBUFFER_COMPLETE {
                            warn!(
                                "ReadTexture: {id:?} cannot be read ({format:?}, status {status:#x})"
                            );
                            ok = false;
                            break;
                        }
                    }
                }
                gl::ReadPixels(
                    x as i32,
                    y as i32,
                    w as i32,
                    h as i32,
                    gl_format,
                    gl_type,
                    dest.wrapping_add(layer as usize * layer_bytes) as *mut _,
                );
            }
            if matches!(src, ReadSource::Texture(_)) {
                // Detach, so a deleted texture is not kept alive by the scratch FBO.
                gl::BindFramebuffer(gl::READ_FRAMEBUFFER, read_fbo);
                gl::FramebufferTexture2D(
                    gl::READ_FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    0,
                    0,
                );
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.bound_fbo);
        }
        ok
    }

    /// `ReadTextureSync`: read `region` of `src` as `format` and wait for it.
    /// Empty if the read failed.
    pub(super) fn cmd_read_texture_sync(
        &mut self,
        src: ReadSource,
        region: &TexRegion,
        format: TexFormat,
    ) -> Vec<u8> {
        let _sa = self.record_command(CommandCategory::Readback, false, false);
        let mut data = vec![0u8; region.bytes(format)];
        if data.is_empty() || !self.has_gl_context() {
            return Vec::new();
        }
        unsafe {
            gl::BindBuffer(gl::PIXEL_PACK_BUFFER, 0);
        }
        if !self.gl_read_region(src, region, format, data.as_mut_ptr()) {
            data.clear();
        }
        data
    }

    /// `ReadbackAsync`: `glReadPixels` into a fresh pixel pack buffer (the
    /// driver queues the copy and returns), then a fence behind it.
    /// `poll_readbacks` collects the buffer once the fence has signalled.
    pub(super) fn cmd_readback_async(
        &mut self,
        src: ReadSource,
        region: &TexRegion,
        format: TexFormat,
        slot: Arc<ReadbackSlot>,
    ) {
        let _sa = self.record_command(CommandCategory::Readback, false, false);
        let size = region.bytes(format);
        if size == 0 || !self.has_gl_context() {
            slot.fail();
            return;
        }
        unsafe {
            let mut pbo = 0;
            gl::GenBuffers(1, &mut pbo);
            gl::BindBuffer(gl::PIXEL_PACK_BUFFER, pbo);
            gl::BufferData(
                gl::PIXEL_PACK_BUFFER,
                size as isize,
                std::ptr::null(),
                gl::STREAM_READ,
            );
            let ok = self.gl_read_region(src, region, format, std::ptr::null_mut());
            gl::BindBuffer(gl::PIXEL_PACK_BUFFER, 0);
            if !ok {
                gl::DeleteBuffers(1, &pbo);
                slot.fail();
                return;
            }
            let fence = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
            // The fence only counts once the commands before it were sent.
            gl::Flush();
            self.pending_readbacks.push(GlPendingReadback {
                pbo,
                fence: fence as usize,
                size,
                slot,
            });
        }
    }

    /// Collect the asynchronous readbacks whose fence has signalled. Never
    /// waits: called once per frame at `BeginFrame`.
    pub(super) fn poll_readbacks(&mut self) {
        if self.pending_readbacks.is_empty() || !self.has_gl_context() {
            return;
        }
        let pending = std::mem::take(&mut self.pending_readbacks);
        for read in pending {
            unsafe {
                let sync = read.fence as gl::types::GLsync;
                let status = gl::ClientWaitSync(sync, 0, 0);
                if status == gl::TIMEOUT_EXPIRED {
                    self.pending_readbacks.push(read);
                    continue;
                }
                if status == gl::ALREADY_SIGNALED || status == gl::CONDITION_SATISFIED {
                    gl::BindBuffer(gl::PIXEL_PACK_BUFFER, read.pbo);
                    let ptr = gl::MapBufferRange(
                        gl::PIXEL_PACK_BUFFER,
                        0,
                        read.size as isize,
                        gl::MAP_READ_BIT,
                    );
                    if ptr.is_null() {
                        read.slot.fail();
                    } else {
                        let bytes =
                            std::slice::from_raw_parts(ptr as *const u8, read.size).to_vec();
                        gl::UnmapBuffer(gl::PIXEL_PACK_BUFFER);
                        read.slot.complete(bytes);
                    }
                    gl::BindBuffer(gl::PIXEL_PACK_BUFFER, 0);
                } else {
                    error!("Readback: fence wait failed (status {status:#x})");
                    read.slot.fail();
                }
                gl::DeleteSync(sync);
                gl::DeleteBuffers(1, &read.pbo);
            }
        }
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
        self.this_frame_stats.passes += 1;

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

    /// Create a texture. A GL texture has either one level or the whole mip
    /// chain (`desc.mips > 1` allocates every level, empty, so that
    /// `GenerateMips` and per-level render targets have storage).
    pub(super) fn cmd_create_texture(
        &mut self,
        id: ResourceId,
        desc: &TexDesc,
        data: Option<Vec<u8>>,
    ) {
        let _sa = self.record_command(CommandCategory::Resource, false, false);
        let resource = self.create_texture(desc, data.as_deref());
        self.resources.insert(id, resource);
        self.binding.tex_descs.insert(id, *desc);
    }

    fn create_texture(&self, desc: &TexDesc, data: Option<&[u8]>) -> GpuResource {
        let (internal_format, gl_format, gl_type) = desc.format.to_gl_formats();
        let levels = if desc.mips > 1 { desc.full_chain() } else { 1 };
        let pixels = |level: u32| match (level, data) {
            (0, Some(d)) => d.as_ptr() as *const std::ffi::c_void,
            _ => std::ptr::null(),
        };
        const CUBE_TARGETS: [gl::types::GLenum; 6] = [
            gl::TEXTURE_CUBE_MAP_POSITIVE_X,
            gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
            gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
            gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
            gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
            gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
        ];
        unsafe {
            let mut handle = 0;
            gl::GenTextures(1, &mut handle);
            // Rows of uploads are tightly packed whatever their width.
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            let (target, resource) = match desc.dim {
                TexDim::D1 => (gl::TEXTURE_1D, GpuResource::Texture1D { handle }),
                TexDim::D2 => (gl::TEXTURE_2D, GpuResource::Texture2D { handle }),
                TexDim::D3 => (gl::TEXTURE_3D, GpuResource::Texture3D { handle }),
                TexDim::Cube => (gl::TEXTURE_CUBE_MAP, GpuResource::TextureCube { handle }),
            };
            gl::BindTexture(target, handle);
            for level in 0..levels {
                let [w, h, d] = desc.level_size(level);
                let l = level as i32;
                match desc.dim {
                    TexDim::D1 => gl::TexImage1D(
                        target,
                        l,
                        internal_format as i32,
                        w as i32,
                        0,
                        gl_format,
                        gl_type,
                        pixels(level),
                    ),
                    TexDim::D2 => gl::TexImage2D(
                        target,
                        l,
                        internal_format as i32,
                        w as i32,
                        h as i32,
                        0,
                        gl_format,
                        gl_type,
                        pixels(level),
                    ),
                    TexDim::D3 => gl::TexImage3D(
                        target,
                        l,
                        internal_format as i32,
                        w as i32,
                        h as i32,
                        d as i32,
                        0,
                        gl_format,
                        gl_type,
                        pixels(level),
                    ),
                    TexDim::Cube => {
                        for face in CUBE_TARGETS {
                            gl::TexImage2D(
                                face,
                                l,
                                internal_format as i32,
                                w as i32,
                                h as i32,
                                0,
                                gl_format,
                                gl_type,
                                std::ptr::null(),
                            );
                        }
                    }
                }
            }

            // The texture's own parameters only matter for a bind without a
            // sampler object, which no draw does; keep the defaults the old
            // creation paths set.
            gl::TexParameteri(target, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(target, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(target, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            if desc.dim != TexDim::D1 {
                gl::TexParameteri(target, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            }
            if desc.dim == TexDim::D3 {
                gl::TexParameteri(target, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
            }

            self.restore_active_unit_binding();
            resource
        }
    }

    /// Write `data` (the texture's own format, tightly packed) to a region.
    pub(super) fn cmd_update_texture(&mut self, id: ResourceId, region: &TexRegion, data: Vec<u8>) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        let (Some(desc), Some((target, handle))) = (
            self.binding.tex_descs.get(&id).copied(),
            self.texture_target_and_handle(id),
        ) else {
            warn!("UpdateTexture: resource {:?} not found", id);
            return;
        };
        let (_, gl_format, gl_type) = desc.format.to_gl_formats();
        let [x, y, z] = region.origin.map(|v| v as i32);
        let [w, h, d] = region.size.map(|v| v as i32);
        let level = region.level as i32;
        let expected = region.texels() * TexFormat::get_size(desc.format) as usize;
        if data.len() < expected {
            warn!(
                "UpdateTexture: {:?} needs {expected} bytes for {:?}, got {}",
                id,
                region,
                data.len()
            );
            return;
        }
        let pixels = data.as_ptr() as *const std::ffi::c_void;
        unsafe {
            gl::BindTexture(target, handle);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            match desc.dim {
                TexDim::D1 => gl::TexSubImage1D(target, level, x, w, gl_format, gl_type, pixels),
                TexDim::D2 => {
                    gl::TexSubImage2D(target, level, x, y, w, h, gl_format, gl_type, pixels)
                }
                TexDim::D3 => {
                    gl::TexSubImage3D(target, level, x, y, z, w, h, d, gl_format, gl_type, pixels)
                }
                TexDim::Cube => {
                    let face = gl::TEXTURE_CUBE_MAP_POSITIVE_X + region.origin[2];
                    gl::TexSubImage2D(face, level, x, y, w, h, gl_format, gl_type, pixels)
                }
            }
        }
        // Not `BindTexture(.., 0)`: unit 0 holds the environment cube map.
        self.restore_active_unit_binding();
    }

    pub(super) fn cmd_generate_mips(&mut self, id: ResourceId) {
        let _sa = self.record_command(CommandCategory::TextureData, false, false);
        if let Some((target, handle)) = self.texture_target_and_handle(id) {
            // `glGenerateMipmap` fills the levels between the base and the max
            // level of the texture, which a sampling view narrowed to the one
            // level it binds (`bind_view`): generate with the full range,
            // and let the next view binding set its own range again.
            let full = (0, 1000);
            let have = self.binding.mip_ranges.get(&id).copied().unwrap_or(full);
            unsafe {
                gl::BindTexture(target, handle);
                if have != full {
                    gl::TexParameteri(target, gl::TEXTURE_BASE_LEVEL, full.0);
                    gl::TexParameteri(target, gl::TEXTURE_MAX_LEVEL, full.1);
                    self.binding.mip_ranges.insert(id, full);
                }
                gl::GenerateMipmap(target);
            }
            self.restore_active_unit_binding();
        } else {
            warn!("GenerateMips: resource {:?} not found", id);
        }
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
            pipelines_cached: self.binding.pipelines.len() as u64,
            samplers: self.binding.samplers.len() as u64,
            bind_groups: self.binding.bind_groups.len() as u64,
            textures: self.resources.values().filter(|r| !matches!(r, GpuResource::Shader { .. } | GpuResource::Mesh { .. })).count() as u64,
            meshes: self.resources.values().filter(|r| matches!(r, GpuResource::Mesh { .. })).count() as u64,
            texture_bytes: crate::render::STAT_NA,
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
