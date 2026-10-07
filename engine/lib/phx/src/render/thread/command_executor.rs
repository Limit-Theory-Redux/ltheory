use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
#[cfg(feature = "stats-server")]
use std::sync::atomic::Ordering;
#[cfg(feature = "stats-server")]
use std::time::Instant;

use tracing::info;

use super::command_executor_gl_binding::GlBindingState;
use crate::render::{
    CommandCategory, MAX_COLOR_ATTACHMENTS, RenderCommand, RenderStats, ResourceId,
    ShaderReloadResult, ViewDim, gl,
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
pub(super) enum GpuResource {
    Shader { program: u32 },
    Texture1D { handle: u32 },
    Texture2D { handle: u32 },
    Texture3D { handle: u32 },
    TextureCube { handle: u32 },
    Mesh { vao: u32, vbo: u32, ebo: u32 },
}

/// Statistics from the render thread (local copy)
#[derive(Debug, Clone, Default)]
pub struct ExecutorStats {
    pub commands_processed: u64,
    pub draw_calls: u64,
    pub state_changes: u64,
    pub frame_count: u64,
}

/// One framebuffer attachment as identified by the FBO cache: a texture and
/// the part of it (mip level, cube face or 3D layer) that is attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct AttachKey {
    pub id: ResourceId,
    pub dim: ViewDim,
    pub mip: u8,
}

/// FBO cache key: the set of attachments a pass renders to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct FboKey {
    pub color: [Option<AttachKey>; MAX_COLOR_ATTACHMENTS],
    pub depth: Option<AttachKey>,
}

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
    /// GL framebuffer objects by attachment set. An entry lives until one of
    /// its textures is destroyed.
    pub(super) fbo_cache: HashMap<FboKey, u32>,
    /// Texture -> the cache keys that attach it, for eviction on destroy.
    pub(super) texture_fbos: HashMap<ResourceId, Vec<FboKey>>,
    /// Framebuffer bound by the open render pass (0 = default framebuffer).
    pub(super) bound_fbo: u32,
    // GL context for buffer swapping (stored here to allow access during execute)
    pub(super) gl_context: Option<WindowActiveGlContext>,
    /// The program of the current pipeline.
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
    /// Texture binding cache: tracks which texture is bound to each slot
    /// Avoids redundant glBindTexture calls
    pub(super) texture_bindings: [TextureBinding; MAX_TEXTURE_SLOTS],
    /// Stats: number of texture binds skipped due to caching
    pub(super) texture_binds_skipped: u64,
    /// Binding model state: pipelines, samplers, bind groups, GL state cache,
    /// uniform ring buffers.
    pub(super) binding: GlBindingState,
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
            fbo_cache: HashMap::new(),
            texture_fbos: HashMap::new(),
            bound_fbo: 0,
            gl_context,
            current_program: 0,
            frame_start: std::time::Instant::now(),
            this_frame_stats: RenderStats::default(),
            #[cfg(feature = "stats-server")]
            category_timing: _category_timing,
            uniform_caches: HashMap::with_capacity(32), // Pre-allocate for typical shader count
            texture_bindings: [TextureBinding::default(); MAX_TEXTURE_SLOTS],
            texture_binds_skipped: 0,
            binding: GlBindingState::new(),
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
            // === Shader Operations ===
            // === Name-based Uniform Operations ===
            // These use cached uniform location lookups to avoid repeated GL calls
            // Arc<str> enables O(1) cloning when building the cache key
            // === Texture Operations ===
            // Uses caching to skip redundant binds.
            // CRITICAL: After binding to a texture unit, we MUST reset ActiveTexture to TEXTURE0
            // to match direct mode behavior (see shader.rs apply_var). Without this reset,
            // subsequent GL operations that expect TEXTURE0 to be active will fail with
            // "unit 0 GLD_TEXTURE_INDEX_2D is unloadable" errors.
            // === Texture State Commands ===
            RenderCommand::UpdateTexture { id, region, data } => {
                self.cmd_update_texture(id, &region, data);
            }

            RenderCommand::GenerateMips { id } => {
                self.cmd_generate_mips(id);
            }

            RenderCommand::SetTexel1DByResource { id, x, color } => {
                self.cmd_set_texel_1d_by_resource(id, x, color);
            }

            RenderCommand::SetTexel2DByResource { id, x, y, color } => {
                self.cmd_set_texel_2d_by_resource(id, x, y, color);
            }

            RenderCommand::CopyTexture { src, dst, size } => {
                self.cmd_copy_texture(&src, &dst, size);
            }

            RenderCommand::CopyTexture2DFromFramebufferByResource {
                id,
                format,
                width,
                height,
            } => {
                self.cmd_copy_texture_2d_from_framebuffer_by_resource(id, format, width, height);
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

            // === Render Passes ===
            RenderCommand::BeginRenderPass(desc) => self.cmd_begin_render_pass(&desc),

            RenderCommand::EndRenderPass => self.cmd_end_render_pass(),

            RenderCommand::BeginFrame { slot } => self.cmd_begin_frame(slot),

            RenderCommand::PassCommands(mut commands) => self.cmd_pass_commands(&mut commands),

            // === Binding model objects ===
            RenderCommand::CreatePipeline { id, desc } => self.cmd_create_pipeline(id, &desc),

            RenderCommand::CreateSampler { id, desc } => self.cmd_create_sampler(id, &desc),

            RenderCommand::CreateBindGroup {
                id,
                shader,
                group,
                entries,
            } => self.cmd_create_bind_group(id, shader, group, &entries),

            RenderCommand::DestroyBindGroups { ids } => self.cmd_destroy_bind_groups(&ids),

            RenderCommand::CreateBuffer { id, size } => self.cmd_create_buffer(id, size),

            RenderCommand::WriteBuffer { id, offset, data } => {
                self.cmd_write_buffer(id, offset, &data)
            }

            // === Mesh Operations ===
            // === Drawing Operations ===
            // === Resource Creation ===
            RenderCommand::CreateShader {
                id,
                vertex_src,
                fragment_src,
                layout,
                reply_tx,
            } => {
                let data = self.cmd_create_shader(id, vertex_src, fragment_src, &layout);
                let _ = reply_tx.send(data);
            }

            RenderCommand::ReloadShader {
                shader_key,
                vertex_src,
                fragment_src,
            } => {
                reply = self.cmd_reload_shader(&shader_key, &vertex_src, &fragment_src);
            }

            RenderCommand::CreateTexture { id, desc, data } => {
                self.cmd_create_texture(id, &desc, data);
            }

            RenderCommand::CreateMesh {
                id,
                vertices,
                indices,
                vertex_format,
            } => self.cmd_create_mesh(id, vertices, indices, vertex_format),

            RenderCommand::DestroyResources { ids } => self.cmd_destroy_resource(&ids),

            // === Uniform Buffer Objects ===

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
