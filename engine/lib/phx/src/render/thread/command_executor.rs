use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
#[cfg(feature = "stats-server")]
use std::sync::atomic::Ordering;
#[cfg(feature = "stats-server")]
use std::time::Instant;

use tracing::info;

use crate::render::{
    CommandCategory, RenderCommand, RenderStats, ResourceId, ShaderReloadResult, gl,
};
use crate::window::WindowActiveGlContext;

/// Result a command may hand back to whoever drove the executor.
///
/// The executor owns no channels, so commands that need to answer the caller
/// return the answer instead of sending it. In threaded mode `RenderThread`
/// forwards it over the matching channel; in immediate mode the caller
/// consumes it directly.
#[derive(Debug, Default)]
pub enum CommandReply {
    #[default]
    None,
    Fence(u64),
    /// Reply to `RenderCommand::PacingFence` - kept on its own variant/channel
    /// so it can never be picked up by whichever code is waiting on a plain
    /// `Fence` (see `RenderCommand::PacingFence`'s docs).
    PacingFence(u64),
    ShaderReload(ShaderReloadResult),
    Stats(Box<RenderStats>),
}

/// GPU resource stored on the render thread
#[derive(Debug)]
#[expect(dead_code)]
pub(super) enum GpuResource {
    Shader { program: u32 },
    Texture1D { handle: u32 },
    Texture2D { handle: u32 },
    Texture3D { handle: u32 },
    TextureCube { handle: u32 },
    Mesh { vao: u32, vbo: u32, ebo: u32 },
    Framebuffer { fbo: u32 },
}

/// Statistics from the render thread (local copy)
#[derive(Debug, Clone, Default)]
pub struct ExecutorStats {
    pub commands_processed: u64,
    pub draw_calls: u64,
    pub state_changes: u64,
    pub frame_count: u64,
}

/// FBO entry for the render thread's FBO stack
pub(super) struct FboEntry {
    pub handle: u32,
    pub color_index: i32,
}

pub(super) const FBO_STACK_DEPTH: usize = 16;

/// Maximum number of texture units to track for caching
/// OpenGL requires at least 16, most GPUs support 32+
pub(super) const MAX_TEXTURE_SLOTS: usize = 16;

/// Texture type for binding cache
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TextureType {
    Texture1D,
    Texture2D,
    Texture3D,
    TextureCube,
}

impl TextureType {
    pub(super) fn to_gl_target(self) -> gl::types::GLenum {
        match self {
            TextureType::Texture1D => gl::TEXTURE_1D,
            TextureType::Texture2D => gl::TEXTURE_2D,
            TextureType::Texture3D => gl::TEXTURE_3D,
            TextureType::TextureCube => gl::TEXTURE_CUBE_MAP,
        }
    }
}

/// Cached texture binding state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct TextureBinding {
    /// GL handle (0 = unbound)
    pub handle: u32,
    /// Texture type (only valid if handle != 0)
    pub tex_type: Option<TextureType>,
}

impl TextureBinding {
    pub(super) fn new(handle: u32, tex_type: TextureType) -> Self {
        Self {
            handle,
            tex_type: Some(tex_type),
        }
    }

    pub(super) fn unbound() -> Self {
        Self::default()
    }
}

