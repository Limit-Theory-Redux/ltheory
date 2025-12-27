//! Render commands for the multithreaded rendering system.
//!
//! All OpenGL operations are encoded as commands and sent to the render thread.
//! This allows the main thread and worker threads to submit rendering work
//! without directly touching the GL context.

use std::sync::Arc;

use super::{BlendMode, CullFace, TexFilter, TexFormat, TexWrapMode};

/// A handle to a GPU resource (shader, texture, buffer, etc.)
/// The actual GL handle lives on the render thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuHandle(pub u32);

impl GpuHandle {
    pub const INVALID: GpuHandle = GpuHandle(0);

    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

/// Unique identifier for resources being created
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u64);

/// Primitive type for drawing operations (command buffer version)
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
        use super::gl;
        match self {
            CmdPrimitiveType::Points => gl::POINTS,
            CmdPrimitiveType::Lines => gl::LINES,
            CmdPrimitiveType::LineStrip => gl::LINE_STRIP,
            CmdPrimitiveType::Triangles => gl::TRIANGLES,
            CmdPrimitiveType::TriangleStrip => gl::TRIANGLE_STRIP,
            CmdPrimitiveType::TriangleFan => gl::TRIANGLE_FAN,
            CmdPrimitiveType::Quads => gl::TRIANGLES, // Quads converted to triangles
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

/// Per-instance data for instanced rendering
/// Layout: model matrix (64 bytes) + color (16 bytes) = 80 bytes per instance
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InstanceData {
    /// Model matrix (column-major, 4x4)
    pub model_matrix: [f32; 16],
    /// Per-instance color (RGBA)
    pub color: [f32; 4],
}

impl InstanceData {
    pub fn new(model_matrix: [f32; 16], color: [f32; 4]) -> Self {
        Self { model_matrix, color }
    }

    pub fn from_transform_color(transform: &[f32; 16], r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            model_matrix: *transform,
            color: [r, g, b, a],
        }
    }
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
    BindShaderByResource { id: ResourceId, shader_key: Option<String> },

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

    // === Texture Operations ===
    /// Bind a 2D texture to a slot
    BindTexture2D { slot: u32, handle: GpuHandle },

    /// Bind a 2D texture by resource ID (for textures created in command mode)
    BindTexture2DByResource { slot: u32, id: ResourceId },

    /// Bind a 3D texture to a slot
    BindTexture3D { slot: u32, handle: GpuHandle },

    /// Bind a cube texture to a slot
    BindTextureCube { slot: u32, handle: GpuHandle },

    /// Unbind texture from slot
    UnbindTexture { slot: u32 },

    // === Texture State Commands ===
    /// Set magnification filter for a 2D texture
    SetTexture2DMagFilter { handle: GpuHandle, filter: TexFilter },

    /// Set minification filter for a 2D texture
    SetTexture2DMinFilter { handle: GpuHandle, filter: TexFilter },

    /// Set wrap mode for a 2D texture (both S and T)
    SetTexture2DWrapMode { handle: GpuHandle, mode: TexWrapMode },

    /// Set mip level range for a 2D texture
    SetTexture2DMipRange { handle: GpuHandle, min_level: i32, max_level: i32 },

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
        attachment: u32,  // GL_COLOR_ATTACHMENT0, GL_DEPTH_ATTACHMENT, etc.
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

    /// Attach a cube map face to the current framebuffer
    FramebufferAttachTextureCube {
        attachment: u32,
        texture: GpuHandle,
        face: u32,  // GL_TEXTURE_CUBE_MAP_POSITIVE_X, etc.
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

    /// Draw immediate mode geometry (vertices submitted directly)
    DrawImmediate {
        primitive: CmdPrimitiveType,
        vertices: Vec<ImmVertex>,
    },

    // === Resource Creation (deferred to GL thread) ===
    /// Create a shader program from source
    CreateShader {
        id: ResourceId,
        vertex_src: String,
        fragment_src: String,
    },

    /// Reload a shader (compile and send result back via channel)
    ReloadShader {
        shader_key: String,
        vertex_src: String,
        fragment_src: String,
    },

    /// Create a 2D texture
    CreateTexture2D {
        id: ResourceId,
        width: u32,
        height: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    },

    /// Create a mesh from vertex/index data
    CreateMesh {
        id: ResourceId,
        vertices: Vec<u8>,
        indices: Vec<u32>,
        vertex_format: VertexFormat,
    },

    /// Destroy a resource
    DestroyResource { id: ResourceId },

    // === Uniform Buffer Objects ===
    /// Create camera UBO
    CreateCameraUBO,

    /// Update camera UBO data
    UpdateCameraUBO {
        data: Box<[u8; 288]>,  // CameraUboData::SIZE = 288 bytes
    },

    /// Create material UBO
    CreateMaterialUBO,

    /// Update material UBO data
    UpdateMaterialUBO {
        data: Box<[u8; 32]>,   // MaterialUboData::SIZE = 32 bytes
    },

    /// Create light UBO
    CreateLightUBO,

    /// Update light UBO data
    UpdateLightUBO {
        data: Box<[u8; 32]>,   // LightUboData::SIZE = 32 bytes
    },

    // === Window Operations ===
    /// Resize the GL surface
    Resize { width: u32, height: u32 },

    /// Swap buffers (present frame)
    SwapBuffers,

    // === Synchronization ===
    /// Flush all pending GL commands (gl::Finish)
    Flush,

    /// Fence for synchronization - render thread sends fence_id back when reached
    Fence { fence_id: u64 },

    /// Shutdown render thread
    Shutdown,
}

/// Vertex format description for mesh creation
#[derive(Debug, Clone)]
pub struct VertexFormat {
    pub has_position: bool,
    pub has_normal: bool,
    pub has_uv: bool,
    pub has_color: bool,
    pub stride: u32,
}

impl Default for VertexFormat {
    fn default() -> Self {
        Self {
            has_position: true,
            has_normal: true,
            has_uv: true,
            has_color: false,
            stride: 32, // 3 floats pos + 3 floats normal + 2 floats uv = 32 bytes
        }
    }
}

impl RenderCommand {
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
        )
    }

    /// Returns true if this command requires synchronization
    pub fn requires_sync(&self) -> bool {
        matches!(
            self,
            RenderCommand::SwapBuffers
                | RenderCommand::Fence { .. }
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
