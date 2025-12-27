//! Dedicated render thread that owns the OpenGL context.
//!
//! The render thread receives commands via a channel and executes them.
//! This allows the main thread to run game logic without blocking on GL calls.

#![allow(unsafe_code)]

use std::collections::HashMap;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crossbeam::channel::{Receiver, Sender, bounded};
use glutin::context::NotCurrentContext;
use glutin::surface::{Surface, WindowSurface};
use tracing::{debug, error, info, warn};

use super::{gl, BlendMode, CullFace, RenderCommand, ResourceId, CmdPrimitiveType, ImmVertex, InstanceData, VertexFormat, push_shader_error};

/// Maximum number of texture units to track for caching
/// OpenGL requires at least 16, most GPUs support 32+
const MAX_TEXTURE_SLOTS: usize = 16;

/// Texture type for binding cache
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextureType {
    Texture2D,
    Texture3D,
    TextureCube,
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

/// GPU resource stored on the render thread
#[derive(Debug)]
pub enum GpuResource {
    Shader { program: u32 },
    Texture2D { handle: u32 },
    Texture3D { handle: u32 },
    TextureCube { handle: u32 },
    Mesh { vao: u32, vbo: u32, ebo: u32 },
    Framebuffer { fbo: u32 },
}

/// Statistics from the render thread (local copy)
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    pub commands_processed: u64,
    pub draw_calls: u64,
    pub state_changes: u64,
    pub frame_count: u64,
}

/// Shared statistics accessible from main thread
#[derive(Debug)]
pub struct SharedRenderStats {
    pub commands_processed: AtomicU64,
    pub draw_calls: AtomicU64,
    pub state_changes: AtomicU64,
    pub frame_count: AtomicU64,
    /// Last frame render time in microseconds
    pub last_frame_time_us: AtomicU64,
    /// Commands processed in the last frame
    pub commands_last_frame: AtomicU64,
    /// Draw calls in the last frame
    pub draw_calls_last_frame: AtomicU64,
    /// Texture binds skipped due to caching (cumulative)
    pub texture_binds_skipped: AtomicU64,
    /// Main thread wait time in microseconds (time spent waiting for render thread)
    pub main_thread_wait_us: AtomicU64,
}

impl Default for SharedRenderStats {
    fn default() -> Self {
        Self {
            commands_processed: AtomicU64::new(0),
            draw_calls: AtomicU64::new(0),
            state_changes: AtomicU64::new(0),
            frame_count: AtomicU64::new(0),
            last_frame_time_us: AtomicU64::new(0),
            commands_last_frame: AtomicU64::new(0),
            draw_calls_last_frame: AtomicU64::new(0),
            texture_binds_skipped: AtomicU64::new(0),
            main_thread_wait_us: AtomicU64::new(0),
        }
    }
}

impl SharedRenderStats {
    pub fn snapshot(&self) -> RenderStats {
        RenderStats {
            commands_processed: self.commands_processed.load(Ordering::Relaxed),
            draw_calls: self.draw_calls.load(Ordering::Relaxed),
            state_changes: self.state_changes.load(Ordering::Relaxed),
            frame_count: self.frame_count.load(Ordering::Relaxed),
        }
    }
}

/// Result of a shader reload operation
#[derive(Debug, Clone)]
pub struct ShaderReloadResult {
    /// The shader key that was reloaded
    pub shader_key: String,
    /// Whether compilation succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// The new program handle (if success)
    pub program: u32,
}

/// Configuration for the render thread
#[derive(Debug, Clone)]
pub struct RenderThreadConfig {
    /// Channel capacity for commands
    pub command_buffer_size: usize,
    /// Channel capacity for fence responses
    pub fence_buffer_size: usize,
}

impl Default for RenderThreadConfig {
    fn default() -> Self {
        Self {
            // Buffer for ~2-3 frames worth of commands
            command_buffer_size: 8192,
            fence_buffer_size: 64,
        }
    }
}

/// Data returned from the render thread when shutting down
pub struct ReturnedGlContext {
    /// The GL context (now not current)
    pub context: NotCurrentContext,
    /// The surface
    pub surface: Surface<WindowSurface>,
}

/// Maximum frames in flight for triple buffering
const MAX_FRAMES_IN_FLIGHT: u64 = 3;

