#[derive(Debug, thiserror::Error)]
pub enum RenderThreadError {
    #[error("Failed to spawn render thread: {0}")]
    ThreadSpawnFailed(#[from] std::io::Error),
    #[error("Render thread did not report context-activation status")]
    ContextActivationSatusFailed(#[from] crossbeam::channel::RecvError),
    #[error("Failed to activate GL context on render thread: {0}")]
    ContextActivationFailed(#[from] crate::window::WindowError),
}
