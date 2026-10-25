#[derive(Debug, thiserror::Error)]
pub enum RenderThreadError {
    #[error("Failed to spawn render thread: {0}")]
    ThreadSpawnFailed(#[from] std::io::Error),
}