/// Handle to communicate with the render thread
pub struct RenderThreadHandle {
    /// Send commands to the render thread
    command_tx: Sender<RenderCommand>,
    /// Receive fence completions from the render thread
    fence_rx: Receiver<u64>,
    /// Receive shader reload results from the render thread
    shader_result_rx: Receiver<ShaderReloadResult>,
    /// Receive returned GL context when render thread shuts down
    context_rx: Receiver<Option<ReturnedGlContext>>,
    /// Next fence ID to use
    next_fence_id: AtomicU64,
    /// Number of frames currently in flight (submitted but not rendered)
    frames_in_flight: AtomicU64,
    /// Whether the render thread is running
    running: Arc<AtomicBool>,
    /// Thread handle for joining
    thread_handle: Option<JoinHandle<()>>,
    /// Shared stats readable from main thread
    shared_stats: Arc<SharedRenderStats>,
}

impl RenderThreadHandle {
    /// Submit a command to the render thread
    pub fn submit(&self, cmd: RenderCommand) {
        if self.running.load(Ordering::Relaxed) {
            if let Err(e) = self.command_tx.send(cmd) {
                error!("Failed to send render command: {:?}", e);
            }
        }
    }

    /// Submit a command to the render thread without blocking.
    /// Returns true if the command was sent, false if the channel was full.
    /// Use this for commands that can be safely dropped (like resize events).
    pub fn try_submit(&self, cmd: RenderCommand) -> bool {
        if self.running.load(Ordering::Relaxed) {
            match self.command_tx.try_send(cmd) {
                Ok(()) => true,
                Err(crossbeam::channel::TrySendError::Full(_)) => {
                    // Channel full, command dropped (will be retried next frame)
                    false
                }
                Err(crossbeam::channel::TrySendError::Disconnected(_)) => {
                    error!("Render thread disconnected");
                    false
                }
            }
        } else {
            false
        }
    }

    /// Submit multiple commands efficiently
    pub fn submit_batch(&self, commands: impl IntoIterator<Item = RenderCommand>) {
        if self.running.load(Ordering::Relaxed) {
            for cmd in commands {
                if let Err(e) = self.command_tx.send(cmd) {
                    error!("Failed to send render command: {:?}", e);
                    break;
                }
            }
        }
    }

    /// Wait for the render thread to process all commands up to this point
    pub fn sync(&self) -> bool {
        if !self.running.load(Ordering::Relaxed) {
            return false;
        }

        let fence_id = self.next_fence_id.fetch_add(1, Ordering::Relaxed);
        self.submit(RenderCommand::Fence { fence_id });

        // Wait for the fence to be signaled
        loop {
            match self.fence_rx.recv() {
                Ok(id) if id == fence_id => return true,
                Ok(_) => continue, // Not our fence, keep waiting
                Err(_) => return false, // Channel closed
            }
        }
    }

    /// End the current frame with triple buffering.
    /// Submits SwapBuffers and fence, blocks only if MAX_FRAMES_IN_FLIGHT are queued.
    /// Uses fence channel for proper synchronization when throttling is needed.
    pub fn end_frame_triple_buffered(&self) {
        if !self.running.load(Ordering::Relaxed) {
            return;
        }

        // Track ALL time spent in this function (includes channel blocking)
        let frame_end_start = std::time::Instant::now();

        // Drain completed fences (non-blocking) to update in-flight count
        while let Ok(_) = self.fence_rx.try_recv() {
            self.frames_in_flight.fetch_sub(1, Ordering::Relaxed);
        }

        // If at limit, block waiting for one frame to complete
        while self.frames_in_flight.load(Ordering::Relaxed) >= MAX_FRAMES_IN_FLIGHT {
            match self.fence_rx.recv() {
                Ok(_) => {
                    self.frames_in_flight.fetch_sub(1, Ordering::Relaxed);
                }
                Err(_) => return, // Channel closed
            }
        }

        // Submit SwapBuffers + fence to track this frame
        // Note: submit() can also block if command channel is full!
        self.submit(RenderCommand::SwapBuffers);
        let fence_id = self.next_fence_id.fetch_add(1, Ordering::Relaxed);
        self.submit(RenderCommand::Fence { fence_id });
        self.frames_in_flight.fetch_add(1, Ordering::Relaxed);

        // Store total time spent in frame end (all blocking)
        let wait_us = frame_end_start.elapsed().as_micros() as u64;
        self.shared_stats.main_thread_wait_us.store(wait_us, Ordering::Relaxed);
    }

    /// Get current frames in flight count
    pub fn get_frames_in_flight(&self) -> u64 {
        self.frames_in_flight.load(Ordering::Relaxed)
    }

    /// Check if the render thread is still running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Get current render stats snapshot
    pub fn get_stats(&self) -> RenderStats {
        self.shared_stats.snapshot()
    }