/// Owns the GL context and every GPU object, and executes `RenderCommand`s
/// against them. Runs on the render thread in command mode, or inline on the
/// main thread in immediate mode - it has no idea which.
pub struct CommandExecutor {
    pub(super) resources: HashMap<ResourceId, GpuResource>,
    /// Hot-reloaded shaders by shader_key (separate from resources for override)
    pub(crate) hot_reloaded_shaders: HashMap<String, u32>,
    pub(super) stats: ExecutorStats,
    /// Snapshot taken at the last `SwapBuffers`, readable at any later point
    /// via `stats_snapshot()`. Also what `SwapBuffers` returns as
    /// `CommandReply::Stats` for the threaded backend to forward.
    pub(super) last_stats: RenderStats,
    // Immediate mode VAO/VBO for DrawImmediate commands
    pub(super) imm_vao: u32,
    pub(super) imm_vbo: u32,
    // FBO stack for push/pop framebuffer operations
    pub(super) fbo_stack: Vec<FboEntry>,
    // GL context for buffer swapping (stored here to allow access during execute)
    pub(super) gl_context: Option<WindowActiveGlContext>,
    // Currently bound shader program (needed for name-based uniform lookups)
    pub(crate) current_program: u32,
    // Frame timing
    pub(super) frame_start: std::time::Instant,
    /// Per-frame counters, grouped in one plain `RenderStats`. Every executed
    /// command bumps its counter here; `SwapBuffers` copies the group into
    /// `last_stats` and resets it for the next frame.
    pub(super) this_frame_stats: RenderStats,
    /// Per-category executor timing flag (dashboard mode). Timing results land
    /// in `this_frame_stats.category_time_us`; only measured while
    /// this is true so normal runs don't pay the clock overhead. Shared with
    /// the main-thread `Renderer` so attaching the stats sink can flip it.
    /// Only read by `record_command` under the `stats-server` feature.
    #[cfg(feature = "stats-server")]
    pub(super) category_timing: Arc<AtomicBool>,
    /// Per-shader cache for uniform locations: program -> (name -> location)
    /// NOT cleared on shader change - preserves locations across shader switches
    /// Uses Arc<str> as key for O(1) cloning from commands
    pub(super) uniform_caches: HashMap<u32, HashMap<Arc<str>, i32>>,
    /// Instance buffer for DrawInstancedWithData (reused across frames)
    pub(super) instance_vbo: u32,
    /// Capacity of instance buffer in instances
    pub(super) instance_vbo_capacity: usize,
    /// Capacity of instance buffer in u32 elements (DrawInstancedIndices)
    pub(super) instance_vbo_capacity_u32: usize,
    /// Texture binding cache: tracks which texture is bound to each slot
    /// Avoids redundant glBindTexture calls
    pub(super) texture_bindings: [TextureBinding; MAX_TEXTURE_SLOTS],
    /// Stats: number of texture binds skipped due to caching
    pub(super) texture_binds_skipped: u64,
    /// Camera UBO handle (0 if not created yet)
    pub(super) camera_ubo: u32,
    /// Material UBO handle (0 if not created yet)
    pub(super) material_ubo: u32,
    /// Light UBO handle (0 if not created yet)
    pub(super) light_ubo: u32,
}

/// RAII guard returned by [`CommandExecutor::record_command`]. Finishes the
/// per-category timing measurement (started when the guard was created) when
/// it drops - i.e. at the end of the `cmd_*` method that created it. Without
/// the `stats-server` feature this is a zero-sized no-op the compiler
/// removes entirely, so `cmd_*` methods carry no added cost in normal builds.
#[cfg(feature = "stats-server")]
pub(super) struct StatsAggregator {
    executor: *mut CommandExecutor,
    category: CommandCategory,
    start: Option<Instant>,
}

#[cfg(feature = "stats-server")]
impl Drop for StatsAggregator {
    fn drop(&mut self) {
        #[allow(unsafe_code)]
        if let Some(start) = self.start {
            // SAFETY: `executor` was cast from the `&mut CommandExecutor`
            // that produced this guard in `record_command`, and that
            // `CommandExecutor` always outlives the guard (the guard is a
            // local dropped before the enclosing `cmd_*` method - and thus
            // the whole call - returns). Deliberately a raw pointer rather
            // than a borrowed `&'a mut CommandExecutor`: the latter would
            // make the borrow checker treat the guard as holding `self`
            // exclusively for its entire lifetime, which would reject the
            // `cmd_*` method's own further use of `self` after creating the
            // guard. Single-threaded, never re-entrant, and nothing else
            // dereferences this specific pointer while the guard is alive.
            let executor = unsafe { &mut *self.executor };
            let elapsed_us = start.elapsed().as_micros() as u64;
            executor.this_frame_stats.category_time_us[self.category.index()] += elapsed_us;
        }
    }
}

#[cfg(not(feature = "stats-server"))]
pub(super) struct StatsAggregator;

impl CommandExecutor {
    pub fn new(gl_context: Option<WindowActiveGlContext>) -> Self {
        Self::new_with_timing(gl_context, Arc::new(AtomicBool::new(false)))
    }

