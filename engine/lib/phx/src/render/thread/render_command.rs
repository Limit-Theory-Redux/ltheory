//! Render commands for the multithreaded rendering system.
//!
//! All OpenGL operations are encoded as commands and sent to the render thread.
//! This allows the main thread and worker threads to submit rendering work
//! without directly touching the GL context.

use std::sync::Arc;

use crossbeam::channel::Sender;

use super::command_category::CommandCategory;
use crate::render::{
    BlendMode, CullFace, InstanceData, TexFilter, TexFormat, TexWrapMode, VertexFormat, gl,
};

/// A handle to a GPU resource (shader, texture, buffer, etc.)
/// The actual GL handle lives on the render thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuHandle(pub u32);

impl GpuHandle {
    pub const INVALID: GpuHandle = GpuHandle(0);

    pub fn is_valid(&self) -> bool {
        self.0 != Self::INVALID.0
    }
}

/// Unique identifier for resources being created
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u64);

/// A fixed, known-in-advance uniform name. `Copy`, so a value costs nothing
/// to construct or send across the render-thread channel. Used by the batch
/// path (`renderer_shared.rs`) for the two per-draw uniforms every shader in
/// this engine's pipeline exposes; extend with more variants if another
/// fixed name needs this path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericUniformName {
    MWorld,
    MWorldIT,
}