    /// Get total commands processed since start
    pub fn get_commands_processed(&self) -> u64 {
        self.shared_stats.commands_processed.load(Ordering::Relaxed)
    }

    /// Get total draw calls since start
    pub fn get_draw_calls(&self) -> u64 {
        self.shared_stats.draw_calls.load(Ordering::Relaxed)
    }

    /// Get total state changes since start
    pub fn get_state_changes(&self) -> u64 {
        self.shared_stats.state_changes.load(Ordering::Relaxed)
    }

    /// Get total frames rendered
    pub fn get_frame_count(&self) -> u64 {
        self.shared_stats.frame_count.load(Ordering::Relaxed)
    }

    /// Get last frame render time in microseconds
    pub fn get_last_frame_time_us(&self) -> u64 {
        self.shared_stats.last_frame_time_us.load(Ordering::Relaxed)
    }

    /// Get commands processed in last frame
    pub fn get_commands_last_frame(&self) -> u64 {
        self.shared_stats.commands_last_frame.load(Ordering::Relaxed)
    }

    /// Get draw calls in last frame
    pub fn get_draw_calls_last_frame(&self) -> u64 {
        self.shared_stats.draw_calls_last_frame.load(Ordering::Relaxed)
    }

    /// Get main thread wait time in microseconds (time spent waiting for render thread)
    pub fn get_main_thread_wait_us(&self) -> u64 {
        self.shared_stats.main_thread_wait_us.load(Ordering::Relaxed)
    }

    /// Get total texture binds skipped due to caching
    pub fn get_texture_binds_skipped(&self) -> u64 {
        self.shared_stats.texture_binds_skipped.load(Ordering::Relaxed)
    }

    /// Reload a shader on the render thread.
    /// Returns the result with success/failure and new program handle.
    /// This blocks until the shader is compiled on the render thread.
    pub fn reload_shader(
        &self,
        shader_key: &str,
        vertex_src: &str,
        fragment_src: &str,
    ) -> ShaderReloadResult {
        if !self.running.load(Ordering::Relaxed) {
            return ShaderReloadResult {
                shader_key: shader_key.to_string(),
                success: false,
                error: Some("Render thread not running".to_string()),
                program: 0,
            };
        }

        // Send the reload command
        self.submit(RenderCommand::ReloadShader {
            shader_key: shader_key.to_string(),
            vertex_src: vertex_src.to_string(),
            fragment_src: fragment_src.to_string(),
        });

        // Wait for the result (blocking)
        match self.shader_result_rx.recv() {
            Ok(result) => result,
            Err(_) => ShaderReloadResult {
                shader_key: shader_key.to_string(),
                success: false,
                error: Some("Channel closed while waiting for shader result".to_string()),
                program: 0,
            },
        }
    }

    /// Request the render thread to shutdown
    pub fn shutdown(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            info!("Requesting render thread shutdown");
            self.submit(RenderCommand::Shutdown);
            self.running.store(false, Ordering::Relaxed);

            // Wait for thread to finish
            if let Some(handle) = self.thread_handle.take() {
                if let Err(e) = handle.join() {
                    error!("Render thread panicked: {:?}", e);
                }
            }
        }
    }

    /// Get the GL context returned from the render thread after shutdown (non-blocking).
    /// This should be called after shutdown() to retrieve the context for
    /// restoring direct GL mode on the main thread.
    pub fn take_returned_context(&self) -> Option<ReturnedGlContext> {
        // Try to receive the context (non-blocking since shutdown already waited)
        match self.context_rx.try_recv() {
            Ok(ctx) => ctx,
            Err(_) => None,
        }
    }

    /// Wait for the GL context to be returned from the render thread (blocking with timeout).
    /// This should be called after shutdown() to retrieve the context for
    /// restoring direct GL mode on the main thread.
    pub fn wait_for_returned_context(&self) -> Option<ReturnedGlContext> {
        use std::time::Duration;

        // Wait up to 5 seconds for the context to be returned
        match self.context_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(ctx) => ctx,
            Err(e) => {
                error!("Timeout or error waiting for GL context return: {:?}", e);
                None
            }
        }
    }
}

impl Drop for RenderThreadHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
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