    /// Constructor with an explicit shared timing flag. Used by the threaded
    /// backend so the main thread can enable per-category timing when the
    /// stats dashboard sink is attached; immediate mode uses the plain
    /// `new()` (timing stays off).
    pub fn new_with_timing(
        gl_context: Option<WindowActiveGlContext>,
        _category_timing: Arc<AtomicBool>,
    ) -> Self {
        Self {
            resources: HashMap::new(),
            hot_reloaded_shaders: HashMap::new(),
            stats: ExecutorStats::default(),
            last_stats: RenderStats::default(),
            imm_vao: 0,
            imm_vbo: 0,
            fbo_stack: Vec::with_capacity(FBO_STACK_DEPTH),
            gl_context,
            current_program: 0,
            frame_start: std::time::Instant::now(),
            this_frame_stats: RenderStats::default(),
            #[cfg(feature = "stats-server")]
            category_timing: _category_timing,
            uniform_caches: HashMap::with_capacity(32), // Pre-allocate for typical shader count
            instance_vbo: 0,
            instance_vbo_capacity: 0,
            instance_vbo_capacity_u32: 0,
            texture_bindings: [TextureBinding::default(); MAX_TEXTURE_SLOTS],
            texture_binds_skipped: 0,
            camera_ubo: 0,
            material_ubo: 0,
            light_ubo: 0,
        }
    }

    /// Whether a usable GL context is attached. Without one, commands are no-ops.
    pub fn has_gl_context(&self) -> bool {
        self.gl_context.is_some()
    }

    /// Accumulated statistics.
    pub fn stats(&self) -> &ExecutorStats {
        &self.stats
    }

    /// The stats snapshot taken at the last `SwapBuffers`.
    pub fn stats_snapshot(&self) -> RenderStats {
        self.last_stats.clone()
    }

    /// Record generic per-command bookkeeping (commands_processed, draw/state
    /// counters, category_counts) and return an RAII guard that finishes
    /// timing this command's execution when it drops (i.e. at the end of the
    /// enclosing `cmd_*` method).
    /// Only elapsed-time write is deferred to `Drop`. Shared by both renderer
    /// backends: threaded mode's `execute()` dispatch and immediate mode's
    /// direct `cmd_*` calls both go through the same `cmd_*` methods, so
    /// instrumenting it here (instead of at `execute()`'s dispatch layer)
    /// gives both backends identical stats for free.
    #[cfg(feature = "stats-server")]
    #[inline(always)]
    pub(super) fn record_command(
        &mut self,
        category: CommandCategory,
        is_draw: bool,
        is_state: bool,
    ) -> StatsAggregator {
        self.stats.commands_processed += 1;
        self.this_frame_stats.commands += 1;
        if is_draw {
            self.stats.draw_calls += 1;
            self.this_frame_stats.draw_calls += 1;
        }
        if is_state {
            self.stats.state_changes += 1;
            self.this_frame_stats.state_changes += 1;
        }
        self.this_frame_stats.category_counts[category.index()] += 1;

        let start = self
            .category_timing
            .load(Ordering::Relaxed)
            .then(Instant::now);

        StatsAggregator {
            executor: self as *mut CommandExecutor,
            category,
            start,
        }
    }

    #[cfg(not(feature = "stats-server"))]
    #[inline(always)]
    pub(super) fn record_command(
        &mut self,
        _category: CommandCategory,
        _is_draw: bool,
        _is_state: bool,
    ) -> StatsAggregator {
        StatsAggregator
    }

    /// Initialize GL resources needed by the render thread
    pub fn init_gl(&mut self) {
        self.init_gl_intern();

        info!("Render thread GL resources initialized");
    }

