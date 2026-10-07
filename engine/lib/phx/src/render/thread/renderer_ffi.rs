use glam::Vec3;

use crate::math::Matrix;
use crate::render::{
    BindGroupDesc, ReadSource, ReadbackTicket, RenderPass, RenderPassDesc, Renderer, TexCube,
    TexFormat, TexView, view_region,
};
use crate::system::{Bytes, Metric};

// =============================================================================
// FFI-exposed Renderer API
//
// These are thin Lua-facing wrappers: they convert Lua-friendly primitives
// (ints, separate floats, ...) into the native types the per-command methods
// on `Renderer` (defined in `renderer_immediate.rs`/`renderer_threaded.rs`)
// expect, then call those directly. The per-command methods carry an
// `_intern` suffix so these wrappers can keep the plain, Lua-facing name
// without clashing with them by name.
// =============================================================================

#[luajit_ffi_gen::luajit_ffi]
impl Renderer {
    // === Frame Management ===

    /// Synchronize with the render thread (wait for all commands to complete)
    pub fn sync(&mut self) -> bool {
        self.sync_intern()
    }

    /// Block until the GPU has finished everything submitted so far
    /// (`glFinish`), e.g. to time a piece of GPU work.
    pub fn gpu_finish(&mut self) {
        Metric::Flush.inc();
        self.gl_finish();
    }

    // === Frame stats (last completed frame; used by LTHEORY_CAPTURE) ===

    /// Draw calls of the last frame (mesh + immediate + instanced).
    pub fn stats_draw_calls(&mut self) -> u64 {
        let s = self.get_stats();
        s.draw_mesh_calls + s.draw_immediate_calls + s.draw_instanced_calls
    }

    /// Render-thread execute time of the last frame, in microseconds.
    pub fn stats_frame_time_us(&mut self) -> u64 {
        self.get_stats().last_frame_time_us
    }

    /// Time the render thread sat blocked waiting for commands in the last
    /// frame (producer starvation), microseconds.
    pub fn stats_recv_wait_us(&mut self) -> u64 {
        self.get_stats().recv_wait_us
    }

    /// Time the render thread spent blocked in the buffer swap (vsync/GPU
    /// back-pressure) in the last frame, microseconds.
    pub fn stats_present_wait_us(&mut self) -> u64 {
        self.get_stats().present_wait_us
    }

    /// Frames the render thread has completed (to de-duplicate stats samples).
    pub fn stats_frame_count(&mut self) -> u64 {
        self.get_stats().frame_count
    }

    /// Commands the render thread processed in the last frame.
    pub fn stats_commands(&mut self) -> u64 {
        self.get_stats().commands
    }

    /// Time the main thread spent blocked in the last frame end, microseconds.
    pub fn stats_main_wait_us(&self) -> u64 {
        self.get_main_thread_wait_us()
    }

    pub fn stats_vertices(&mut self) -> u64 {
        self.get_stats().vertices_drawn
    }

    // === Readback ===

    /// Read the `w` x `h` texels at `x`, `y` of `view` (its mip level, face or
    /// layer) as `fmt` and wait for them: rows from the first up, tightly
    /// packed, in the layout of `fmt` (the texture is converted if it is
    /// stored differently). **Stalls until the GPU has produced the data**:
    /// for screenshots, tests and tools only, never in a frame. Empty `Bytes`
    /// if the read failed.
    pub fn read_sync(
        &mut self,
        view: &TexView,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        fmt: TexFormat,
    ) -> Bytes {
        let region = view_region(view, x, y, w, h);
        Bytes::from_vec(self.read_texture_sync(ReadSource::Texture(view.tex), region, fmt))
    }

    /// Start reading the `w` x `h` texels at `x`, `y` of `view` as `fmt`
    /// without waiting. Poll the ticket (`:ready()`) once per frame; the data
    /// arrives two or three frames later. See `ReadbackTicket`.
    pub fn read_async(
        &mut self,
        view: &TexView,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        fmt: TexFormat,
    ) -> ReadbackTicket {
        let region = view_region(view, x, y, w, h);
        self.read_texture_async(ReadSource::Texture(view.tex), region, fmt)
    }

    // === Render Passes ===

    /// Begin a render pass on `desc`'s attachments. Only one pass may be open
    /// at a time; end it with `RenderPass:finish()`.
    pub fn begin_pass(&mut self, desc: &RenderPassDesc) -> RenderPass {
        self.begin_pass_intern(desc)
    }

    /// The open pass, for code that records into it without owning it (for
    /// example UI widgets calling `pass:setUiTransform`). It cannot `finish`
    /// the pass. Errors if no pass is open.
    pub fn current_pass(&self) -> RenderPass {
        self.current_pass_intern()
    }

    // === Window Operations ===

    /// Signal resize
    pub fn resize(&mut self, width: u32, height: u32) {
        self.resize_intern(width, height);
    }

    /// Signal swap buffers (frame end)
    pub fn swap_buffers(&mut self) {
        self.swap_buffers_intern();
    }

    // === Frame group (group 0) ===

    /// Set the camera of the passes that begin from now on (and of the open
    /// pass): view and projection matrices and the direction towards the
    /// primary light. Rendering is camera-relative, so the eye is the origin.
    /// Replaces the old shader-variable stack and the camera UBO update.
    pub fn set_camera(&mut self, view: &Matrix, proj: &Matrix, star_dir: &Vec3) {
        self.set_camera_intern(view, proj, *star_dir);
    }

    /// Set the environment cube maps (`envMap` and `irMap` of group 0) of the
    /// passes that begin from now on (and of the open pass). Replaces
    /// the old per-shader `envMap`/`irMap` variables.
    pub fn set_environment(&mut self, env_map: &TexCube, ir_map: &TexCube) {
        self.set_environment_intern(env_map, ir_map);
    }

    // === Binding model objects ===

    /// Create a bind group from `desc`; bind it in a pass with
    /// `pass:setBindGroup(group, id)`.
    pub fn create_bind_group(&mut self, desc: &BindGroupDesc) -> u32 {
        self.create_bind_group_from_desc(desc).0
    }
}
