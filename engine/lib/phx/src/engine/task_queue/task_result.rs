use super::{TaskId, WorkerId};
use crate::engine::Payload;

enum TaskResultData {
    Payload(Box<Payload>),
    Error(String),
}

pub struct TaskResult {
    worker_id: WorkerId,
    task_id: TaskId,
    data: TaskResultData,
}

impl TaskResult {
    pub fn new(worker_id: WorkerId, task_id: TaskId, payload: Box<Payload>) -> Self {
        Self {
            worker_id,
            task_id,
            data: TaskResultData::Payload(payload),
        }
    }

    pub fn new_error(worker_id: WorkerId, task_id: TaskId, msg: &str) -> Self {
        Self {
            worker_id,
            task_id,
            data: TaskResultData::Error(msg.into()),
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl TaskResult {
    pub fn worker_id(&self) -> u8 {
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
