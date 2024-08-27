use super::TaskQueueError;

pub trait WorkerThread<IN, OUT> {
    fn send(&self, data: IN) -> Result<(), TaskQueueError>;
    fn recv(&self) -> Result<Option<OUT>, TaskQueueError>;
}
