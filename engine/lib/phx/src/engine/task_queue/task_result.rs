use super::{TaskId, WorkerIndex};
use crate::engine::Payload;

/// Task execution result data: payload on success or error message otherwise.
enum TaskResultData {
    Payload(Box<Payload>),
    Error(String),
    /// The worker function returned nothing (nil).
    Empty,
}

/// Task result information.
pub struct TaskResult {
    worker_id: WorkerIndex,
    task_id: TaskId,
    data: TaskResultData,
}

impl TaskResult {
    /// Create a success result.
    pub fn new(worker_id: WorkerIndex, task_id: TaskId, payload: Box<Payload>) -> Self {
        Self {
            worker_id,
            task_id,
            data: TaskResultData::Payload(payload),
        }
    }

    /// Create a result without data.
    pub fn new_empty(worker_id: WorkerIndex, task_id: TaskId) -> Self {
        Self {
            worker_id,
            task_id,
            data: TaskResultData::Empty,
        }
    }

    /// Create an error result.
    pub fn new_error(worker_id: WorkerIndex, task_id: TaskId, msg: &str) -> Self {
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
            TaskResultData::Error(_) | TaskResultData::Empty => None,
        }
    }

    pub fn error(&self) -> Option<&str> {
        match &self.data {
            TaskResultData::Payload(_) | TaskResultData::Empty => None,
            TaskResultData::Error(err) => Some(err.as_str()),
        }
    }
}