    /// Main render loop
    pub fn execute(&mut self, cmd: RenderCommand) -> CommandReply {
        let mut reply = CommandReply::None;

        match cmd {
            // === State Management ===
            RenderCommand::SetViewport {
                x,
                y,
                width,
                height,
            } => self.cmd_set_viewport(x, y, width, height),

            RenderCommand::SetScissor {
                x,
                y,
                width,
                height,
            } => self.cmd_set_scissor(x, y, width, height),

            RenderCommand::EnableScissor(enable) => self.cmd_enable_scissor(enable),

            RenderCommand::SetBlendMode(mode) => {
                self.cmd_set_blend_mode(mode);
            }

            RenderCommand::SetCullFace(face) => {
                self.cmd_set_cull_face(face);
            }

            RenderCommand::SetDepthTest(enable) => self.cmd_set_depth_test(enable),

            RenderCommand::SetDepthWritable(enable) => self.cmd_set_depth_writable(enable),

            RenderCommand::SetWireframe(enable) => self.cmd_set_wireframe(enable),

            RenderCommand::SetLineWidth(width) => self.cmd_set_line_width(width),

            RenderCommand::SetPointSize(size) => self.cmd_set_point_size(size),

            // === Shader Operations ===
            RenderCommand::BindShader { handle } => self.cmd_bind_shader(handle),

            RenderCommand::BindShaderByResource { id, shader_key } => {
                self.cmd_bind_shader_by_resource(id, shader_key);
            }

            RenderCommand::UnbindShader => self.cmd_unbind_shader(),

            RenderCommand::SetUniformInt { location, value } => {
                self.cmd_set_uniform_int(location, value);
            }

            RenderCommand::SetUniformInt2 { location, value } => {
                self.cmd_set_uniform_int2(location, value);
            }

            RenderCommand::SetUniformInt3 { location, value } => {
                self.cmd_set_uniform_int3(location, value);
            }

            RenderCommand::SetUniformInt4 { location, value } => {
                self.cmd_set_uniform_int4(location, value);
            }

            RenderCommand::SetUniformFloat { location, value } => {
                self.cmd_set_uniform_float(location, value);
            }

            RenderCommand::SetUniformFloat2 { location, value } => {
                self.cmd_set_uniform_float2(location, value);
            }

            RenderCommand::SetUniformFloat3 { location, value } => {
                self.cmd_set_uniform_float3(location, value);
            }

            RenderCommand::SetUniformFloat4 { location, value } => {
                self.cmd_set_uniform_float4(location, value);
            }

            RenderCommand::SetUniformMat4 { location, value } => {
                self.cmd_set_uniform_mat4(location, value);
            }

            RenderCommand::SetInstanceUniforms(cmd) => {
                self.cmd_set_uniform_mat4(cmd.world_loc, cmd.world);
                self.cmd_set_uniform_mat4(cmd.world_it_loc, cmd.world_it);
                self.cmd_set_uniform_float(cmd.scale_loc, cmd.scale);
            }

            // === Name-based Uniform Operations ===
            // These use cached uniform location lookups to avoid repeated GL calls
            // Arc<str> enables O(1) cloning when building the cache key
            RenderCommand::SetUniformIntByName { name, value } => {
                self.cmd_set_uniform_int_by_name(name, value);
            }

            RenderCommand::SetUniformInt2ByName { name, value } => {
                self.cmd_set_uniform_int2_by_name(name, value);
            }

            RenderCommand::SetUniformInt3ByName { name, value } => {
                self.cmd_set_uniform_int3_by_name(name, value);
            }

            RenderCommand::SetUniformInt4ByName { name, value } => {
                self.cmd_set_uniform_int4_by_name(name, value);
            }

            RenderCommand::SetUniformFloatByName { name, value } => {
                self.cmd_set_uniform_float_by_name(name, value);
            }

            RenderCommand::SetUniformFloat2ByName { name, value } => {
                self.cmd_set_uniform_float2_by_name(name, value);
            }

            RenderCommand::SetUniformFloat3ByName { name, value } => {
                self.cmd_set_uniform_float3_by_name(name, value);
            }

            RenderCommand::SetUniformFloat4ByName { name, value } => {
                self.cmd_set_uniform_float4_by_name(name, value);
            }

            RenderCommand::SetUniformMat4ByName { name, value } => {
                self.cmd_set_uniform_mat4_by_name(name, value);
            }

            RenderCommand::SetUniformMat4ByGenericName { name, value } => {
                self.cmd_set_uniform_mat4_by_generic_name(name, value);
            }

            // === Texture Operations ===
            // Uses caching to skip redundant binds.
            // CRITICAL: After binding to a texture unit, we MUST reset ActiveTexture to TEXTURE0
            // to match direct mode behavior (see shader.rs apply_var). Without this reset,
            // subsequent GL operations that expect TEXTURE0 to be active will fail with
            // "unit 0 GLD_TEXTURE_INDEX_2D is unloadable" errors.
            RenderCommand::BindTexture2D { slot, handle } => self.cmd_bind_texture_2d(slot, handle),

            RenderCommand::BindTexture2DByResource { slot, id } => {
                self.cmd_bind_texture_2d_by_resource(slot, id);
            }

            RenderCommand::BindTexture1DByResource { slot, id } => {
                self.cmd_bind_texture_1d_by_resource(slot, id);
            }

            RenderCommand::BindTexture3D { slot, handle } => self.cmd_bind_texture_3d(slot, handle),

            RenderCommand::BindTexture3DByResource { slot, id } => {
                self.cmd_bind_texture_3d_by_resource(slot, id);
            }

            RenderCommand::BindTextureCube { slot, handle } => {
                self.cmd_bind_texture_cube(slot, handle);
            }

            RenderCommand::BindTextureCubeByResource { slot, id } => {
                self.cmd_bind_texture_cube_by_resource(slot, id);
            }

            RenderCommand::UnbindTexture { slot } => self.cmd_unbind_texture(slot),

            // === Texture State Commands ===
            RenderCommand::SetTexture2DMagFilter { handle, filter } => {
                self.cmd_set_texture_2d_mag_filter(handle, filter);
            }

            RenderCommand::SetTexture2DMinFilter { handle, filter } => {
                self.cmd_set_texture_2d_min_filter(handle, filter);
            }

            RenderCommand::SetTexture2DWrapMode { handle, mode } => {
                self.cmd_set_texture_2d_wrap_mode(handle, mode);
            }

            RenderCommand::SetTexture2DMipRange {
                handle,
                min_level,
                max_level,
            } => self.cmd_set_texture_2d_mip_range(handle, min_level, max_level),

            RenderCommand::GenerateMipmap2D { handle } => self.cmd_generate_mipmap_2d(handle),

            RenderCommand::UpdateTexture2DData {
                handle,
                width,
                height,
                internal_format,
                pixel_format,
                data_format,
                data,
            } => self.cmd_update_texture_2d_data(
                handle,
                width,
                height,
                internal_format,
                pixel_format,
                data_format,
                data,
            ),

            RenderCommand::UpdateTexture2DDataByResource {
                id,
                width,
                height,
                internal_format,
                pixel_format,
                data_format,
                data,
            } => {
                self.cmd_update_texture_2d_data_by_resource(
                    id,
                    width,
                    height,
                    internal_format,
                    pixel_format,
                    data_format,
                    data,
                );
            }

            RenderCommand::SetTexture2DAnisotropy { handle, factor } => {
                self.cmd_set_texture_2d_anisotropy(handle, factor);
            }

            RenderCommand::SetTexture2DAnisotropyByResource { id, factor } => {
                self.cmd_set_texture_2d_anisotropy_by_resource(id, factor);
            }

            RenderCommand::SetTexture2DMipRangeByResource {
                id,
                min_level,
                max_level,
            } => {
                self.cmd_set_texture_2d_mip_range_by_resource(id, min_level, max_level);
            }

            RenderCommand::SetTexel1DByResource { id, x, color } => {
                self.cmd_set_texel_1d_by_resource(id, x, color);
            }

            RenderCommand::SetTexel2DByResource { id, x, y, color } => {
                self.cmd_set_texel_2d_by_resource(id, x, y, color);
            }

            RenderCommand::SetTextureMagFilterByResource { id, filter } => {
                self.cmd_set_texture_mag_filter_by_resource(id, filter);
            }

            RenderCommand::SetTextureMinFilterByResource { id, filter } => {
                self.cmd_set_texture_min_filter_by_resource(id, filter);
            }

            RenderCommand::SetTextureWrapModeByResource { id, mode } => {
                self.cmd_set_texture_wrap_mode_by_resource(id, mode);
            }

            RenderCommand::GenerateMipmapByResource { id } => {
                self.cmd_generate_mipmap_by_resource(id);
            }

            RenderCommand::UpdateTexture1DDataByResource {
                id,
                width,
                internal_format,
                pixel_format,
                data_format,
                data,
            } => {
                self.cmd_update_texture_1d_data_by_resource(
                    id,
                    width,
                    internal_format,
                    pixel_format,
                    data_format,
                    data,
                );
            }

            RenderCommand::UpdateTexture3DDataByResource {
                id,
                width,
                height,
                depth,
                internal_format,
                pixel_format,
                data_format,
                data,
            } => {
                self.cmd_update_texture_3d_data_by_resource(
                    id,
                    width,
                    height,
                    depth,
                    internal_format,
                    pixel_format,
                    data_format,
                    data,
                );
            }

            RenderCommand::UpdateTextureCubeFaceDataByResource {
                id,
                face,
                level,
                size,
                internal_format,
                pixel_format,
                data_format,
                data,
            } => {
                self.cmd_update_texture_cube_face_data_by_resource(
                    id,
                    face,
                    level,
                    size,
                    internal_format,
                    pixel_format,
                    data_format,
                    data,
                );
            }

            RenderCommand::CopyTexture2DFromFramebufferByResource {
                id,
                internal_format,
                width,
                height,
            } => {
                self.cmd_copy_texture_2d_from_framebuffer_by_resource(
                    id,
                    internal_format,
                    width,
                    height,
                );
            }

            RenderCommand::ReadTexture1DData {
                id,
                pixel_format,
                data_format,
                reply_tx,
            } => {
                let data = self.cmd_read_texture_1d_data(id, pixel_format, data_format);
                let _ = reply_tx.send(data);
            }

            RenderCommand::ReadTexture2DData {
                id,
                pixel_format,
                data_format,
                reply_tx,
            } => {
                let data = self.cmd_read_texture_2d_data(id, pixel_format, data_format);
                let _ = reply_tx.send(data);
            }

            RenderCommand::ReadTexture3DData {
                id,
                pixel_format,
                data_format,
                reply_tx,
            } => {
                let data = self.cmd_read_texture_3d_data(id, pixel_format, data_format);
                let _ = reply_tx.send(data);
            }

            RenderCommand::ReadTextureCubeFaceData {
                id,
                face,
                level,
                pixel_format,
                data_format,
                reply_tx,
            } => {
                let data = self.cmd_read_texture_cube_face_data(
                    id,
                    face,
                    level,
                    pixel_format,
                    data_format,
                );
                let _ = reply_tx.send(data);
            }

            RenderCommand::SamplePixel2DByResource { id, x, y, reply_tx } => {
                let data = self.cmd_sample_pixel_2d_by_resource(id, x, y);
                let _ = reply_tx.send(data);
            }

            RenderCommand::ReadFramebufferPixels {
                x,
                y,
                width,
                height,
                reply_tx,
            } => {
                let data = self.cmd_read_framebuffer_pixels(x, y, width, height);
                let _ = reply_tx.send(data);
            }

            // === Framebuffer Operations ===
            RenderCommand::PushFramebuffer {
                id: _,
                width: _,
                height: _,
            } => {
                self.cmd_push_framebuffer();
            }

            RenderCommand::PopFramebuffer => {
                self.cmd_pop_framebuffer();
            }

            RenderCommand::FramebufferAttachTexture2D {
                attachment,
                texture,
                level,
            } => self.cmd_framebuffer_attach_texture_2d(attachment, texture, level),

            RenderCommand::FramebufferAttachTexture2DByResource {
                attachment,
                id,
                level,
            } => {
                self.cmd_framebuffer_attach_texture_2d_by_resource(attachment, id, level);
            }

            RenderCommand::FramebufferAttachTexture3D {
                attachment,
                texture,
                layer,
                level,
            } => self.cmd_framebuffer_attach_texture_3d(attachment, texture, layer, level),

            RenderCommand::FramebufferAttachTexture3DByResource {
                attachment,
                id,
                layer,
                level,
            } => {
                self.cmd_framebuffer_attach_texture_3d_by_resource(attachment, id, layer, level);
            }

            RenderCommand::FramebufferAttachTextureCube {
                attachment,
                texture,
                face,
                level,
            } => self.cmd_framebuffer_attach_texture_cube(attachment, texture, face, level),

            RenderCommand::FramebufferAttachTextureCubeByResource {
                attachment,
                id,
                face,
                level,
            } => {
                self.cmd_framebuffer_attach_texture_cube_by_resource(attachment, id, face, level);
            }

            RenderCommand::SetDrawBuffers { count } => self.cmd_set_draw_buffers(count),

            RenderCommand::BindFramebuffer { handle } => self.cmd_bind_framebuffer(handle),

            RenderCommand::BindDefaultFramebuffer => self.cmd_bind_default_framebuffer(),

            RenderCommand::Clear { color, depth } => self.cmd_clear(color, depth),

            // === Mesh Operations ===
            RenderCommand::BindMesh { vao } => self.cmd_bind_mesh(vao),

            RenderCommand::UnbindMesh => self.cmd_unbind_mesh(),

            // === Drawing Operations ===
            RenderCommand::DrawMesh {
                vao,
                index_count,
                primitive,
            } => {
                self.cmd_draw_mesh(vao, index_count, primitive);
            }

            RenderCommand::DrawMeshInstanced {
                vao,
                index_count,
                instance_count,
                primitive,
            } => {
                self.cmd_draw_mesh_instanced(vao, index_count, instance_count, primitive);
            }

            RenderCommand::DrawMeshByResource {
                id,
                index_count,
                primitive,
            } => {
                self.cmd_draw_mesh_by_resource(id, index_count, primitive);
            }

            RenderCommand::DrawMeshInstancedByResource {
                id,
                index_count,
                instance_count,
                primitive,
            } => {
                self.cmd_draw_mesh_instanced_by_resource(
                    id,
                    index_count,
                    instance_count,
                    primitive,
                );
            }

            RenderCommand::DrawInstancedWithData {
                mesh_id,
                index_count,
                instances,
                primitive,
            } => {
                self.cmd_draw_instanced_with_data(mesh_id, index_count, &instances, primitive);
            }

            RenderCommand::DrawInstancedIndices {
                mesh_id,
                index_count,
                indices,
                primitive,
            } => {
                self.cmd_draw_instanced_indices(mesh_id, index_count, &indices, primitive);
            }

            RenderCommand::BindMeshByResource { id } => self.cmd_bind_mesh_by_resource(id),

            RenderCommand::DrawImmediate {
                primitive,
                vertices,
            } => {
                self.cmd_draw_immediate(primitive, &vertices);
            }

            // === Resource Creation ===
            RenderCommand::CreateShader {
                id,
                vertex_src,
                fragment_src,
                reply_tx,
            } => {
                let data = self.cmd_create_shader(id, vertex_src, fragment_src);
                let _ = reply_tx.send(data);
            }

            RenderCommand::GetUniformLocationByResource { id, name, reply_tx } => {
                let data = self.cmd_get_uniform_location_by_resource(id, name);
                let _ = reply_tx.send(data);
            }

            RenderCommand::ReloadShader {
                shader_key,
                vertex_src,
                fragment_src,
            } => {
                reply = self.cmd_reload_shader(&shader_key, &vertex_src, &fragment_src);
            }

            RenderCommand::CreateTexture1D {
                id,
                width,
                format,
                data,
            } => self.cmd_create_texture_1d(id, width, format, data),

            RenderCommand::CreateTexture2D {
                id,
                width,
                height,
                format,
                data,
            } => self.cmd_create_texture_2d(id, width, height, format, data),

            RenderCommand::CreateTexture3D {
                id,
                width,
                height,
                depth,
                format,
                data,
            } => self.cmd_create_texture_3d(id, width, height, depth, format, data),

            RenderCommand::CreateTextureCube { id, size, format } => {
                self.cmd_create_texture_cube(id, size, format);
            }

            RenderCommand::CreateMesh {
                id,
                vertices,
                indices,
                vertex_format,
            } => self.cmd_create_mesh(id, vertices, indices, vertex_format),

            RenderCommand::DestroyResources { ids } => self.cmd_destroy_resource(&ids),

            // === Uniform Buffer Objects ===
            RenderCommand::CreateCameraUBO => self.cmd_create_camera_ubo(),
            RenderCommand::UpdateCameraUBO { data } => self.cmd_update_camera_ubo(&data),
            RenderCommand::CreateMaterialUBO => self.cmd_create_material_ubo(),
            RenderCommand::UpdateMaterialUBO { data } => self.cmd_update_material_ubo(&data),
            RenderCommand::CreateLightUBO => self.cmd_create_light_ubo(),
            RenderCommand::UpdateLightUBO { data } => self.cmd_update_light_ubo(&data),

            // === Window Operations ===
            RenderCommand::Resize { width, height } => self.cmd_resize(width, height),

            RenderCommand::SetPresentMode { mode } => self.cmd_set_present_mode(mode),

            RenderCommand::SwapBuffers => {
                reply = self.cmd_swap_buffers();
            }

            // === Synchronization ===
            RenderCommand::Flush => self.cmd_flush(),

            RenderCommand::Fence { fence_id } => {
                reply = self.cmd_fence(fence_id);
            }

            RenderCommand::PacingFence { fence_id } => {
                reply = self.cmd_pacing_fence(fence_id);
            }

            RenderCommand::Shutdown => {
                // Handled by the caller's loop
            }
        }

        reply
    }
}