impl GenericUniformName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MWorld => "mWorld",
            Self::MWorldIT => "mWorldIT",
        }
    }
}

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
    // === State Management ===
    /// Set the viewport
    SetViewport {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },

    /// Set scissor test region
    SetScissor {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },

    /// Enable or disable scissor test
    EnableScissor(bool),

    /// Set blend mode
    SetBlendMode(BlendMode),

    /// Set face culling mode
    SetCullFace(CullFace),

    /// Enable or disable depth testing
    SetDepthTest(bool),

    /// Enable or disable depth writing
    SetDepthWritable(bool),

    /// Set wireframe mode
    SetWireframe(bool),

    /// Set line width for line primitives
    SetLineWidth(f32),

    /// Set point size for point primitives
    SetPointSize(f32),

    // === Shader Operations ===
    /// Bind a shader program
    BindShader { handle: GpuHandle },
    /// Bind a shader by resource ID (for shaders created on render thread)
    /// If shader_key is provided, check hot_reloaded_shaders first for live updates
    BindShaderByResource {
        id: ResourceId,
        shader_key: Option<String>,
    },

    /// Unbind current shader (bind 0)
    UnbindShader,

    /// Set integer uniform
    SetUniformInt { location: i32, value: i32 },

    /// Set ivec2 uniform
    SetUniformInt2 { location: i32, value: [i32; 2] },

    /// Set ivec3 uniform
    SetUniformInt3 { location: i32, value: [i32; 3] },

    /// Set ivec4 uniform
    SetUniformInt4 { location: i32, value: [i32; 4] },

    /// Set float uniform
    SetUniformFloat { location: i32, value: f32 },

    /// Set vec2 uniform
    SetUniformFloat2 { location: i32, value: [f32; 2] },

    /// Set vec3 uniform
    SetUniformFloat3 { location: i32, value: [f32; 3] },

    /// Set vec4 uniform
    SetUniformFloat4 { location: i32, value: [f32; 4] },

    /// Set mat4 uniform
    SetUniformMat4 { location: i32, value: [f32; 16] },

    /// Batched per-instance uniforms: mWorld + mWorldIT + scale in one
    /// command. The three per-mesh matrix/scale sends dominate the uniform
    /// command stream (3 commands + 3 FFI crossings per mesh); batching them
    /// cuts that to 1 command + 1 crossing. mWorldIT is derived from the
    /// already-computed mWorld (inverse) instead of a rebuild + fresh invert
    /// on the Lua side.
    SetInstanceUniforms {
        world_loc: i32,
        world_it_loc: i32,
        scale_loc: i32,
        world: [f32; 16],
        world_it: [f32; 16],
        scale: f32,
    },

    // === Name-based Uniform Operations (for command mode) ===
    // These look up uniform location by name on the render thread,
    // since the render thread's shader has different uniform indices
    // than the main thread's shader.
    /// Set integer uniform by name (Arc<str> for cheap cloning)
    SetUniformIntByName { name: Arc<str>, value: i32 },

    /// Set ivec2 uniform by name
    SetUniformInt2ByName { name: Arc<str>, value: [i32; 2] },

    /// Set ivec3 uniform by name
    SetUniformInt3ByName { name: Arc<str>, value: [i32; 3] },

    /// Set ivec4 uniform by name
    SetUniformInt4ByName { name: Arc<str>, value: [i32; 4] },

    /// Set float uniform by name
    SetUniformFloatByName { name: Arc<str>, value: f32 },

    /// Set vec2 uniform by name
    SetUniformFloat2ByName { name: Arc<str>, value: [f32; 2] },

    /// Set vec3 uniform by name
    SetUniformFloat3ByName { name: Arc<str>, value: [f32; 3] },

    /// Set vec4 uniform by name
    SetUniformFloat4ByName { name: Arc<str>, value: [f32; 4] },

    /// Set mat4 uniform by name
    SetUniformMat4ByName { name: Arc<str>, value: [f32; 16] },

    /// Set mat4 uniform by a fixed, known-in-advance name - no allocation,
    /// no cache needed at the call site (see `GenericUniformName`)
    SetUniformMat4ByGenericName {
        name: GenericUniformName,
        value: [f32; 16],
    },

    // === Texture Operations ===
    /// Bind a 2D texture to a slot
    BindTexture2D { slot: u32, handle: GpuHandle },

    /// Bind a 2D texture by resource ID (for textures created in command mode)
    BindTexture2DByResource { slot: u32, id: ResourceId },

    /// Bind a 1D texture by resource ID
    BindTexture1DByResource { slot: u32, id: ResourceId },

    /// Bind a 3D texture to a slot
    BindTexture3D { slot: u32, handle: GpuHandle },

    /// Bind a 3D texture by resource ID
    BindTexture3DByResource { slot: u32, id: ResourceId },

    /// Bind a cube texture to a slot
    BindTextureCube { slot: u32, handle: GpuHandle },

    /// Bind a cube texture by resource ID
    BindTextureCubeByResource { slot: u32, id: ResourceId },

    /// Unbind texture from slot
    UnbindTexture { slot: u32 },

    // === Texture State Commands ===
    /// Set magnification filter for a 2D texture
    SetTexture2DMagFilter {
        handle: GpuHandle,
        filter: TexFilter,
    },

    /// Set minification filter for a 2D texture
    SetTexture2DMinFilter {
        handle: GpuHandle,
        filter: TexFilter,
    },

    /// Set wrap mode for a 2D texture (both S and T)
    SetTexture2DWrapMode {
        handle: GpuHandle,
        mode: TexWrapMode,
    },

    /// Set mip level range for a 2D texture
    SetTexture2DMipRange {
        handle: GpuHandle,
        min_level: i32,
        max_level: i32,
    },

    /// Generate mipmaps for a 2D texture
    GenerateMipmap2D { handle: GpuHandle },

    /// Update data for a 2D texture (full image replacement)
    UpdateTexture2DData {
        handle: GpuHandle,
        width: i32,
        height: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    },

    /// Update data for a 2D texture by ResourceId (for textures created in command mode)
    UpdateTexture2DDataByResource {
        id: ResourceId,
        width: i32,
        height: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    },

    /// Set anisotropy filter for a 2D texture
    SetTexture2DAnisotropy { handle: GpuHandle, factor: f32 },

    /// Set anisotropy filter for a 2D texture by resource ID
    SetTexture2DAnisotropyByResource { id: ResourceId, factor: f32 },

    /// Set mip level range for a 2D texture by resource ID
    SetTexture2DMipRangeByResource {
        id: ResourceId,
        min_level: i32,
        max_level: i32,
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

    /// Set magnification filter for a texture by resource ID.
    /// Dispatches on the resource's own kind (1D/2D/3D/Cube) to pick the GL
    /// target, so callers don't need to know it.
    SetTextureMagFilterByResource { id: ResourceId, filter: TexFilter },

    /// Set minification filter for a texture by resource ID (see above)
    SetTextureMinFilterByResource { id: ResourceId, filter: TexFilter },

    /// Set wrap mode for a texture by resource ID (see above). Applies to
    /// every wrap axis the resource's target has (S only for 1D; S+T for
    /// 2D/Cube; S+T+R for 3D).
    SetTextureWrapModeByResource { id: ResourceId, mode: TexWrapMode },

    /// Generate mipmaps for a texture by resource ID (see above)
    GenerateMipmapByResource { id: ResourceId },

    /// Update data for a 1D texture by ResourceId
    UpdateTexture1DDataByResource {
        id: ResourceId,
        width: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    },

    /// Update data for a 3D texture by ResourceId
    UpdateTexture3DDataByResource {
        id: ResourceId,
        width: i32,
        height: i32,
        depth: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    },

    /// Update data for one face of a cube texture by ResourceId
    UpdateTextureCubeFaceDataByResource {
        id: ResourceId,
        face: u32,
        level: i32,
        size: i32,
        internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    },

    /// Copy the currently-bound read framebuffer into a (already-created,
    /// empty) 2D texture by resource ID. Used by `Tex2D::deep_clone` - the
    /// caller is expected to have already bound the source via
    /// `RenderTarget::push_tex2d`.
    CopyTexture2DFromFramebufferByResource {
        id: ResourceId,
        internal_format: i32,
        width: i32,
        height: i32,
    },

    /// Blocking readback of a 1D texture's full pixel data. The reply is
    /// sent directly on `reply_tx` by the executor - this works identically
    /// in both backends: the caller submits the command, then blocks on
    /// `reply_tx`'s paired receiver.
    ReadTexture1DData {
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
        reply_tx: Sender<Vec<u8>>,
    },

    /// Blocking readback of a 2D texture's full pixel data (see above)
    ReadTexture2DData {
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
        reply_tx: Sender<Vec<u8>>,
    },

    /// Blocking readback of a 3D texture's full pixel data (see above)
    ReadTexture3DData {
        id: ResourceId,
        pixel_format: u32,
        data_format: u32,
        reply_tx: Sender<Vec<u8>>,
    },

    /// Blocking readback of one face/level of a cube texture (see above)
    ReadTextureCubeFaceData {
        id: ResourceId,
        face: u32,
        level: i32,
        pixel_format: u32,
        data_format: u32,
        reply_tx: Sender<Vec<u8>>,
    },

    /// Blocking readback of a single RGBA8 pixel from a 2D texture, sampled
    /// via a temporary FBO (see above)
    SamplePixel2DByResource {
        id: ResourceId,
        x: i32,
        y: i32,
        reply_tx: Sender<[u8; 4]>,
    },

    /// Blocking readback of a rectangle of pixels from the currently-bound
    /// framebuffer (see above). Used by `Tex2D::screen_capture`.
    ReadFramebufferPixels {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        reply_tx: Sender<Vec<u8>>,
    },

    // === Framebuffer Operations ===
    /// Create and bind a new framebuffer, returning its handle via the FBO stack
    /// This is used by RenderTarget::push()
    PushFramebuffer {
        /// Local ID for tracking (mapped to GL handle on render thread)
        id: u64,
        width: i32,
        height: i32,
    },

    /// Pop and delete the current framebuffer, restore previous
    /// This is used by RenderTarget::pop()
    PopFramebuffer,

    /// Attach a 2D texture to the current framebuffer (by GL handle)
    FramebufferAttachTexture2D {
        attachment: u32, // GL_COLOR_ATTACHMENT0, GL_DEPTH_ATTACHMENT, etc.
        texture: GpuHandle,
        level: i32,
    },

    /// Attach a 2D texture to the current framebuffer (by resource ID)
    /// Used when the texture was created on the render thread
    FramebufferAttachTexture2DByResource {
        attachment: u32,
        id: ResourceId,
        level: i32,
    },

    /// Attach a 3D texture layer to the current framebuffer
    FramebufferAttachTexture3D {
        attachment: u32,
        texture: GpuHandle,
        layer: i32,
        level: i32,
    },

    /// Attach a 3D texture layer to the current framebuffer (by resource ID)
    FramebufferAttachTexture3DByResource {
        attachment: u32,
        id: ResourceId,
        layer: i32,
        level: i32,
    },

    /// Attach a cube map face to the current framebuffer
    FramebufferAttachTextureCube {
        attachment: u32,
        texture: GpuHandle,
        face: u32, // GL_TEXTURE_CUBE_MAP_POSITIVE_X, etc.
        level: i32,
    },

    /// Attach a cube map face to the current framebuffer (by resource ID)
    FramebufferAttachTextureCubeByResource {
        attachment: u32,
        id: ResourceId,
        face: u32,
        level: i32,
    },

    /// Set draw buffers for current framebuffer
    SetDrawBuffers { count: i32 },

    /// Bind a framebuffer by handle (legacy)
    BindFramebuffer { handle: GpuHandle },

    /// Bind default framebuffer (0)
    BindDefaultFramebuffer,

    /// Clear color and/or depth buffer
    Clear {
        color: Option<[f32; 4]>,
        depth: Option<f32>,
    },

    // === Mesh Operations ===
    /// Bind a mesh's VAO and enable vertex attributes
    BindMesh { vao: GpuHandle },

    /// Bind a mesh by its resource ID
    BindMeshByResource { id: ResourceId },

    /// Unbind mesh VAO and disable vertex attributes
    UnbindMesh,

    // === Drawing Operations ===
    /// Draw a mesh using its VAO
    DrawMesh {
        vao: GpuHandle,
        index_count: i32,
        primitive: CmdPrimitiveType,
    },

    /// Draw instanced mesh
    DrawMeshInstanced {
        vao: GpuHandle,
        index_count: i32,
        instance_count: i32,
        primitive: CmdPrimitiveType,
    },

    /// Draw a mesh by its resource ID (for command mode)
    DrawMeshByResource {
        id: ResourceId,
        index_count: i32,
        primitive: CmdPrimitiveType,
    },

    /// Draw instanced mesh by resource ID
    DrawMeshInstancedByResource {
        id: ResourceId,
        index_count: i32,
        instance_count: i32,
        primitive: CmdPrimitiveType,
    },

    /// Draw instanced mesh with per-instance data (transforms + colors)
    /// This creates/updates a temporary instance buffer and sets up attribute divisors
    DrawInstancedWithData {
        mesh_id: ResourceId,
        index_count: i32,
        instances: Vec<InstanceData>,
        primitive: CmdPrimitiveType,
    },

    /// Draw instanced mesh with per-instance INDICES into a static data
    /// texture (texture-fetch instancing): the GPU pulls each instance's
    /// transform from a texture uploaded once, so the producer only sends
    /// 4-byte indices per instance instead of an 84-byte InstanceData.
    /// This is the GL 3.3-compatible form of GPU-driven instancing and
    /// maps 1:1 onto a storage buffer under wgpu.
    DrawInstancedIndices {
        mesh_id: ResourceId,
        index_count: i32,
        indices: Vec<u32>,
        primitive: CmdPrimitiveType,
    },

    /// Draw immediate mode geometry (vertices submitted directly)
    DrawImmediate {
        primitive: CmdPrimitiveType,
        vertices: Vec<ImmVertex>,
    },

    // === Resource Creation (deferred to GL thread) ===
    /// Create a shader program from source. `reply_tx` receives `None` on
    /// success, `Some(error)` on a compile/link failure - the caller decides
    /// whether that's fatal (`Shader::new`/`load` panic) or recoverable
    /// (`Shader::reload` returns `false`).
    CreateShader {
        id: ResourceId,
        vertex_src: String,
        fragment_src: String,
        reply_tx: Sender<Option<String>>,
    },

    /// Blocking lookup of a uniform's location for a specific shader
    /// resource, independent of whichever program is currently bound (see
    /// `get_uniform_location_cached` for the current-program version used by
    /// by-name uniform commands). Sends `-1` if the resource doesn't exist or
    /// has no such uniform, matching `glGetUniformLocation`'s own convention.
    GetUniformLocationByResource {
        id: ResourceId,
        name: Arc<str>,
        reply_tx: Sender<i32>,
    },

    /// Reload a shader (compile and send result back via channel)
    ReloadShader {
        shader_key: String,
        vertex_src: String,
        fragment_src: String,
    },

    /// Create a 1D texture
    CreateTexture1D {
        id: ResourceId,
        width: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    },

    /// Create a 2D texture
    CreateTexture2D {
        id: ResourceId,
        width: u32,
        height: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    },

    /// Create a 3D texture
    CreateTexture3D {
        id: ResourceId,
        width: u32,
        height: u32,
        depth: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    },

    /// Create a cube texture (6 empty faces, matching `TexCube::new`)
    CreateTextureCube {
        id: ResourceId,
        size: u32,
        format: TexFormat,
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

    // === Uniform Buffer Objects ===
    /// Create camera UBO
    CreateCameraUBO,

    /// Update camera UBO data
    UpdateCameraUBO {
        data: Box<[u8; 288]>, // CameraUboData::SIZE = 288 bytes
    },

    /// Create material UBO
    CreateMaterialUBO,

    /// Update material UBO data
    UpdateMaterialUBO {
        data: [u8; 32], // MaterialUboData::SIZE = 32 bytes
    },

    /// Create light UBO
    CreateLightUBO,

    /// Update light UBO data
    UpdateLightUBO {
        data: [u8; 32], // LightUboData::SIZE = 32 bytes
    },

    // === Window Operations ===
    /// Resize the GL surface
    Resize { width: u32, height: u32 },

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
    /// frame time actually goes (draws vs uniforms vs texture binds vs …).
    pub fn category(&self) -> CommandCategory {
        use RenderCommand::*;
        match self {
            // === State Management ===
            SetViewport { .. }
            | SetScissor { .. }
            | EnableScissor(_)
            | SetBlendMode(_)
            | SetCullFace(_)
            | SetDepthTest(_)
            | SetDepthWritable(_)
            | SetWireframe(_)
            | SetLineWidth(_)
            | SetPointSize(_) => CommandCategory::State,

            // === Shader Operations ===
            BindShader { .. } | BindShaderByResource { .. } | UnbindShader => {
                CommandCategory::Shader
            }

            // === Uniform Operations ===
            SetUniformInt { .. }
            | SetUniformInt2 { .. }
            | SetUniformInt3 { .. }
            | SetUniformInt4 { .. }
            | SetUniformFloat { .. }
            | SetUniformFloat2 { .. }
            | SetUniformFloat3 { .. }
            | SetUniformFloat4 { .. }
            | SetUniformMat4 { .. }
            | SetInstanceUniforms { .. }
            | SetUniformIntByName { .. }
            | SetUniformInt2ByName { .. }
            | SetUniformInt3ByName { .. }
            | SetUniformInt4ByName { .. }
            | SetUniformFloatByName { .. }
            | SetUniformFloat2ByName { .. }
            | SetUniformFloat3ByName { .. }
            | SetUniformFloat4ByName { .. }
            | SetUniformMat4ByName { .. }
            | SetUniformMat4ByGenericName { .. } => CommandCategory::Uniform,

            // === Texture Binding ===
            BindTexture2D { .. }
            | BindTexture2DByResource { .. }
            | BindTexture1DByResource { .. }
            | BindTexture3D { .. }
            | BindTexture3DByResource { .. }
            | BindTextureCube { .. }
            | BindTextureCubeByResource { .. }
            | UnbindTexture { .. } => CommandCategory::Texture,

            // === Texture State / Data ===
            SetTexture2DMagFilter { .. }
            | SetTexture2DMinFilter { .. }
            | SetTexture2DWrapMode { .. }
            | SetTexture2DMipRange { .. }
            | GenerateMipmap2D { .. }
            | UpdateTexture2DData { .. }
            | UpdateTexture2DDataByResource { .. }
            | SetTexture2DAnisotropy { .. }
            | SetTexture2DAnisotropyByResource { .. }
            | SetTexture2DMipRangeByResource { .. }
            | SetTexel1DByResource { .. }
            | SetTexel2DByResource { .. }
            | SetTextureMagFilterByResource { .. }
            | SetTextureMinFilterByResource { .. }
            | SetTextureWrapModeByResource { .. }
            | GenerateMipmapByResource { .. }
            | UpdateTexture1DDataByResource { .. }
            | UpdateTexture3DDataByResource { .. }
            | UpdateTextureCubeFaceDataByResource { .. }
            | CopyTexture2DFromFramebufferByResource { .. } => CommandCategory::TextureData,

            // === Blocking Readbacks ===
            ReadTexture1DData { .. }
            | ReadTexture2DData { .. }
            | ReadTexture3DData { .. }
            | ReadTextureCubeFaceData { .. }
            | SamplePixel2DByResource { .. }
            | ReadFramebufferPixels { .. } => CommandCategory::Readback,

            // === Framebuffer Operations ===
            PushFramebuffer { .. }
            | PopFramebuffer
            | FramebufferAttachTexture2D { .. }
            | FramebufferAttachTexture2DByResource { .. }
            | FramebufferAttachTexture3D { .. }
            | FramebufferAttachTexture3DByResource { .. }
            | FramebufferAttachTextureCube { .. }
            | FramebufferAttachTextureCubeByResource { .. }
            | SetDrawBuffers { .. }
            | BindFramebuffer { .. }
            | BindDefaultFramebuffer
            | Clear { .. } => CommandCategory::Framebuffer,

            // === Mesh Operations ===
            BindMesh { .. } | BindMeshByResource { .. } | UnbindMesh => CommandCategory::Mesh,

            // === Drawing Operations ===
            DrawMesh { .. }
            | DrawMeshInstanced { .. }
            | DrawMeshByResource { .. }
            | DrawMeshInstancedByResource { .. }
            | DrawInstancedWithData { .. }
            | DrawInstancedIndices { .. }
            | DrawImmediate { .. } => CommandCategory::Draw,

            // === Resource Creation / Destruction ===
            CreateShader { .. }
            | GetUniformLocationByResource { .. }
            | ReloadShader { .. }
            | CreateTexture1D { .. }
            | CreateTexture2D { .. }
            | CreateTexture3D { .. }
            | CreateTextureCube { .. }
            | CreateMesh { .. }
            | DestroyResources { .. } => CommandCategory::Resource,

            // === Uniform Buffer Objects ===
            CreateCameraUBO
            | UpdateCameraUBO { .. }
            | CreateMaterialUBO
            | UpdateMaterialUBO { .. }
            | CreateLightUBO
            | UpdateLightUBO { .. } => CommandCategory::Ubo,

            // === Window / Synchronization ===
            Resize { .. } | SwapBuffers | Flush | Fence { .. } | PacingFence { .. } | Shutdown => {
                CommandCategory::Sync
            }
        }
    }

    /// Returns true if this command modifies GPU state
    pub fn is_state_change(&self) -> bool {
        matches!(
            self,
            RenderCommand::SetViewport { .. }
                | RenderCommand::SetScissor { .. }
                | RenderCommand::EnableScissor(_)
                | RenderCommand::SetBlendMode(_)
                | RenderCommand::SetCullFace(_)
                | RenderCommand::SetDepthTest(_)
                | RenderCommand::SetDepthWritable(_)
                | RenderCommand::SetWireframe(_)
                | RenderCommand::SetLineWidth(_)
                | RenderCommand::SetPointSize(_)
                | RenderCommand::BindShader { .. }
                | RenderCommand::UnbindShader
                | RenderCommand::BindTexture2D { .. }
                | RenderCommand::BindTexture3D { .. }
                | RenderCommand::BindTextureCube { .. }
                | RenderCommand::BindFramebuffer { .. }
                | RenderCommand::BindDefaultFramebuffer
        )
    }

    /// Returns true if this command is a draw call
    pub fn is_draw_call(&self) -> bool {
        matches!(
            self,
            RenderCommand::DrawMesh { .. }
                | RenderCommand::DrawMeshInstanced { .. }
                | RenderCommand::DrawMeshByResource { .. }
                | RenderCommand::DrawMeshInstancedByResource { .. }
                | RenderCommand::DrawImmediate { .. }
                | RenderCommand::DrawInstancedWithData { .. }
                | RenderCommand::DrawInstancedIndices { .. }
        )
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
                | RenderCommand::CreateTexture2D { .. }
                | RenderCommand::CreateMesh { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_handle() {
        assert!(!GpuHandle::INVALID.is_valid());
        assert!(GpuHandle(1).is_valid());
    }

    #[test]
    fn test_command_classification() {
        let state_cmd = RenderCommand::SetBlendMode(BlendMode::Alpha);
        assert!(state_cmd.is_state_change());
        assert!(!state_cmd.is_draw_call());

        let draw_cmd = RenderCommand::DrawMesh {
            vao: GpuHandle(1),
            index_count: 36,
            primitive: CmdPrimitiveType::Triangles,
        };
        assert!(!draw_cmd.is_state_change());
        assert!(draw_cmd.is_draw_call());
    }
}
