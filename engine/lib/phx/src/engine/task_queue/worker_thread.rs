use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tracing::error;

use super::{TaskId, TaskQueueError, WorkerInData, WorkerOutData};

pub struct WorkerThread<IN, OUT> {
    in_sender: Sender<WorkerInData<IN>>,
    out_receiver: Receiver<WorkerOutData<OUT>>,
    handle: Option<JoinHandle<Result<(), TaskQueueError>>>,
    tasks_in_progress: usize,
}

impl<IN: Send + 'static, OUT: Send + 'static> WorkerThread<IN, OUT> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(
            Receiver<WorkerInData<IN>>,
            Sender<WorkerOutData<OUT>>,
        ) -> Result<(), TaskQueueError>,
        F: Send + 'static,
    {
        let (in_sender, in_receiver) = channel();
        let (out_sender, out_receiver) = channel();

        let handle = thread::spawn(move || f(in_receiver, out_sender));

        Self {
            in_sender,
            out_receiver,
            handle: Some(handle),
            tasks_in_progress: 0,
        }
    }

    pub fn new_native<F>(f: F) -> Self
    where
        F: Fn(IN) -> OUT,
        F: Send + 'static,
    {
        let (in_sender, in_receiver) = channel();
        let (out_sender, out_receiver) = channel();

        let handle = thread::spawn(move || {
            loop {
                let res: Result<WorkerInData<IN>, _> =
                    in_receiver.recv_timeout(Duration::from_millis(500));
                match res {
                    Ok(in_data) => {
                        let data = match in_data {
                            WorkerInData::Ping => WorkerOutData::Pong,
                            WorkerInData::Data(task_id, data) => {
                                WorkerOutData::Data(task_id, f(data))
                            }
                            WorkerInData::Stop => break,
                        };

                        if out_sender.send(data).is_err() {
                            return Err(TaskQueueError::ThreadError(
                                "Cannot send data to the worker".into(),
                            ));
                        }
                    }
                    Err(err) => match err {
                        RecvTimeoutError::Timeout => continue,
                        RecvTimeoutError::Disconnected => {
                            return Err(TaskQueueError::ThreadError(
                                "Worker is disconnected".into(),
                            ))
                        }
                    },
                }
            }
            Ok(())
        });

        Self {
            in_sender,
            out_receiver,
            handle: Some(handle),
            tasks_in_progress: 0,
        }
    }

    pub fn tasks_in_progress(&self) -> usize {
        self.tasks_in_progress
    }

    pub fn stop(&self) -> Result<(), TaskQueueError> {
        self.in_sender
            .send(WorkerInData::Stop)
            .map_err(|_| TaskQueueError::ThreadError("Cannot stop worker thread".into()))
    }

    pub fn send(&mut self, task_id: TaskId, data: IN) -> Result<(), TaskQueueError> {
        self.in_sender
            .send(WorkerInData::Data(task_id, data))
            .map_err(|_| {
                TaskQueueError::ThreadError("Cannot send data to the worker thread".into())
            })?;
        self.tasks_in_progress += 1;
        Ok(())
    }

    pub fn recv(&mut self) -> Result<Option<(TaskId, OUT)>, TaskQueueError> {
        match self.out_receiver.recv_timeout(Duration::from_millis(500)) {
            Ok(out_data) => match out_data {
                WorkerOutData::Pong => Ok(None),
                WorkerOutData::Data(task_id, data) => {
                    self.tasks_in_progress -= 1;
                    Ok(Some((task_id, data)))
                }
            },
            Err(err) => match err {
                RecvTimeoutError::Timeout => Ok(None),
                RecvTimeoutError::Disconnected => Err(TaskQueueError::ThreadError(
                    "Worker thread is disconnected".into(),
                )),
            },
        }
    }
}

impl<IN, OUT> Drop for WorkerThread<IN, OUT> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            // TODO: check leftover data in the out receiver

            if self.in_sender.send(WorkerInData::Stop).is_err() {
                error!("Cannot stop thread");
            }

            match handle.join() {
                Ok(res) => {
                    if let Err(err) = res {
                        error!("Worker thread failed. Error: {err}");
                    }
                }
                Err(err) => {
                    error!("Cannot finish worker thread properly. Error: {err:?}");
                }
            }
        }
    }
}