/// The render thread state - runs on a dedicated thread
struct RenderThread {
    command_rx: Receiver<RenderCommand>,
    fence_tx: Sender<u64>,
    /// Channel to send shader reload results back to main thread
    shader_result_tx: Sender<ShaderReloadResult>,
    /// Channel to return GL context to main thread on shutdown
    context_tx: Sender<Option<ReturnedGlContext>>,
    resources: HashMap<ResourceId, GpuResource>,
    /// Hot-reloaded shaders by shader_key (separate from resources for override)
    hot_reloaded_shaders: HashMap<String, u32>,
    running: Arc<AtomicBool>,
    stats: RenderStats,
    /// Shared stats accessible from main thread
    shared_stats: Arc<SharedRenderStats>,
    // Immediate mode VAO/VBO for DrawImmediate commands
    imm_vao: u32,
    imm_vbo: u32,
    // FBO stack for push/pop framebuffer operations
    fbo_stack: Vec<FboEntry>,
    // GL context for buffer swapping (stored here to allow access during execute)
    gl_context: Option<crate::window::RenderThreadGlContext>,
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

impl RenderThread {
    fn new(
        command_rx: Receiver<RenderCommand>,
        fence_tx: Sender<u64>,
        shader_result_tx: Sender<ShaderReloadResult>,
        context_tx: Sender<Option<ReturnedGlContext>>,
        running: Arc<AtomicBool>,
        shared_stats: Arc<SharedRenderStats>,
        gl_context: Option<crate::window::RenderThreadGlContext>,
    ) -> Self {
        Self {
            command_rx,
            fence_tx,
            shader_result_tx,
            context_tx,
            resources: HashMap::new(),
            hot_reloaded_shaders: HashMap::new(),
            running,
            stats: RenderStats::default(),
            shared_stats,
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

    /// Bind a texture with caching - returns true if actually bound, false if skipped.
    /// This avoids redundant glBindTexture calls which can be expensive.
    fn bind_texture_cached(&mut self, slot: u32, handle: u32, tex_type: TextureType) -> bool {
        let slot_idx = slot as usize;
        if slot_idx >= MAX_TEXTURE_SLOTS {
            // Slot out of range, just bind directly
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + slot);
                let target = match tex_type {
                    TextureType::Texture2D => gl::TEXTURE_2D,
                    TextureType::Texture3D => gl::TEXTURE_3D,
                    TextureType::TextureCube => gl::TEXTURE_CUBE_MAP,
                };
                gl::BindTexture(target, handle);
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
            let target = match tex_type {
                TextureType::Texture2D => gl::TEXTURE_2D,
                TextureType::Texture3D => gl::TEXTURE_3D,
                TextureType::TextureCube => gl::TEXTURE_CUBE_MAP,
            };
            gl::BindTexture(target, handle);
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
                    let target = match tex_type {
                        TextureType::Texture2D => gl::TEXTURE_2D,
                        TextureType::Texture3D => gl::TEXTURE_3D,
                        TextureType::TextureCube => gl::TEXTURE_CUBE_MAP,
                    };
                    gl::BindTexture(target, 0);
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
    fn init_gl(&mut self) {
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
            info!("Render thread GL state after reset: FBO={}, VAO={}, viewport={:?}",
                  current_fbo, current_vao, viewport);

            // Create VAO/VBO for immediate mode rendering
            gl::GenVertexArrays(1, &mut self.imm_vao);
            gl::GenBuffers(1, &mut self.imm_vbo);

            gl::BindVertexArray(self.imm_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.imm_vbo);

            // Setup vertex attributes for ImmVertex: pos (3f), normal (3f), uv (2f), color (4f)
            // Attribute locations must match shader.rs BindAttribLocation calls:
            //   0 = vertex_position, 1 = vertex_normal, 2 = vertex_uv, 3 = vertex_color
            let stride = std::mem::size_of::<ImmVertex>() as i32;  // 12 floats = 48 bytes

            // Position attribute (location 0 = vertex_position)
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());

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
    fn run(&mut self) {
        info!("Render thread started");

        // Only initialize GL resources if we have a valid context
        if self.gl_context.is_some() {
            self.init_gl();
        } else {
            warn!("Render thread running without GL context - commands will be no-ops");
        }

        while self.running.load(Ordering::Relaxed) {
            match self.command_rx.recv() {
                Ok(cmd) => {
                    if matches!(cmd, RenderCommand::Shutdown) {
                        info!("Render thread received shutdown command");
                        break;
                    }
                    self.execute(cmd);
                }
                Err(_) => {
                    debug!("Command channel closed, render thread exiting");
                    break;
                }
            }
        }

        self.cleanup();
        info!("Render thread stopped. Stats: {:?}", self.stats);
    }

    /// Execute a single render command
    fn execute(&mut self, cmd: RenderCommand) {
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
            RenderCommand::SetViewport { x, y, width, height } => unsafe {
                gl::Viewport(x, y, width, height);
            },

            RenderCommand::SetScissor { x, y, width, height } => unsafe {
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
            },

            RenderCommand::SetCullFace(face) => {
                self.set_cull_face(face);
            },

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
                    unsafe { gl::Uniform1i(loc, value); }
                }
            },

            RenderCommand::SetUniformInt2ByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe { gl::Uniform2i(loc, value[0], value[1]); }
                }
            },

