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
            command_buffer_size: 8192,
            fence_buffer_size: 64,
        }
    }
}
