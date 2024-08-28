pub type TaskId = usize;

pub enum WorkerInData<T> {
    Ping,
    Data(TaskId, T),
    Stop,
}

pub enum WorkerOutData<T> {
    Pong,
    Data(TaskId, T),
}
