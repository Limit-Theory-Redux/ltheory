const DEFAULT_COMMAND_BUFFER_SIZE: usize = 8192;
const DEFAULT_FENCE_BUFFER_SIZE: usize = 64;

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
            command_buffer_size: DEFAULT_COMMAND_BUFFER_SIZE,
            fence_buffer_size: DEFAULT_FENCE_BUFFER_SIZE,
        }
    }
}
