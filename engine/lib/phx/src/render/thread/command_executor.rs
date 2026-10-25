use crate::render::RenderStats;

/// Result a command may hand back to whoever drove the executor.
///
/// The executor owns no channels, so commands that need to answer the caller
/// return the answer instead of sending it; `RenderThread` forwards it over
/// the matching channel.
#[derive(Debug, Default)]
pub enum CommandReply {
    #[default]
    None,
    Fence(u64),
    /// Reply to `RenderCommand::PacingFence` - kept on its own variant/channel
    /// so it can never be picked up by whichever code is waiting on a plain
    /// `Fence` (see `RenderCommand::PacingFence`'s docs).
    PacingFence(u64),
    Stats(Box<RenderStats>),
}

/// Statistics from the render thread (local copy)
#[derive(Debug, Clone, Default)]
pub struct ExecutorStats {
    pub commands_processed: u64,
    pub draw_calls: u64,
    pub state_changes: u64,
    pub frame_count: u64,
}
