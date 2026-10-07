//! Render commands for the multithreaded rendering system.
//!
//! All OpenGL operations are encoded as commands and sent to the render thread.
//! This allows the main thread and worker threads to submit rendering work
//! without directly touching the GL context.

use std::sync::Arc;

use crossbeam::channel::Sender;

use super::command_category::CommandCategory;
use crate::render::{
    BindEntry, BindGroupId, BlockLayout, BufferId, PassCommands, PipelineDesc, PipelineId,
    ReadSource, ReadbackSlot, RenderPassDesc, SamplerDesc, SamplerId, ShaderLayout, TexDesc,
    TexFormat, TexRegion, TexView, VertexFormat, gl,
};
use crate::window::PresentMode;

/// Unique identifier for resources being created
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u64);

/// Primitive type for drawing operations (command buffer version)
#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdPrimitiveType {
    Points,
    Lines,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
    Quads,
}

impl CmdPrimitiveType {
    pub fn to_gl(&self) -> u32 {
        match self {
            Self::Points => gl::POINTS,
            Self::Lines => gl::LINES,
            Self::LineStrip => gl::LINE_STRIP,
            Self::Triangles => gl::TRIANGLES,
            Self::TriangleStrip => gl::TRIANGLE_STRIP,
            Self::TriangleFan => gl::TRIANGLE_FAN,
            Self::Quads => gl::TRIANGLES, // Quads converted to triangles
        }
    }
}

/// Vertex data for immediate mode drawing
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ImmVertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

/// A render command that can be executed on the render thread.
///
/// Commands are designed to be:
/// 1. Self-contained - all data needed is in the command
/// 2. Thread-safe to send between threads
/// 3. Efficiently batchable
#[derive(Debug, Clone)]
pub enum RenderCommand {
    /// Write `data`, tightly packed texels in the texture's own `TexFormat`
    /// layout (see `convert_texels`), to `region` of a texture. `GpuResource`
    /// kind decides the target (1D, 2D, 3D, or a cube face by `origin[2]`).
    UpdateTexture {
        id: ResourceId,
        region: TexRegion,
        data: Vec<u8>,
    },

    /// Set a single texel of a 1D texture by resource ID
    SetTexel1DByResource {
        id: ResourceId,
        x: i32,
        color: [f32; 4],
    },

    /// Set a single texel of a 2D texture by resource ID
    SetTexel2DByResource {
        id: ResourceId,
        x: i32,
        y: i32,
        color: [f32; 4],
    },

    /// Fill every mip level below 0 from level 0. GL: `glGenerateMipmap`.
    /// wgpu: a chain of blit passes, one per level (and face or slice).
    GenerateMips { id: ResourceId },

    /// Copy a `size[0]` x `size[1]` rectangle (`size[2]` layers, always 1 on
    /// GL) from the origin of `src` to the origin of `dst`. Both views are
    /// one level of a 2D texture, one face of a cube or one z-slice of a 3D
    /// texture, with the same format. GL: `glBlitFramebuffer` between two
    /// scratch framebuffers (`glCopyImageSubData` needs 4.3).
    CopyTexture {
        src: TexView,
        dst: TexView,
        size: [u32; 3],
    },

    /// Copy the currently-bound read framebuffer into a (already-created,
    /// empty) 2D texture by resource ID. Used by `Tex2D::deep_clone` - the
    /// caller is expected to have already bound the source by opening
    /// a render pass on it.
    CopyTexture2DFromFramebufferByResource {
        id: ResourceId,
        format: TexFormat,
        width: i32,
        height: i32,
    },

    /// Blocking readback of `region` of `src` in `format` (tightly packed
    /// rows, row 0 first; the executor converts if the texture is stored
    /// differently). The reply is sent directly on `reply_tx` by the executor
    /// - this works identically in both backends: the caller submits the
    /// command, then blocks on `reply_tx`'s paired receiver. An empty reply
    /// means the read failed.
    ReadTextureSync {
        src: ReadSource,
        region: TexRegion,
        format: TexFormat,
        reply_tx: Sender<Vec<u8>>,
    },

    /// Start a readback of `region` of `src` in `format` without waiting
    /// (GL: a pixel pack buffer and a fence; wgpu: a mappable buffer). The
    /// executor polls it at `BeginFrame` and fills `slot` when the GPU is
    /// done (see `ReadbackTicket`).
    ReadbackAsync {
        src: ReadSource,
        region: TexRegion,
        format: TexFormat,
        slot: Arc<ReadbackSlot>,
    },

    // === Render Passes ===
    /// Begin a render pass: bind the attachments (or the backbuffer), set the
    /// viewport to the attachment size, apply the load ops.
    BeginRenderPass(Box<RenderPassDesc>),

    /// End the open render pass and return to the default framebuffer.
    EndRenderPass,

    /// A new frame starts in uniform-ring slot `slot`. GL: wait for the fence
    /// inserted after the frame that last used the slot, so its ring buffers
    /// can be overwritten.
    BeginFrame { slot: u8 },

    /// The commands recorded in the open pass since the last flush, with the
    /// uniform ring bytes they reference (see `PassEncoder`).
    PassCommands(Box<PassCommands>),

    // === Binding model objects ===
    /// Create a pipeline (shader + fixed-function state).
    CreatePipeline {
        id: PipelineId,
        desc: Box<PipelineDesc>,
    },

    /// Create a sampler object.
    CreateSampler { id: SamplerId, desc: SamplerDesc },

    /// Create a bind group: uniform ranges and textures with samplers for one
    /// group of `shader`.
    CreateBindGroup {
        id: BindGroupId,
        shader: ResourceId,
        group: u8,
        entries: Box<[BindEntry]>,
    },