            RenderCommand::SetUniformInt3ByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe { gl::Uniform3i(loc, value[0], value[1], value[2]); }
                }
            },

            RenderCommand::SetUniformInt4ByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe { gl::Uniform4i(loc, value[0], value[1], value[2], value[3]); }
                }
            },

            RenderCommand::SetUniformFloatByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe { gl::Uniform1f(loc, value); }
                }
            },

            RenderCommand::SetUniformFloat2ByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe { gl::Uniform2f(loc, value[0], value[1]); }
                }
            },

            RenderCommand::SetUniformFloat3ByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe { gl::Uniform3f(loc, value[0], value[1], value[2]); }
                }
            },

            RenderCommand::SetUniformFloat4ByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe { gl::Uniform4f(loc, value[0], value[1], value[2], value[3]); }
                }
            },

            RenderCommand::SetUniformMat4ByName { name, value } => {
                let loc = self.get_uniform_location_cached(name);
                if loc >= 0 {
                    unsafe { gl::UniformMatrix4fv(loc, 1, gl::FALSE, value.as_ptr()); }
                }
            },

            // === Texture Operations ===
            // Uses caching to skip redundant binds.
            // CRITICAL: After binding to a texture unit, we MUST reset ActiveTexture to TEXTURE0
            // to match direct mode behavior (see shader.rs apply_var). Without this reset,
            // subsequent GL operations that expect TEXTURE0 to be active will fail with
            // "unit 0 GLD_TEXTURE_INDEX_2D is unloadable" errors.
            RenderCommand::BindTexture2D { slot, handle } => {
                self.bind_texture_cached(slot, handle.0, TextureType::Texture2D);
            },

            RenderCommand::BindTexture2DByResource { slot, id } => {
                if let Some(GpuResource::Texture2D { handle }) = self.resources.get(&id) {
                    self.bind_texture_cached(slot, *handle, TextureType::Texture2D);
                } else {
                    warn!("BindTexture2DByResource: resource {:?} not found", id);
                }
            },

            RenderCommand::BindTexture3D { slot, handle } => {
                self.bind_texture_cached(slot, handle.0, TextureType::Texture3D);
            },

            RenderCommand::BindTextureCube { slot, handle } => {
                self.bind_texture_cached(slot, handle.0, TextureType::TextureCube);
            },

            RenderCommand::UnbindTexture { slot } => {
                self.unbind_texture_cached(slot);
            },

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

            RenderCommand::SetTexture2DMipRange { handle, min_level, max_level } => unsafe {
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
                        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                        gl::BindTexture(gl::TEXTURE_2D, 0);
                    }
                } else {
                    warn!("UpdateTexture2DDataByResource: resource {:?} not found", id);
                }
            },

            RenderCommand::SetTexture2DAnisotropy { handle, factor } => unsafe {
                gl::BindTexture(gl::TEXTURE_2D, handle.0);
                gl::TexParameterf(gl::TEXTURE_2D, gl::TEXTURE_MAX_ANISOTROPY_EXT, factor);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            },

            // === Framebuffer Operations ===
            RenderCommand::PushFramebuffer { id: _, width: _, height: _ } => {
                self.push_framebuffer();
            },

            RenderCommand::PopFramebuffer => {
                self.pop_framebuffer();
            },

            RenderCommand::FramebufferAttachTexture2D { attachment, texture, level } => unsafe {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    attachment,
                    gl::TEXTURE_2D,
                    texture.0,
                    level,
                );
                // Update color index if this is a color attachment
                if attachment >= gl::COLOR_ATTACHMENT0 && attachment <= gl::COLOR_ATTACHMENT3 {
                    if let Some(fbo) = self.fbo_stack.last_mut() {
                        fbo.color_index = (attachment - gl::COLOR_ATTACHMENT0 + 1) as i32;
                        gl::DrawBuffers(fbo.color_index, DRAW_BUFS.as_ptr());
                    }
                }
            },

            RenderCommand::FramebufferAttachTexture2DByResource { attachment, id, level } => {
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
                        if attachment >= gl::COLOR_ATTACHMENT0 && attachment <= gl::COLOR_ATTACHMENT3 {
                            if let Some(fbo) = self.fbo_stack.last_mut() {
                                fbo.color_index = (attachment - gl::COLOR_ATTACHMENT0 + 1) as i32;
                                gl::DrawBuffers(fbo.color_index, DRAW_BUFS.as_ptr());
                            }
                        }
                    }
                } else {
                    warn!("FramebufferAttachTexture2DByResource: resource {:?} not found", id);
                }
            },

            RenderCommand::FramebufferAttachTexture3D { attachment, texture, layer, level } => unsafe {
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

            RenderCommand::FramebufferAttachTextureCube { attachment, texture, face, level } => unsafe {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    attachment,
                    face,
                    texture.0,
                    level,
                );
                if let Some(fbo) = self.fbo_stack.last_mut() {
                    fbo.color_index = (attachment - gl::COLOR_ATTACHMENT0 + 1) as i32;
                    gl::DrawBuffers(fbo.color_index, DRAW_BUFS.as_ptr());
                }
            },

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
            RenderCommand::DrawMesh { vao, index_count, primitive } => unsafe {
                gl::BindVertexArray(vao.0);
                gl::DrawElements(
                    primitive.to_gl(),
                    index_count,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
                gl::BindVertexArray(0);
            },

            RenderCommand::DrawMeshInstanced { vao, index_count, instance_count, primitive } => unsafe {
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

            RenderCommand::DrawMeshByResource { id, index_count, primitive } => {
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
            },

            RenderCommand::DrawMeshInstancedByResource { id, index_count, instance_count, primitive } => {
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
            },

            RenderCommand::DrawInstancedWithData { mesh_id, index_count, instances, primitive } => {
                if instances.is_empty() {
                    return; // Nothing to draw
                }

                let mesh_vao = if let Some(GpuResource::Mesh { vao, .. }) = self.resources.get(&mesh_id) {
                    *vao
                } else {
                    warn!("DrawInstancedWithData: mesh resource {:?} not found", mesh_id);
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

                self.stats.draw_calls += 1;
                self.draw_calls_this_frame += 1;
            },

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
            },

            RenderCommand::DrawImmediate { primitive, vertices } => {
                self.draw_immediate(primitive, &vertices);
            },

            // === Resource Creation ===
            RenderCommand::CreateShader { id, vertex_src, fragment_src } => {
                match self.create_shader(&vertex_src, &fragment_src) {
                    Ok(program) => {
                        self.resources.insert(id, GpuResource::Shader { program });
                        debug!("Created shader {:?} with program {}", id, program);
                    }
                    Err(e) => {
                        error!("Failed to create shader {:?}: {}", id, e);
                    }
                }
            },

            RenderCommand::ReloadShader { shader_key, vertex_src, fragment_src } => {
                // Compile shader on render thread and send result back
                let result = match self.create_shader(&vertex_src, &fragment_src) {
                    Ok(program) => {
                        // Delete old hot-reloaded shader if exists
                        if let Some(old_program) = self.hot_reloaded_shaders.remove(&shader_key) {
                            unsafe { gl::DeleteProgram(old_program); }
                            debug!("Deleted previous hot-reloaded shader for '{}'", shader_key);
                        }

                        // Store the new program for this shader_key
                        self.hot_reloaded_shaders.insert(shader_key.clone(), program);
                        info!("Shader '{}' reloaded successfully on render thread (program={})", shader_key, program);

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
                        push_shader_error(&shader_key, "compile", &e);
                        ShaderReloadResult {
                            shader_key,
                            success: false,
                            error: Some(e),
                            program: 0,
                        }
                    }
                };
                if let Err(e) = self.shader_result_tx.send(result) {
                    error!("Failed to send shader reload result: {:?}", e);
                }
            },

            RenderCommand::CreateTexture2D { id, width, height, format, data } => {
                let handle = self.create_texture_2d(width, height, format, data.as_deref());
                self.resources.insert(id, GpuResource::Texture2D { handle });
                debug!("Created texture2d {:?} with handle {}", id, handle);
            },

            RenderCommand::CreateMesh { id, vertices, indices, vertex_format } => {
                let (vao, vbo, ebo) = self.create_mesh(&vertices, &indices, &vertex_format);
                self.resources.insert(id, GpuResource::Mesh { vao, vbo, ebo });
                debug!("Created mesh {:?} with vao {}", id, vao);
            },

            RenderCommand::DestroyResource { id } => {
                if let Some(resource) = self.resources.remove(&id) {
                    self.destroy_resource(resource);
                    debug!("Destroyed resource {:?}", id);
                }
            },

            // === Uniform Buffer Objects ===
            RenderCommand::CreateCameraUBO => {
                self.create_camera_ubo();
            },

            RenderCommand::UpdateCameraUBO { data } => {
                self.update_camera_ubo(&data);
            },

            RenderCommand::CreateMaterialUBO => {
                self.create_material_ubo();
            },

            RenderCommand::UpdateMaterialUBO { data } => {
                self.update_material_ubo(&data);
            },

            RenderCommand::CreateLightUBO => {
                self.create_light_ubo();
            },

            RenderCommand::UpdateLightUBO { data } => {
                self.update_light_ubo(&data);
            },

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

                // Sync all stats to shared atomics at end of frame
                self.shared_stats.commands_processed.store(self.stats.commands_processed, Ordering::Relaxed);
                self.shared_stats.draw_calls.store(self.stats.draw_calls, Ordering::Relaxed);
                self.shared_stats.state_changes.store(self.stats.state_changes, Ordering::Relaxed);
                self.shared_stats.frame_count.store(self.stats.frame_count, Ordering::Relaxed);
                self.shared_stats.last_frame_time_us.store(frame_time_us, Ordering::Relaxed);
                self.shared_stats.commands_last_frame.store(self.commands_this_frame, Ordering::Relaxed);
                self.shared_stats.draw_calls_last_frame.store(self.draw_calls_this_frame, Ordering::Relaxed);
                self.shared_stats.texture_binds_skipped.store(self.texture_binds_skipped, Ordering::Relaxed);

                // Perform actual buffer swap if we have a GL context
                if let Some(ref ctx) = self.gl_context {
                    if let Err(e) = ctx.swap_buffers() {
                        error!("Failed to swap buffers: {}", e);
                    }
                } else {
                    if self.stats.frame_count == 1 {
                        error!("SwapBuffers: no GL context available!");
                    }
                }

                // Reset per-frame counters and start new frame timing
                self.commands_this_frame = 0;
                self.draw_calls_this_frame = 0;
                self.frame_start = std::time::Instant::now();
            },

            // === Synchronization ===
            RenderCommand::Flush => unsafe {
                gl::Finish();
            },

            RenderCommand::Fence { fence_id } => {
                if let Err(e) = self.fence_tx.send(fence_id) {
                    warn!("Failed to send fence signal: {:?}", e);
                }
            },

            RenderCommand::Shutdown => {
                // Handled in run() loop
            },
        }
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
            let size = (vertices.len() * std::mem::size_of::<ImmVertex>()) as isize;
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
                return Err(format!("Vertex shader error: {}", String::from_utf8_lossy(&buffer)));
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
                return Err(format!("Fragment shader error: {}", String::from_utf8_lossy(&buffer)));
            }

            let program = gl::CreateProgram();
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);

            // CRITICAL: Bind attribute locations BEFORE linking!
            // Must match the VAO setup: 0=position, 1=normal, 2=uv, 3=color
            gl::BindAttribLocation(program, 0, b"vertex_position\0".as_ptr() as *const _);
            gl::BindAttribLocation(program, 1, b"vertex_normal\0".as_ptr() as *const _);
            gl::BindAttribLocation(program, 2, b"vertex_uv\0".as_ptr() as *const _);
            gl::BindAttribLocation(program, 3, b"vertex_color\0".as_ptr() as *const _);

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
                return Err(format!("Shader link error: {}", String::from_utf8_lossy(&buffer)));
            }

            // Bind CameraUBO to binding point 0 (if present in shader)
            let block_index = gl::GetUniformBlockIndex(program, b"CameraUBO\0".as_ptr() as *const _);
            if block_index != gl::INVALID_INDEX {
                gl::UniformBlockBinding(program, block_index, 0);
            }

            // Bind LightUBO to binding point 2 (if present in shader)
            let light_block_index = gl::GetUniformBlockIndex(program, b"LightUBO\0".as_ptr() as *const _);
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
        format: super::TexFormat,
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

    fn create_mesh(&self, vertices: &[u8], indices: &[u32], format: &VertexFormat) -> (u32, u32, u32) {
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
                gl::VertexAttribPointer(location, 3, gl::FLOAT, gl::FALSE, stride, offset as *const _);
                offset += 12; // 3 floats
                location += 1;
            }

            if format.has_normal {
                gl::EnableVertexAttribArray(location);
                gl::VertexAttribPointer(location, 3, gl::FLOAT, gl::FALSE, stride, offset as *const _);
                offset += 12; // 3 floats
                location += 1;
            }

            if format.has_uv {
                gl::EnableVertexAttribArray(location);
                gl::VertexAttribPointer(location, 2, gl::FLOAT, gl::FALSE, stride, offset as *const _);
                offset += 8; // 2 floats
                location += 1;
            }

            if format.has_color {
                gl::EnableVertexAttribArray(location);
                gl::VertexAttribPointer(location, 4, gl::FLOAT, gl::FALSE, stride, offset as *const _);
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
                GpuResource::Texture2D { handle } | GpuResource::Texture3D { handle } | GpuResource::TextureCube { handle } => {
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
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                288,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );
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
            gl::BufferSubData(
                gl::UNIFORM_BUFFER,
                0,
                288,
                data.as_ptr() as *const _,
            );
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
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                32,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );
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
            gl::BufferSubData(
                gl::UNIFORM_BUFFER,
                0,
                32,
                data.as_ptr() as *const _,
            );
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
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                32,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );
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
            gl::BufferSubData(
                gl::UNIFORM_BUFFER,
                0,
                32,
                data.as_ptr() as *const _,
            );
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }

    /// Push a new framebuffer onto the FBO stack
    fn push_framebuffer(&mut self) {
        if self.fbo_stack.len() >= FBO_STACK_DEPTH {
            error!("RenderThread: Maximum FBO stack depth {} exceeded", FBO_STACK_DEPTH);
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
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                0,
                0,
            );

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

    fn cleanup(&mut self) {
        info!("Cleaning up render thread resources ({} resources to clean)", self.resources.len());

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
        if let Some(gl_ctx) = self.gl_context.take() {
            info!("Releasing GL context for main thread...");
            match gl_ctx.release_for_main_thread() {
                Ok((not_current_ctx, surface)) => {
                    let returned = ReturnedGlContext {
                        context: not_current_ctx,
                        surface,
                    };
                    info!("Signaling main thread with returned context...");
                    if let Err(e) = self.context_tx.send(Some(returned)) {
                        error!("Failed to return GL context: {:?}", e);
                    } else {
                        info!("GL context returned to main thread");
                    }
                }
                Err(e) => {
                    // On macOS, this is expected - context was leaked to avoid deadlock
                    warn!("Could not release GL context: {} - marking unavailable", e);
                    super::set_gl_unavailable();
                    if let Err(e) = self.context_tx.send(None) {
                        error!("Failed to signal main thread: {:?}", e);
                    } else {
                        info!("Main thread signaled (no context returned)");
                    }
                }
            }
        } else {
            info!("No GL context to return");
            if let Err(e) = self.context_tx.send(None) {
                error!("Failed to signal main thread: {:?}", e);
            }
        }
    }
}

