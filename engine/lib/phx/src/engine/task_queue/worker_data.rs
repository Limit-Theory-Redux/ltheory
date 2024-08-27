pub type JobId = usize;

pub enum WorkerInData<T> {
    Ping,
    Data(T),
    Stop,
}

pub enum WorkerOutData<T> {
    Pong,
    Data(T),
}
