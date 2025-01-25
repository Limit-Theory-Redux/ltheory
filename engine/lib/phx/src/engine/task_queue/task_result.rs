use super::{TaskId, WorkerId};
use crate::engine::Payload;

/// Task execution result data: payload on success or error message otherwise.
enum TaskResultData {
    Payload(Box<Payload>),
    Error(String),
}

/// Task result information.
pub struct TaskResult {
    worker_id: WorkerId,
    task_id: TaskId,
    data: TaskResultData,
}

impl TaskResult {
    /// Create a success result.
    pub fn new(worker_id: WorkerId, task_id: TaskId, payload: Box<Payload>) -> Self {
        Self {
            worker_id,
            task_id,
            data: TaskResultData::Payload(payload),
        }
    }

    /// Create an error result.
    pub fn new_error(worker_id: WorkerId, task_id: TaskId, msg: &str) -> Self {
        Self {
            worker_id,
            task_id,
            data: TaskResultData::Error(msg.into()),
        }
    }
}

/// Task result information.
/// Result data can be either payload on success or error message on fail.
#[luajit_ffi_gen::luajit_ffi]
impl TaskResult {
    pub fn worker_id(&self) -> u16 {
        self.worker_id
    }

    pub fn task_id(&self) -> usize {
        self.task_id
    }

    pub fn payload(&self) -> Option<&Payload> {
        match &self.data {
            TaskResultData::Payload(payload) => Some(payload.as_ref()),
            TaskResultData::Error(_) => None,
        }
    }

    pub fn error(&self) -> Option<&str> {
        match &self.data {
            TaskResultData::Payload(_) => None,
            TaskResultData::Error(err) => Some(err.as_str()),
        }
    }
}