/// Spawn the render thread.
///
/// The GL context data is passed to the render thread which will make
/// the context current on its own thread.
pub fn spawn_render_thread(
    config: RenderThreadConfig,
    gl_data: Option<crate::window::RenderThreadGlData>,
) -> RenderThreadHandle {
    // Use bounded channel for backpressure - SwapBuffers will block to sync with render thread
    let (command_tx, command_rx) = bounded(config.command_buffer_size);
    let (fence_tx, fence_rx) = bounded(config.fence_buffer_size);
    let (shader_result_tx, shader_result_rx) = bounded(16); // Buffer for shader reload results
    let (context_tx, context_rx) = bounded(1); // Only one context to return
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let shared_stats = Arc::new(SharedRenderStats::default());
    let shared_stats_clone = shared_stats.clone();

    let thread_handle = thread::Builder::new()
        .name("RenderThread".into())
        .spawn(move || {
            // Make GL context current on this thread
            let gl_context = if let Some(data) = gl_data {
                match data.make_current() {
                    Ok(ctx) => {
                        info!("GL context made current on render thread");
                        Some(ctx)
                    }
                    Err(e) => {
                        error!("Failed to make GL context current on render thread: {}", e);
                        None
                    }
                }
            } else {
                warn!("Render thread started without GL context - commands will be no-ops");
                None
            };

            // Pass GL context to render thread for buffer swapping
            let mut render_thread = RenderThread::new(command_rx, fence_tx, shader_result_tx, context_tx, running_clone, shared_stats_clone, gl_context);
            render_thread.run();

            // GL context will be returned via channel or dropped if cleanup fails
        })
        .expect("Failed to spawn render thread");

    info!("Render thread spawned");

    RenderThreadHandle {
        command_tx,
        fence_rx,
        shader_result_rx,
        context_rx,
        next_fence_id: AtomicU64::new(1),
        frames_in_flight: AtomicU64::new(0),
        running,
        thread_handle: Some(thread_handle),
        shared_stats,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_thread_config_default() {
        let config = RenderThreadConfig::default();
        assert_eq!(config.command_buffer_size, 8192);
        assert_eq!(config.fence_buffer_size, 64);
    }
}
