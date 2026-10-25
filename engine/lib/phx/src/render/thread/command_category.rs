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
    /// Fixed-function state changes (no command of this category is left)
    State,
    /// Shader bind/unbind (no command of this category is left)
    Shader,
    /// Loose uniform sets (no command of this category is left)
    Uniform,
    /// Texture binds (no command of this category is left)
    Texture,
    /// Texture parameter/upload/texel commands
    TextureData,
    /// Blocking readbacks (glReadPixels etc.)
    Readback,
    /// FBO push/pop/attach, draw buffers, clear
    Framebuffer,
    /// Mesh bind/unbind (no command of this category is left)
    Mesh,
    /// Pass commands (`PassCommands`: draws and the state around them)
    Draw,
    /// Shader/texture/mesh creation, destroy, reload
    Resource,
    /// SwapBuffers, fences, flush, resize, shutdown. `SwapBuffers`'s own
    /// blocking present (vsync/vblank wait) is deliberately excluded from
    /// this category's timing and reported separately as
    /// `RenderStats::present_wait_us` - see the comment in `cmd_swap_buffers`.
    Sync,
}

impl CommandCategory {
    pub const ALL: [Self; 11] = [
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
            Self::Sync => "sync",
        }
    }

    pub fn index(&self) -> usize {
        *self as usize
    }
}
