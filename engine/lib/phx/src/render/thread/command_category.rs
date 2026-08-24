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
    pub const ALL: [CommandCategory; 12] = [
        CommandCategory::State,
        CommandCategory::Shader,
        CommandCategory::Uniform,
        CommandCategory::Texture,
        CommandCategory::TextureData,
        CommandCategory::Readback,
        CommandCategory::Framebuffer,
        CommandCategory::Mesh,
        CommandCategory::Draw,
        CommandCategory::Resource,
        CommandCategory::Ubo,
        CommandCategory::Sync,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            CommandCategory::State => "state",
            CommandCategory::Shader => "shader",
            CommandCategory::Uniform => "uniform",
            CommandCategory::Texture => "texture",
            CommandCategory::TextureData => "texture_data",
            CommandCategory::Readback => "readback",
            CommandCategory::Framebuffer => "framebuffer",
            CommandCategory::Mesh => "mesh",
            CommandCategory::Draw => "draw",
            CommandCategory::Resource => "resource",
            CommandCategory::Ubo => "ubo",
            CommandCategory::Sync => "sync",
        }
    }

    pub fn index(&self) -> usize {
        *self as usize
    }
}
