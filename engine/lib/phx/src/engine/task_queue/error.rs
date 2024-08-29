use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskQueueError {
    #[error("Thread error: {0}")]
    ThreadError(String),
    #[error("Lua error: {0}")]
    LuaError(#[from] mlua::Error),
}
