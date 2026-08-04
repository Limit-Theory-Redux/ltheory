#![allow(unsafe_code)]

use std::collections::HashMap;
use std::ptr;
use std::sync::Arc;

use tracing::{debug, error, info, warn};

use crate::render::gl::types::GLsizeiptr;
use crate::render::{
    BlendMode, CmdPrimitiveType, CullFace, ImmVertex, InstanceData, RenderCommand, RenderStats,
    ResourceId, ShaderReloadResult, TexFormat, VertexFormat, gl,
};
use crate::window::{WindowActiveGlContext, WindowGlContext};

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
    ShaderReload(ShaderReloadResult),
    Stats(RenderStats),
}

/// GPU resource stored on the render thread
#[derive(Debug)]
#[expect(dead_code)]
enum GpuResource {
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
struct FboEntry {
    handle: u32,
    color_index: i32,
}

const FBO_STACK_DEPTH: usize = 16;
const DRAW_BUFS: [u32; 4] = [
    gl::COLOR_ATTACHMENT0,
    gl::COLOR_ATTACHMENT1,
    gl::COLOR_ATTACHMENT2,
    gl::COLOR_ATTACHMENT3,
];

/// Maximum number of texture units to track for caching
/// OpenGL requires at least 16, most GPUs support 32+
const MAX_TEXTURE_SLOTS: usize = 16;

/// Texture type for binding cache
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextureType {
    Texture1D,
    Texture2D,
    Texture3D,
    TextureCube,
}

impl TextureType {
    fn to_gl_target(self) -> gl::types::GLenum {
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
struct TextureBinding {
    /// GL handle (0 = unbound)
    handle: u32,
    /// Texture type (only valid if handle != 0)
    tex_type: Option<TextureType>,
}

impl TextureBinding {
    fn new(handle: u32, tex_type: TextureType) -> Self {
        Self {
            handle,
            tex_type: Some(tex_type),
        }
    }

    fn unbound() -> Self {
        Self::default()
    }
}

/// Owns the GL context and every GPU object, and executes `RenderCommand`s
/// against them. Runs on the render thread in command mode, or inline on the
/// main thread in immediate mode - it has no idea which.
pub struct CommandExecutor {
    resources: HashMap<ResourceId, GpuResource>,
    /// Hot-reloaded shaders by shader_key (separate from resources for override)
    hot_reloaded_shaders: HashMap<String, u32>,
    stats: ExecutorStats,
    /// Snapshot taken at the last `SwapBuffers`, readable at any later point
    /// via `stats_snapshot()`. Also what `SwapBuffers` returns as
    /// `CommandReply::Stats` for the threaded backend to forward.
    last_stats: RenderStats,
    // Immediate mode VAO/VBO for DrawImmediate commands
    imm_vao: u32,
    imm_vbo: u32,
    // FBO stack for push/pop framebuffer operations
    fbo_stack: Vec<FboEntry>,
    // GL context for buffer swapping (stored here to allow access during execute)
    gl_context: Option<WindowActiveGlContext>,
    // Currently bound shader program (needed for name-based uniform lookups)
    current_program: u32,
    // Frame timing
    frame_start: std::time::Instant,
    commands_this_frame: u64,
    draw_calls_this_frame: u64,
    /// Per-shader cache for uniform locations: program -> (name -> location)
    /// NOT cleared on shader change - preserves locations across shader switches
    /// Uses Arc<str> as key for O(1) cloning from commands
    uniform_caches: HashMap<u32, HashMap<Arc<str>, i32>>,
    /// Instance buffer for DrawInstancedWithData (reused across frames)
    instance_vbo: u32,
    /// Capacity of instance buffer in instances
    instance_vbo_capacity: usize,
    /// Texture binding cache: tracks which texture is bound to each slot
    /// Avoids redundant glBindTexture calls
    texture_bindings: [TextureBinding; MAX_TEXTURE_SLOTS],
    /// Stats: number of texture binds skipped due to caching
    texture_binds_skipped: u64,
    /// Camera UBO handle (0 if not created yet)
    camera_ubo: u32,
    /// Material UBO handle (0 if not created yet)
    material_ubo: u32,
    /// Light UBO handle (0 if not created yet)
    light_ubo: u32,
}

impl CommandExecutor {
    pub fn new(gl_context: Option<WindowActiveGlContext>) -> Self {
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
            commands_this_frame: 0,
            draw_calls_this_frame: 0,
            uniform_caches: HashMap::with_capacity(32), // Pre-allocate for typical shader count
            instance_vbo: 0,
            instance_vbo_capacity: 0,
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

    fn bind_texture_cached(&mut self, slot: u32, handle: u32, tex_type: TextureType) -> bool {
        let slot_idx = slot as usize;
        if slot_idx >= MAX_TEXTURE_SLOTS {
            // Slot out of range, just bind directly
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + slot);
                gl::BindTexture(tex_type.to_gl_target(), handle);
                gl::ActiveTexture(gl::TEXTURE0);
            }
            return true;
        }

        let new_binding = TextureBinding::new(handle, tex_type);
        let current = &self.texture_bindings[slot_idx];

        // Check if already bound
        if current.handle == handle && current.tex_type == Some(tex_type) {
            self.texture_binds_skipped += 1;
            return false;
        }

        // Different texture or type - need to bind
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(tex_type.to_gl_target(), handle);
            gl::ActiveTexture(gl::TEXTURE0);
        }

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
                return;
            }

