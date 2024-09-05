pub type TaskId = usize;

/// Variants of the worker input data.
pub enum WorkerInData<T> {
    Ping,
    Data(TaskId, T),
    Stop,
}

/// Variants of the worker output data.
pub enum WorkerOutData<T> {
    Pong,
    Data(TaskId, T),
}
