use super::{TaskId, WorkerId};
use crate::engine::Payload;

pub struct TaskResult {
    worker_id: WorkerId,
    task_id: TaskId,
    data: Payload,
}

impl TaskResult {
    pub fn new(worker_id: WorkerId, task_id: TaskId, data: Payload) -> Self {
        Self {
            worker_id,
            task_id,
            data,
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

    pub fn data(&self) -> &Payload {
        &self.data
    }
}
