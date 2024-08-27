use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskQueueError {
    #[error("Thread error")]
    ThreadError(String),
    #[error("Lua error")]
    LuaError(#[from] mlua::Error),
}
