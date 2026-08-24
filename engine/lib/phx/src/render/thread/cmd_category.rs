//! Coarse cost categories for render commands.
//!
//! Extracted from `render_command` so the categorization table lives on
//! its own; [`RenderCommand::category`] maps each command variant here.

/// Coarse cost category for a render command, used by the stats dashboard
/// to show where render-thread time goes. Order matters: `ALL.len()` and
/// the per-frame accumulator arrays in the executor are indexed by
/// discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CmdCategory {
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

impl CmdCategory {
    pub const ALL: [CmdCategory; 12] = [
        CmdCategory::State,
        CmdCategory::Shader,
        CmdCategory::Uniform,
        CmdCategory::Texture,
        CmdCategory::TextureData,
        CmdCategory::Readback,
        CmdCategory::Framebuffer,
        CmdCategory::Mesh,
        CmdCategory::Draw,
        CmdCategory::Resource,
        CmdCategory::Ubo,
        CmdCategory::Sync,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            CmdCategory::State => "state",
            CmdCategory::Shader => "shader",
            CmdCategory::Uniform => "uniform",
            CmdCategory::Texture => "texture",
            CmdCategory::TextureData => "texture_data",
            CmdCategory::Readback => "readback",
            CmdCategory::Framebuffer => "framebuffer",
            CmdCategory::Mesh => "mesh",
            CmdCategory::Draw => "draw",
            CmdCategory::Resource => "resource",
            CmdCategory::Ubo => "ubo",
            CmdCategory::Sync => "sync",
        }
    }

    pub fn index(&self) -> usize {
        *self as usize
    }
}
