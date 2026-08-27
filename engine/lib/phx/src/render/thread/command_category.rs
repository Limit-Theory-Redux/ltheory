//! Coarse cost categories for render commands.
//!
//! [`RenderCommand::category`] maps each command variant here.

/// Coarse cost category for a render command, used by the stats dashboard
/// to show where render-thread time goes. Order matters: `ALL.len()` and
/// the per-frame accumulator arrays in the executor are indexed by
/// discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandCategory {
    /// Viewport/scissor/blend/depth/cull/wireframe state changes
    State,
    /// Shader bind/unbind (also what invalidates the texture cache)
    Shader,
    /// SetUniform* by location or by name
    Uniform,
    /// BindTexture*/UnbindTexture
    Texture,
    /// Texture parameter/upload/texel commands
    TextureData,
    /// Blocking readbacks (glReadPixels etc.)
    Readback,
    /// FBO push/pop/attach, draw buffers, clear
    Framebuffer,
    /// Mesh bind/unbind
    Mesh,
    /// DrawMesh*/DrawImmediate
    Draw,
    /// Shader/texture/mesh creation, destroy, reload
    Resource,
    /// Camera/material/light UBO updates
    Ubo,
    /// SwapBuffers, fences, flush, resize, shutdown
    Sync,
}

impl CommandCategory {
    pub const ALL: [Self; 12] = [
        Self::State,
        Self::Shader,
        Self::Uniform,
        Self::Texture,
        Self::TextureData,
        Self::Readback,
        Self::Framebuffer,
        Self::Mesh,
        Self::Draw,
        Self::Resource,
        Self::Ubo,
        Self::Sync,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::Shader => "shader",
            Self::Uniform => "uniform",
            Self::Texture => "texture",
            Self::TextureData => "texture_data",
            Self::Readback => "readback",
            Self::Framebuffer => "framebuffer",
            Self::Mesh => "mesh",
            Self::Draw => "draw",
            Self::Resource => "resource",
            Self::Ubo => "ubo",
            Self::Sync => "sync",
        }
    }

    pub fn index(&self) -> usize {
        *self as usize
    }
}
