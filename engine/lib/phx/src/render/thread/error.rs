#[derive(Debug, thiserror::Error)]
pub enum RenderThreadError {
    #[error("failed to activate GL context on render thread: {0}")]
    ContextActivationFailed(#[from] crate::window::WindowError),
}