            // Unbind based on current type
            if let Some(tex_type) = current.tex_type {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0 + slot);
                    gl::BindTexture(tex_type.to_gl_target(), 0);
                    gl::ActiveTexture(gl::TEXTURE0);
                }
            }

            self.texture_bindings[slot_idx] = TextureBinding::unbound();
        } else {
            // Slot out of range, can't track - just unbind 2D as fallback
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + slot);
                gl::BindTexture(gl::TEXTURE_2D, 0);
                gl::ActiveTexture(gl::TEXTURE0);
            }
        }
    }

    /// Invalidate texture cache (call when GL context state may be externally modified)
    fn invalidate_texture_cache(&mut self) {
        self.texture_bindings = [TextureBinding::default(); MAX_TEXTURE_SLOTS];
    }

    /// Get uniform location with per-shader caching to avoid repeated gl::GetUniformLocation calls.
    /// Cache is keyed by (program, name) - preserves locations across shader switches.
    /// Takes Arc<str> to avoid allocation - Arc::clone() is O(1).
    /// Returns -1 if uniform not found (matches OpenGL behavior).
    fn get_uniform_location_cached(&mut self, name: Arc<str>) -> i32 {
        if self.current_program == 0 {
            return -1;
        }

        // Get or create cache for current shader
        let cache = self
            .uniform_caches
            .entry(self.current_program)
            .or_insert_with(|| HashMap::with_capacity(32));

        // Check cache first
        if let Some(&loc) = cache.get(&name) {
            return loc;
        }

        // Cache miss - query OpenGL
        let c_name = std::ffi::CString::new(&*name).unwrap_or_default();
        let loc = unsafe { gl::GetUniformLocation(self.current_program, c_name.as_ptr()) };

        // Store in cache (even if -1 to avoid repeated lookups for non-existent uniforms)
        cache.insert(name, loc);
        loc
    }

    /// Initialize GL resources needed by the render thread
    pub fn init_gl(&mut self) {
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
            let stride = std::mem::size_of::<ImmVertex>() as i32; // 12 floats = 48 bytes

            // Position attribute (location 0 = vertex_position)
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null());

            // Normal attribute (location 1 = vertex_normal)
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * 4) as *const _);

            // UV attribute (location 2 = vertex_uv)
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, stride, (6 * 4) as *const _);

            // Color attribute (location 3 = vertex_color)
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(3, 4, gl::FLOAT, gl::FALSE, stride, (8 * 4) as *const _);

            gl::BindVertexArray(0);
        }

        info!("Render thread GL resources initialized");
    }

    /// Main render loop
    pub fn execute(&mut self, cmd: RenderCommand) -> CommandReply {
        let mut reply = CommandReply::None;

        self.stats.commands_processed += 1;
        self.commands_this_frame += 1;

        if cmd.is_draw_call() {
            self.stats.draw_calls += 1;
            self.draw_calls_this_frame += 1;
        }
        if cmd.is_state_change() {
            self.stats.state_changes += 1;
        }

        match cmd {
            // === State Management ===
            RenderCommand::SetViewport {
                x,
                y,
                width,
                height,
            } => unsafe {
                gl::Viewport(x, y, width, height);
            },

            RenderCommand::SetScissor {
                x,
                y,
                width,
                height,
            } => unsafe {
                gl::Scissor(x, y, width, height);
            },

            RenderCommand::EnableScissor(enable) => unsafe {
                if enable {
                    gl::Enable(gl::SCISSOR_TEST);
                } else {
                    gl::Disable(gl::SCISSOR_TEST);
                }
            },

            RenderCommand::SetBlendMode(mode) => {
                self.set_blend_mode(mode);
            }

            RenderCommand::SetCullFace(face) => {
                self.set_cull_face(face);
            }

            RenderCommand::SetDepthTest(enable) => unsafe {
                if enable {
                    gl::Enable(gl::DEPTH_TEST);
                } else {
                    gl::Disable(gl::DEPTH_TEST);
                }
            },

            RenderCommand::SetDepthWritable(enable) => unsafe {
                gl::DepthMask(if enable { gl::TRUE } else { gl::FALSE });
            },

            RenderCommand::SetWireframe(enable) => unsafe {
                gl::PolygonMode(gl::FRONT_AND_BACK, if enable { gl::LINE } else { gl::FILL });
            },

            RenderCommand::SetLineWidth(width) => unsafe {
                gl::LineWidth(width);
            },

            RenderCommand::SetPointSize(size) => unsafe {
                gl::PointSize(size);
            },

            // === Shader Operations ===
            RenderCommand::BindShader { handle } => unsafe {
                if handle.0 != self.current_program {
                    // Invalidate texture cache when shader changes - different shaders
                    // expect different textures in the same slots (critical for post-fx)
                    self.invalidate_texture_cache();
                }
                gl::UseProgram(handle.0);
                self.current_program = handle.0;
            },

            RenderCommand::BindShaderByResource { id, shader_key } => {
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
                    if p != self.current_program {
                        // Invalidate texture cache when shader changes - different shaders
                        // expect different textures in the same slots (critical for post-fx)
                        self.invalidate_texture_cache();
                    }
                    unsafe {
                        gl::UseProgram(p);
                    }
                    self.current_program = p;
                } else {
                    error!("BindShaderByResource: resource {:?} not found!", id);
                }
            }

            RenderCommand::UnbindShader => unsafe {
                self.invalidate_texture_cache();
                gl::UseProgram(0);
                self.current_program = 0;
            },

            RenderCommand::SetUniformInt { location, value } => unsafe {
                gl::Uniform1i(location, value);
            },

            RenderCommand::SetUniformInt2 { location, value } => unsafe {
                gl::Uniform2i(location, value[0], value[1]);
            },

            RenderCommand::SetUniformInt3 { location, value } => unsafe {
                gl::Uniform3i(location, value[0], value[1], value[2]);
            },

            RenderCommand::SetUniformInt4 { location, value } => unsafe {
                gl::Uniform4i(location, value[0], value[1], value[2], value[3]);
            },

            RenderCommand::SetUniformFloat { location, value } => unsafe {
                gl::Uniform1f(location, value);
            },

            RenderCommand::SetUniformFloat2 { location, value } => unsafe {
                gl::Uniform2f(location, value[0], value[1]);
            },

            RenderCommand::SetUniformFloat3 { location, value } => unsafe {
                gl::Uniform3f(location, value[0], value[1], value[2]);
            },

            RenderCommand::SetUniformFloat4 { location, value } => unsafe {
                gl::Uniform4f(location, value[0], value[1], value[2], value[3]);
            },

            RenderCommand::SetUniformMat4 { location, value } => unsafe {
                gl::UniformMatrix4fv(location, 1, gl::FALSE, value.as_ptr());
            },

            // === Name-based Uniform Operations ===
            // These use cached uniform location lookups to avoid repeated GL calls
            // Arc<str> enables O(1) cloning when building the cache key
            RenderCommand::SetUniformIntByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe {
                        gl::Uniform1i(loc, value);
                    }
                }
            }

            RenderCommand::SetUniformInt2ByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe {
                        gl::Uniform2i(loc, value[0], value[1]);
                    }
                }
            }

            RenderCommand::SetUniformInt3ByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe {
                        gl::Uniform3i(loc, value[0], value[1], value[2]);
                    }
                }
            }

            RenderCommand::SetUniformInt4ByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe {
                        gl::Uniform4i(loc, value[0], value[1], value[2], value[3]);
                    }
                }
            }

            RenderCommand::SetUniformFloatByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe {
                        gl::Uniform1f(loc, value);
                    }
                }
            }

            RenderCommand::SetUniformFloat2ByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe {
                        gl::Uniform2f(loc, value[0], value[1]);
                    }
                }
            }

            RenderCommand::SetUniformFloat3ByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe {
                        gl::Uniform3f(loc, value[0], value[1], value[2]);
                    }
                }
            }

            RenderCommand::SetUniformFloat4ByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe {
                        gl::Uniform4f(loc, value[0], value[1], value[2], value[3]);
                    }
                }
            }

            RenderCommand::SetUniformMat4ByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe {
                        gl::UniformMatrix4fv(loc, 1, gl::FALSE, value.as_ptr());
                    }
                }
            }

            // === Texture Operations ===
            // Uses caching to skip redundant binds.
            // CRITICAL: After binding to a texture unit, we MUST reset ActiveTexture to TEXTURE0
            // to match direct mode behavior (see shader.rs apply_var). Without this reset,
            // subsequent GL operations that expect TEXTURE0 to be active will fail with
            // "unit 0 GLD_TEXTURE_INDEX_2D is unloadable" errors.
            RenderCommand::BindTexture2D { slot, handle } => {
                self.bind_texture_cached(slot, handle.0, TextureType::Texture2D);
            }

            RenderCommand::BindTexture2DByResource { slot, id } => {
                if let Some(GpuResource::Texture2D { handle }) = self.resources.get(&id) {
                    self.bind_texture_cached(slot, *handle, TextureType::Texture2D);
                } else {
                    warn!("BindTexture2DByResource: resource {:?} not found", id);
                }
            }

            RenderCommand::BindTexture1DByResource { slot, id } => {
                if let Some(GpuResource::Texture1D { handle }) = self.resources.get(&id) {
                    self.bind_texture_cached(slot, *handle, TextureType::Texture1D);
                } else {
                    warn!("BindTexture1DByResource: resource {:?} not found", id);
                }
            }

            RenderCommand::BindTexture3D { slot, handle } => {
                self.bind_texture_cached(slot, handle.0, TextureType::Texture3D);
            }

            RenderCommand::BindTexture3DByResource { slot, id } => {
                if let Some(GpuResource::Texture3D { handle }) = self.resources.get(&id) {
                    self.bind_texture_cached(slot, *handle, TextureType::Texture3D);
                } else {
                    warn!("BindTexture3DByResource: resource {:?} not found", id);
                }
            }

            RenderCommand::BindTextureCube { slot, handle } => {
                self.bind_texture_cached(slot, handle.0, TextureType::TextureCube);
            }

            RenderCommand::BindTextureCubeByResource { slot, id } => {
                if let Some(GpuResource::TextureCube { handle }) = self.resources.get(&id) {
                    self.bind_texture_cached(slot, *handle, TextureType::TextureCube);
                } else {
                    warn!("BindTextureCubeByResource: resource {:?} not found", id);
                }
            }

            RenderCommand::UnbindTexture { slot } => {
                self.unbind_texture_cached(slot);
            }

            // === Texture State Commands ===
            RenderCommand::SetTexture2DMagFilter { handle, filter } => unsafe {
                gl::BindTexture(gl::TEXTURE_2D, handle.0);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter as i32);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            },

            RenderCommand::SetTexture2DMinFilter { handle, filter } => unsafe {
                gl::BindTexture(gl::TEXTURE_2D, handle.0);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as i32);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            },

            RenderCommand::SetTexture2DWrapMode { handle, mode } => unsafe {
                gl::BindTexture(gl::TEXTURE_2D, handle.0);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, mode as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, mode as i32);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            },

            RenderCommand::SetTexture2DMipRange {
                handle,
                min_level,
                max_level,
            } => unsafe {
                gl::BindTexture(gl::TEXTURE_2D, handle.0);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, min_level);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, max_level);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            },

            RenderCommand::GenerateMipmap2D { handle } => unsafe {
                gl::BindTexture(gl::TEXTURE_2D, handle.0);
                gl::GenerateMipmap(gl::TEXTURE_2D);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            },

            RenderCommand::UpdateTexture2DData {
                handle,
                width,
                height,
                internal_format,
                pixel_format,
                data_format,
                data,
            } => unsafe {
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
                gl::BindTexture(gl::TEXTURE_2D, 0);
            },

            RenderCommand::UpdateTexture2DDataByResource {
                id,
                width,
                height,
                internal_format,
                pixel_format,
                data_format,
                data,
            } => {
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
                        gl::TexParameteri(
                            gl::TEXTURE_2D,
                            gl::TEXTURE_MIN_FILTER,
                            gl::NEAREST as i32,
                        );
                        gl::TexParameteri(
                            gl::TEXTURE_2D,
                            gl::TEXTURE_MAG_FILTER,
                            gl::NEAREST as i32,
                        );
                        gl::TexParameteri(
                            gl::TEXTURE_2D,
                            gl::TEXTURE_WRAP_S,
                            gl::CLAMP_TO_EDGE as i32,
                        );
                        gl::TexParameteri(
                            gl::TEXTURE_2D,
                            gl::TEXTURE_WRAP_T,
                            gl::CLAMP_TO_EDGE as i32,
                        );
                        gl::BindTexture(gl::TEXTURE_2D, 0);
                    }
                } else {
                    warn!("UpdateTexture2DDataByResource: resource {:?} not found", id);
                }
            }

            RenderCommand::SetTexture2DAnisotropy { handle, factor } => unsafe {
                gl::BindTexture(gl::TEXTURE_2D, handle.0);
                gl::TexParameterf(gl::TEXTURE_2D, gl::TEXTURE_MAX_ANISOTROPY_EXT, factor);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            },

            RenderCommand::SetTexture2DAnisotropyByResource { id, factor } => {
                if let Some((target, handle)) = self.texture_target_and_handle(id) {
                    unsafe {
                        gl::BindTexture(target, handle);
                        gl::TexParameterf(target, gl::TEXTURE_MAX_ANISOTROPY_EXT, factor);
                        gl::BindTexture(target, 0);
                    }
                } else {
                    warn!(
                        "SetTexture2DAnisotropyByResource: resource {:?} not found",
                        id
                    );
                }
            }

            RenderCommand::SetTexture2DMipRangeByResource {
                id,
                min_level,
                max_level,
            } => {
                if let Some((target, handle)) = self.texture_target_and_handle(id) {
                    unsafe {
                        gl::BindTexture(target, handle);
                        gl::TexParameteri(target, gl::TEXTURE_BASE_LEVEL, min_level);
                        gl::TexParameteri(target, gl::TEXTURE_MAX_LEVEL, max_level);
                        gl::BindTexture(target, 0);
                    }
                } else {
                    warn!(
                        "SetTexture2DMipRangeByResource: resource {:?} not found",
                        id
                    );
                }
            }

            RenderCommand::SetTexel1DByResource { id, x, color } => {
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
                        gl::BindTexture(target, 0);
                    }
                } else {
                    warn!("SetTexel1DByResource: resource {:?} not found", id);
                }
            }

            RenderCommand::SetTexel2DByResource { id, x, y, color } => {
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
                        gl::BindTexture(target, 0);
                    }
                } else {
                    warn!("SetTexel2DByResource: resource {:?} not found", id);
                }
            }

            RenderCommand::SetTextureMagFilterByResource { id, filter } => {
                if let Some((target, handle)) = self.texture_target_and_handle(id) {
                    unsafe {
                        gl::BindTexture(target, handle);
                        gl::TexParameteri(target, gl::TEXTURE_MAG_FILTER, filter as i32);
                        gl::BindTexture(target, 0);
                    }
                } else {
                    warn!("SetTextureMagFilterByResource: resource {:?} not found", id);
                }
            }

            RenderCommand::SetTextureMinFilterByResource { id, filter } => {
                if let Some((target, handle)) = self.texture_target_and_handle(id) {
                    unsafe {
                        gl::BindTexture(target, handle);
                        gl::TexParameteri(target, gl::TEXTURE_MIN_FILTER, filter as i32);
                        gl::BindTexture(target, 0);
                    }
                } else {
                    warn!("SetTextureMinFilterByResource: resource {:?} not found", id);
                }
            }

            RenderCommand::SetTextureWrapModeByResource { id, mode } => {
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
                        gl::BindTexture(target, 0);
                    }
                } else {
                    warn!("SetTextureWrapModeByResource: resource {:?} not found", id);
                }
            }

            RenderCommand::GenerateMipmapByResource { id } => {
                if let Some((target, handle)) = self.texture_target_and_handle(id) {
                    unsafe {
                        gl::BindTexture(target, handle);
                        gl::GenerateMipmap(target);
                        gl::BindTexture(target, 0);
                    }
                } else {
                    warn!("GenerateMipmapByResource: resource {:?} not found", id);
                }
            }

            RenderCommand::UpdateTexture1DDataByResource {
                id,
                width,
                internal_format,
                pixel_format,
                data_format,
                data,
            } => {
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
                        gl::TexParameteri(
                            gl::TEXTURE_1D,
                            gl::TEXTURE_MIN_FILTER,
                            gl::NEAREST as i32,
                        );
                        gl::TexParameteri(
                            gl::TEXTURE_1D,
                            gl::TEXTURE_MAG_FILTER,
                            gl::NEAREST as i32,
                        );
                        gl::TexParameteri(
                            gl::TEXTURE_1D,
                            gl::TEXTURE_WRAP_S,
                            gl::CLAMP_TO_EDGE as i32,
                        );
                        gl::BindTexture(gl::TEXTURE_1D, 0);
                    }
                } else {
                    warn!("UpdateTexture1DDataByResource: resource {:?} not found", id);
                }
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
                        gl::TexParameteri(
                            gl::TEXTURE_3D,
                            gl::TEXTURE_MIN_FILTER,
                            gl::NEAREST as i32,
                        );
                        gl::TexParameteri(
                            gl::TEXTURE_3D,
                            gl::TEXTURE_MAG_FILTER,
                            gl::NEAREST as i32,
                        );
                        gl::TexParameteri(
                            gl::TEXTURE_3D,
                            gl::TEXTURE_WRAP_S,
                            gl::CLAMP_TO_EDGE as i32,
                        );
                        gl::TexParameteri(
                            gl::TEXTURE_3D,
                            gl::TEXTURE_WRAP_T,
                            gl::CLAMP_TO_EDGE as i32,
                        );
                        gl::TexParameteri(
                            gl::TEXTURE_3D,
                            gl::TEXTURE_WRAP_R,
                            gl::CLAMP_TO_EDGE as i32,
                        );
                        gl::BindTexture(gl::TEXTURE_3D, 0);
                    }
                } else {
                    warn!("UpdateTexture3DDataByResource: resource {:?} not found", id);
                }
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

            RenderCommand::CopyTexture2DFromFramebufferByResource {
                id,
                internal_format,
                width,
                height,
            } => {
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
                        gl::BindTexture(gl::TEXTURE_2D, 0);
                    }
                } else {
                    warn!(
                        "CopyTexture2DFromFramebufferByResource: resource {:?} not found",
                        id
                    );
                }
            }

            RenderCommand::ReadTexture1DData {
                id,
                pixel_format,
                data_format,
                reply_tx,
            } => {
                let mut data = Vec::new();
                if let Some(GpuResource::Texture1D { handle }) = self.resources.get(&id) {
                    unsafe {
                        let mut width = 0;
                        gl::BindTexture(gl::TEXTURE_1D, *handle);
                        gl::GetTexLevelParameteriv(
                            gl::TEXTURE_1D,
                            0,
                            gl::TEXTURE_WIDTH,
                            &mut width,
                        );
                        data = vec![
                            0u8;
                            self.texel_buffer_size(width, 1, 1, pixel_format, data_format)
                        ];
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

            RenderCommand::ReadTexture2DData {
                id,
                pixel_format,
                data_format,
                reply_tx,
            } => {
                let mut data = Vec::new();
                if let Some(GpuResource::Texture2D { handle }) = self.resources.get(&id) {
                    unsafe {
                        let (mut width, mut height) = (0, 0);
                        gl::BindTexture(gl::TEXTURE_2D, *handle);
                        gl::GetTexLevelParameteriv(
                            gl::TEXTURE_2D,
                            0,
                            gl::TEXTURE_WIDTH,
                            &mut width,
                        );
                        gl::GetTexLevelParameteriv(
                            gl::TEXTURE_2D,
                            0,
                            gl::TEXTURE_HEIGHT,
                            &mut height,
                        );
                        data =
                            vec![
                                0u8;
                                self.texel_buffer_size(width, height, 1, pixel_format, data_format)
                            ];
                        gl::GetTexImage(
                            gl::TEXTURE_2D,
                            0,
                            pixel_format,
                            data_format,
                            data.as_mut_ptr() as *mut _,
                        );
                        gl::BindTexture(gl::TEXTURE_2D, 0);
                    }
                } else {
                    warn!("ReadTexture2DData: resource {:?} not found", id);
                }
                let _ = reply_tx.send(data);
            }

            RenderCommand::ReadTexture3DData {
                id,
                pixel_format,
                data_format,
                reply_tx,
            } => {
                let mut data = Vec::new();
                if let Some(GpuResource::Texture3D { handle }) = self.resources.get(&id) {
                    unsafe {
                        let (mut width, mut height, mut depth) = (0, 0, 0);
                        gl::BindTexture(gl::TEXTURE_3D, *handle);
                        gl::GetTexLevelParameteriv(
                            gl::TEXTURE_3D,
                            0,
                            gl::TEXTURE_WIDTH,
                            &mut width,
                        );
                        gl::GetTexLevelParameteriv(
                            gl::TEXTURE_3D,
                            0,
                            gl::TEXTURE_HEIGHT,
                            &mut height,
                        );
                        gl::GetTexLevelParameteriv(
                            gl::TEXTURE_3D,
                            0,
                            gl::TEXTURE_DEPTH,
                            &mut depth,
                        );
                        data = vec![
                            0u8;
                            self.texel_buffer_size(
                                width,
                                height,
                                depth,
                                pixel_format,
                                data_format
                            )
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

            RenderCommand::ReadTextureCubeFaceData {
                id,
                face,
                level,
                pixel_format,
                data_format,
                reply_tx,
            } => {
                let mut data = Vec::new();
                if let Some(GpuResource::TextureCube { handle }) = self.resources.get(&id) {
                    unsafe {
                        let mut size = 0;
                        gl::BindTexture(gl::TEXTURE_CUBE_MAP, *handle);
                        gl::GetTexLevelParameteriv(face, level, gl::TEXTURE_WIDTH, &mut size);
                        data = vec![
                            0u8;
                            self.texel_buffer_size(size, size, 1, pixel_format, data_format)
                        ];
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

            RenderCommand::SamplePixel2DByResource { id, x, y, reply_tx } => {
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

            RenderCommand::ReadFramebufferPixels {
                x,
                y,
                width,
                height,
                reply_tx,
            } => {
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

            // === Framebuffer Operations ===
            RenderCommand::PushFramebuffer {
                id: _,
                width: _,
                height: _,
            } => {
                self.push_framebuffer();
            }

            RenderCommand::PopFramebuffer => {
                self.pop_framebuffer();
            }

            RenderCommand::FramebufferAttachTexture2D {
                attachment,
                texture,
                level,
            } => unsafe {
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
            },

            RenderCommand::FramebufferAttachTexture2DByResource {
                attachment,
                id,
                level,
            } => {
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

            RenderCommand::FramebufferAttachTexture3D {
                attachment,
                texture,
                layer,
                level,
            } => unsafe {
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
            },

            RenderCommand::FramebufferAttachTexture3DByResource {
                attachment,
                id,
                layer,
                level,
            } => {
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

            RenderCommand::FramebufferAttachTextureCube {
                attachment,
                texture,
                face,
                level,
            } => unsafe {
                gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachment, face, texture.0, level);
                if let Some(fbo) = self.fbo_stack.last_mut() {
                    fbo.color_index = (attachment - gl::COLOR_ATTACHMENT0 + 1) as i32;
                    gl::DrawBuffers(fbo.color_index, DRAW_BUFS.as_ptr());
                }
            },

            RenderCommand::FramebufferAttachTextureCubeByResource {
                attachment,
                id,
                face,
                level,
            } => {
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

            RenderCommand::SetDrawBuffers { count } => unsafe {
                gl::DrawBuffers(count, DRAW_BUFS.as_ptr());
            },

            RenderCommand::BindFramebuffer { handle } => unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, handle.0);
            },

            RenderCommand::BindDefaultFramebuffer => unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            },

            RenderCommand::Clear { color, depth } => unsafe {
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
            },

            // === Mesh Operations ===
            RenderCommand::BindMesh { vao } => unsafe {
                gl::BindVertexArray(vao.0);
                gl::EnableVertexAttribArray(0);
                gl::EnableVertexAttribArray(1);
                gl::EnableVertexAttribArray(2);
            },

            RenderCommand::UnbindMesh => unsafe {
                gl::DisableVertexAttribArray(0);
                gl::DisableVertexAttribArray(1);
                gl::DisableVertexAttribArray(2);
                gl::BindVertexArray(0);
            },

            // === Drawing Operations ===
            RenderCommand::DrawMesh {
                vao,
                index_count,
                primitive,
            } => unsafe {
                gl::BindVertexArray(vao.0);
                gl::DrawElements(
                    primitive.to_gl(),
                    index_count,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
                gl::BindVertexArray(0);
            },

            RenderCommand::DrawMeshInstanced {
                vao,
                index_count,
                instance_count,
                primitive,
            } => unsafe {
                gl::BindVertexArray(vao.0);
                gl::DrawElementsInstanced(
                    primitive.to_gl(),
                    index_count,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                    instance_count,
                );
                gl::BindVertexArray(0);
            },

            RenderCommand::DrawMeshByResource {
                id,
                index_count,
                primitive,
            } => {
                if let Some(GpuResource::Mesh { vao, .. }) = self.resources.get(&id) {
                    unsafe {
                        gl::BindVertexArray(*vao);
                        gl::DrawElements(
                            primitive.to_gl(),
                            index_count,
                            gl::UNSIGNED_INT,
                            ptr::null(),
                        );
                        gl::BindVertexArray(0);
                    }
                } else {
                    warn!("DrawMeshByResource: resource {:?} not found", id);
                }
            }

            RenderCommand::DrawMeshInstancedByResource {
                id,
                index_count,
                instance_count,
                primitive,
            } => {
                if let Some(GpuResource::Mesh { vao, .. }) = self.resources.get(&id) {
                    unsafe {
                        gl::BindVertexArray(*vao);
                        gl::DrawElementsInstanced(
                            primitive.to_gl(),
                            index_count,
                            gl::UNSIGNED_INT,
                            ptr::null(),
                            instance_count,
                        );
                        gl::BindVertexArray(0);
                    }
                } else {
                    warn!("DrawMeshInstancedByResource: resource {:?} not found", id);
                }
            }

            RenderCommand::DrawInstancedWithData {
                mesh_id,
                index_count,
                instances,
                primitive,
            } => {
                if instances.is_empty() {
                    return reply; // Nothing to draw
                }

                let mesh_vao =
                    if let Some(GpuResource::Mesh { vao, .. }) = self.resources.get(&mesh_id) {
                        *vao
                    } else {
                        warn!(
                            "DrawInstancedWithData: mesh resource {:?} not found",
                            mesh_id
                        );
                        return reply;
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
                            ptr::null(),
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

                    // Set up instance attributes (model matrix as 4 vec4 columns + color)
                    // InstanceData layout: model_matrix[16] + color[4] = 80 bytes
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

                    // Draw instanced
                    gl::DrawElementsInstanced(
                        primitive.to_gl(),
                        index_count,
                        gl::UNSIGNED_INT,
                        ptr::null(),
                        instance_count as i32,
                    );

                    // Disable instance attributes and reset divisors
                    for attrib in 4..=8 {
                        gl::VertexAttribDivisor(attrib, 0);
                        gl::DisableVertexAttribArray(attrib);
                    }

                    gl::BindVertexArray(0);
                    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                }
                // Note: draw call counting is handled by is_draw_call() in execute()
            }

            RenderCommand::BindMeshByResource { id } => {
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

            RenderCommand::DrawImmediate {
                primitive,
                vertices,
            } => {
                self.draw_immediate(primitive, &vertices);
            }

            // === Resource Creation ===
            RenderCommand::CreateShader {
                id,
                vertex_src,
                fragment_src,
            } => match self.create_shader(&vertex_src, &fragment_src) {
                Ok(program) => {
                    self.resources.insert(id, GpuResource::Shader { program });
                    debug!("Created shader {:?} with program {}", id, program);
                }
                Err(e) => {
                    error!("Failed to create shader {:?}: {}", id, e);
                }
            },

            RenderCommand::ReloadShader {
                shader_key,
                vertex_src,
                fragment_src,
            } => {
                // Compile shader on render thread and send result back
                let result = match self.create_shader(&vertex_src, &fragment_src) {
                    Ok(program) => {
                        // Delete old hot-reloaded shader if exists
                        if let Some(old_program) = self.hot_reloaded_shaders.remove(&shader_key) {
                            // Clear uniform cache for the old program to prevent stale lookups
                            // (GL may reuse the program ID for a new shader)
                            self.uniform_caches.remove(&old_program);
                            unsafe {
                                gl::DeleteProgram(old_program);
                            }
                            debug!("Deleted previous hot-reloaded shader for '{}'", shader_key);
                        }

                        // Store the new program for this shader_key
                        self.hot_reloaded_shaders
                            .insert(shader_key.clone(), program);
                        info!(
                            "Shader '{}' reloaded successfully on render thread (program={})",
                            shader_key, program
                        );

                        ShaderReloadResult {
                            shader_key,
                            success: true,
                            error: None,
                            program,
                        }
                    }
                    Err(e) => {
                        warn!("Shader '{}' reload failed: {}", shader_key, e);
                        // Push error to global queue for UI overlay
                        // push_shader_error(&shader_key, "compile", &e);
                        ShaderReloadResult {
                            shader_key,
                            success: false,
                            error: Some(e),
                            program: 0,
                        }
                    }
                };
                reply = CommandReply::ShaderReload(result);
            }

            RenderCommand::CreateTexture1D {
                id,
                width,
                format,
                data,
            } => {
                let handle = self.create_texture_1d(width, format, data.as_deref());
                self.resources.insert(id, GpuResource::Texture1D { handle });
                debug!("Created texture1d {:?} with handle {}", id, handle);
            }

            RenderCommand::CreateTexture2D {
                id,
                width,
                height,
                format,
                data,
            } => {
                let handle = self.create_texture_2d(width, height, format, data.as_deref());
                self.resources.insert(id, GpuResource::Texture2D { handle });
                debug!("Created texture2d {:?} with handle {}", id, handle);
            }

            RenderCommand::CreateTexture3D {
                id,
                width,
                height,
                depth,
                format,
                data,
            } => {
                let handle = self.create_texture_3d(width, height, depth, format, data.as_deref());
                self.resources.insert(id, GpuResource::Texture3D { handle });
                debug!("Created texture3d {:?} with handle {}", id, handle);
            }

            RenderCommand::CreateTextureCube { id, size, format } => {
                let handle = self.create_texture_cube(size, format);
                self.resources
                    .insert(id, GpuResource::TextureCube { handle });
                debug!("Created texturecube {:?} with handle {}", id, handle);
            }

            RenderCommand::CreateMesh {
                id,
                vertices,
                indices,
                vertex_format,
            } => {
                let (vao, vbo, ebo) = self.create_mesh(&vertices, &indices, &vertex_format);
                self.resources
                    .insert(id, GpuResource::Mesh { vao, vbo, ebo });
                debug!("Created mesh {:?} with vao {}", id, vao);
            }

            RenderCommand::DestroyResource { id } => {
                if let Some(resource) = self.resources.remove(&id) {
                    self.destroy_resource(resource);
                    debug!("Destroyed resource {:?}", id);
                }
            }

            // === Uniform Buffer Objects ===
            RenderCommand::CreateCameraUBO => {
                self.create_camera_ubo();
            }

            RenderCommand::UpdateCameraUBO { data } => {
                self.update_camera_ubo(&data);
            }

            RenderCommand::CreateMaterialUBO => {
                self.create_material_ubo();
            }

            RenderCommand::UpdateMaterialUBO { data } => {
                self.update_material_ubo(&data);
            }

            RenderCommand::CreateLightUBO => {
                self.create_light_ubo();
            }

            RenderCommand::UpdateLightUBO { data } => {
                self.update_light_ubo(&data);
            }

            // === Window Operations ===
            RenderCommand::Resize { width, height } => {
                // Only update the viewport - surface resize is handled by the window system.
                // Note: Calling ctx.resize() here was causing freezes during window resize,
                // likely due to synchronization issues with the window manager.
                // The viewport update is sufficient for correct rendering.
                unsafe {
                    gl::Viewport(0, 0, width as i32, height as i32);
                }
            }

            RenderCommand::SwapBuffers => {
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
                    texture_binds_skipped: self.texture_binds_skipped,
                };
                reply = CommandReply::Stats(self.last_stats.clone());

                // Perform actual buffer swap if we have a GL context
                if let Some(ref ctx) = self.gl_context {
                    if let Err(e) = ctx.swap_buffers() {
                        error!("Failed to swap buffers: {}", e);
                    }
                } else if self.stats.frame_count == 1 {
                    error!("SwapBuffers: no GL context available!");
                }

                // Reset per-frame counters and start new frame timing
                self.commands_this_frame = 0;
                self.draw_calls_this_frame = 0;
                self.frame_start = std::time::Instant::now();
            }

            // === Synchronization ===
            RenderCommand::Flush => unsafe {
                gl::Finish();
            },

            RenderCommand::Fence { fence_id } => {
                reply = CommandReply::Fence(fence_id);
            }

            RenderCommand::Shutdown => {
                // Handled by the caller's loop
            }
        }

        reply
    }

    fn set_blend_mode(&self, mode: BlendMode) {
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

    fn set_cull_face(&self, face: CullFace) {
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

    fn draw_immediate(&self, primitive: CmdPrimitiveType, vertices: &[ImmVertex]) {
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
    }

    fn create_shader(&self, vertex_src: &str, fragment_src: &str) -> Result<u32, String> {
        unsafe {
            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            let vs_src = std::ffi::CString::new(vertex_src).unwrap();
            gl::ShaderSource(vs, 1, &vs_src.as_ptr(), ptr::null());
            gl::CompileShader(vs);

            let mut success = 0;
            gl::GetShaderiv(vs, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetShaderiv(vs, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl::GetShaderInfoLog(vs, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut _);
                gl::DeleteShader(vs);
                return Err(format!(
                    "Vertex shader error: {}",
                    String::from_utf8_lossy(&buffer)
                ));
            }

            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            let fs_src = std::ffi::CString::new(fragment_src).unwrap();
            gl::ShaderSource(fs, 1, &fs_src.as_ptr(), ptr::null());
            gl::CompileShader(fs);

            gl::GetShaderiv(fs, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetShaderiv(fs, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl::GetShaderInfoLog(fs, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut _);
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

            gl::LinkProgram(program);

            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl::GetProgramInfoLog(program, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut _);
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
                data.map_or(ptr::null(), |d| d.as_ptr() as *const _),
            );

            // Use NEAREST filtering to match direct mode behavior (important for fonts/crisp textures)
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            gl::BindTexture(gl::TEXTURE_2D, 0);
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
                data.map_or(ptr::null(), |d| d.as_ptr() as *const _),
            );

            gl::TexParameteri(gl::TEXTURE_1D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_1D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_1D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);

            gl::BindTexture(gl::TEXTURE_1D, 0);
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
                data.map_or(ptr::null(), |d| d.as_ptr() as *const _),
            );

            gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);

            gl::BindTexture(gl::TEXTURE_3D, 0);
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
                    ptr::null(),
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

            gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);
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
                GpuResource::Framebuffer { fbo } => {
                    gl::DeleteFramebuffers(1, &fbo);
                }
            }
        }
    }

    /// Create the camera UBO (binding point 0)
    fn create_camera_ubo(&mut self) {
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
    fn update_camera_ubo(&mut self, data: &[u8; 288]) {
        if self.camera_ubo == 0 {
            self.create_camera_ubo();
        }

        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.camera_ubo);
            gl::BufferSubData(gl::UNIFORM_BUFFER, 0, 288, data.as_ptr() as *const _);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }

    /// Create material UBO
    fn create_material_ubo(&mut self) {
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
    fn update_material_ubo(&mut self, data: &[u8; 32]) {
        if self.material_ubo == 0 {
            self.create_material_ubo();
        }

        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.material_ubo);
            gl::BufferSubData(gl::UNIFORM_BUFFER, 0, 32, data.as_ptr() as *const _);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }

    /// Create light UBO
    fn create_light_ubo(&mut self) {
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
    fn update_light_ubo(&mut self, data: &[u8; 32]) {
        if self.light_ubo == 0 {
            self.create_light_ubo();
        }

        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.light_ubo);
            gl::BufferSubData(gl::UNIFORM_BUFFER, 0, 32, data.as_ptr() as *const _);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }

    /// Push a new framebuffer onto the FBO stack
    fn push_framebuffer(&mut self) {
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
    fn pop_framebuffer(&mut self) {
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
    pub fn cleanup(&mut self) -> Option<WindowGlContext> {
        info!(
            "Cleaning up render thread resources ({} resources to clean)",
            self.resources.len()
        );

        // Destroy all remaining resources
        let resources: Vec<_> = self.resources.drain().collect();
        for (id, resource) in resources {
            debug!("Cleaning up resource {:?}", id);
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
        // RenderThreadGlContext::release_for_main_thread() handles:
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
}