    /// Destroy bind groups (their material was dropped or changed).
    DestroyBindGroups { ids: Vec<BindGroupId> },

    /// Create a uniform buffer of `size` bytes (material parameter arenas).
    CreateBuffer { id: BufferId, size: u32 },

    /// Write `data` at byte `offset` of buffer `id`. Not allowed inside an
    /// open pass on wgpu (`queue.write_buffer` orders before the whole
    /// submission); GL executes it in command order.
    WriteBuffer {
        id: BufferId,
        offset: u32,
        data: Vec<u8>,
    },

    // === Resource Creation (deferred to GL thread) ===
    /// Create a shader program from source and apply its `#group` layout
    /// (block bindings, sampler units). `reply_tx` receives the reflected
    /// uniform blocks on success, the error on a compile/link failure - the
    /// caller decides whether that's fatal (`Shader::new`/`load` panic) or
    /// recoverable (`Shader::reload` returns `false`).
    CreateShader {
        id: ResourceId,
        vertex_src: String,
        fragment_src: String,
        layout: Arc<ShaderLayout>,
        reply_tx: Sender<Result<Vec<BlockLayout>, String>>,
    },

    /// Create a texture. `data` (optional, 1D/2D/3D only) is level 0 in the
    /// texture's own `TexFormat` layout; the other levels are allocated
    /// (`desc.mips`) but not filled, see `GenerateMips`.
    CreateTexture {
        id: ResourceId,
        desc: Box<TexDesc>,
        data: Option<Vec<u8>>,
    },

    /// Create a mesh from vertex/index data
    CreateMesh {
        id: ResourceId,
        vertices: Vec<u8>,
        indices: Vec<u32>,
        vertex_format: VertexFormat,
    },

    /// Destroy multiple resources
    DestroyResources { ids: Vec<ResourceId> },

    // === Window Operations ===
    /// Resize the GL surface
    Resize { width: u32, height: u32 },

    /// Change the swap interval (vsync on/off). Must go through the render
    /// thread because `set_swap_interval` requires the GL context current,
    /// and the context lives there - see `Engine::changed_window`.
    SetPresentMode { mode: PresentMode },

    /// Swap buffers (present frame)
    SwapBuffers,

    // === Synchronization ===
    /// Flush all pending GL commands (gl::Finish)
    Flush,

    /// Fence for synchronization - render thread sends fence_id back when reached.
    /// Used by blocking round-trips (readbacks, shader reload/compile, etc.)
    /// via `Renderer::sync_intern`.
    Fence { fence_id: u64 },

    /// Same as `Fence`, but replied on its own channel (see
    /// `CommandReply::PacingFence`) so it can never be consumed by a
    /// `sync_intern` call that happens to be waiting concurrently, or vice
    /// versa - the two used to share one channel, which let one steal the
    /// other's fence and corrupt `end_frame_triple_buffered`'s in-flight
    /// count (or make `sync_intern` hang waiting for a fence that was
    /// already consumed elsewhere).
    PacingFence { fence_id: u64 },

    /// Shutdown render thread
    Shutdown,
}

impl RenderCommand {
    /// Coarse cost category used by the stats dashboard. Commands in the same
    /// category have similar per-command GPU/driver cost, so summing counts
    /// and execution time per category shows *where* the render thread's
    /// frame time actually goes (pass commands vs texture data vs resources
    /// vs ...).
    pub fn category(&self) -> CommandCategory {
        use RenderCommand::*;
        match self {
            // === Texture State / Data ===
            UpdateTexture { .. }
            | SetTexel1DByResource { .. }
            | SetTexel2DByResource { .. }
            | GenerateMips { .. }
            | CopyTexture { .. }
            | CopyTexture2DFromFramebufferByResource { .. } => CommandCategory::TextureData,

            // === Blocking Readbacks ===
            ReadTextureSync { .. } | ReadbackAsync { .. } => CommandCategory::Readback,

            // === Render Passes ===
            BeginRenderPass(_) | EndRenderPass => CommandCategory::Framebuffer,
            BeginFrame { .. } => CommandCategory::Sync,
            PassCommands(_) => CommandCategory::Draw,

            // === Resource Creation / Destruction ===
            CreateShader { .. }
            | CreatePipeline { .. }
            | CreateSampler { .. }
            | CreateBindGroup { .. }
            | DestroyBindGroups { .. }
            | CreateBuffer { .. }
            | WriteBuffer { .. }
            | CreateTexture { .. }
            | CreateMesh { .. }
            | DestroyResources { .. } => CommandCategory::Resource,

            // === Window / Synchronization ===
            Resize { .. }
            | SetPresentMode { .. }
            | SwapBuffers
            | Flush
            | Fence { .. }
            | PacingFence { .. }
            | Shutdown => CommandCategory::Sync,
        }
    }

    /// Returns true if this command requires synchronization
    pub fn requires_sync(&self) -> bool {
        matches!(
            self,
            RenderCommand::SwapBuffers
                | RenderCommand::Fence { .. }
                | RenderCommand::PacingFence { .. }
                | RenderCommand::Shutdown
                | RenderCommand::CreateShader { .. }
                | RenderCommand::CreateTexture { .. }
                | RenderCommand::CreateMesh { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_categories() {
        assert_eq!(RenderCommand::SwapBuffers.category(), CommandCategory::Sync);
        assert_eq!(
            RenderCommand::EndRenderPass.category(),
            CommandCategory::Framebuffer
        );
    }
}
