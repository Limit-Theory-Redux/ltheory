use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tracing::{debug, error, warn};

use super::{TaskId, TaskQueueError, WorkerInData, WorkerOutData};

/// Worker thread template.
pub struct WorkerThread<IN, OUT> {
    name: String,
    in_sender: Sender<WorkerInData<IN>>,
    out_receiver: Receiver<WorkerOutData<OUT>>,
    handle: Option<JoinHandle<Result<(), TaskQueueError>>>,
    next_task_id: TaskId,
    tasks_in_progress: usize,
}

impl<IN: Send + 'static, OUT: Send + 'static> WorkerThread<IN, OUT> {
    /// Creates custom worker thread.
    pub fn new<F>(name: &str, f: F) -> Self
    where
        F: FnOnce(
            Receiver<WorkerInData<IN>>,
            Sender<WorkerOutData<OUT>>,
        ) -> Result<(), TaskQueueError>,
        F: Send + 'static,
    {
        let worker_name = name.to_string();
        let (in_sender, in_receiver) = channel();
        let (out_sender, out_receiver) = channel();

        debug!("Starting worker thread: {name:?}");

        let handle = thread::spawn(move || {
            let res = f(in_receiver, out_sender);

            if let Err(err) = &res {
                error!("Failed to execute task in the worker {worker_name:?}. Error: {err}");
            }

            res
        });

        debug!("Worker thread {name:?} was successfully started");

        Self {
            name: name.into(),
            in_sender,
            out_receiver,
            handle: Some(handle),
            next_task_id: 0,
            tasks_in_progress: 0,
        }
    }

    /// Create function based native worker thread.
    pub fn new_native<F>(name: &str, f: F) -> Self
    where
        F: Fn(IN) -> OUT,
        F: Send + 'static,
    {
        let worker_name = name.to_string();
        Self::new(name, move |in_receiver, out_sender| {
            loop {
                let res: Result<WorkerInData<IN>, _> =
                    in_receiver.recv_timeout(Duration::from_millis(500));
                match res {
                    Ok(in_data) => {
                        let data = match in_data {
                            WorkerInData::Ping => WorkerOutData::Pong,
                            WorkerInData::Data(task_id, data) => {
                                debug!("Worker {worker_name} received task {task_id}");
                                WorkerOutData::Data(task_id, f(data))
                            }
                            WorkerInData::Stop => {
                                debug!("Worker {worker_name:?} was stopped");
                                break;
                            }
                        };

                        if out_sender.send(data).is_err() {
                            return Err(TaskQueueError::ThreadError(format!(
                                "Cannot send data to the worker {worker_name:?}"
                            )));
                        }
                    }
                    Err(err) => match err {
                        RecvTimeoutError::Timeout => continue,
                        RecvTimeoutError::Disconnected => {
                            return Err(TaskQueueError::ThreadError(format!(
                                "Worker {worker_name:?} is disconnected"
                            )))
                        }
                    },
                }
            }
            Ok(())
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tasks_in_progress(&self) -> usize {
        self.tasks_in_progress
    }

    /// Checks if the associated worker thread has finished running its main function.
    pub fn is_finished(&self) -> bool {
        self.handle
            .as_ref()
            .map(|h| h.is_finished())
            .unwrap_or(true)
    }

    /// Send stop signal to the worker thread.
    pub fn stop(&self) -> Result<(), TaskQueueError> {
        if let Some(handle) = &self.handle {
            if self.tasks_in_progress > 0 {
                warn!(
                    "Worker {:?} still has {} task(s) in progress",
                    self.name, self.tasks_in_progress
                );
            }
            if !handle.is_finished() {
                // TODO: what to do with the hanging thread?
                debug!("Send stop signal to {:?} worker", self.name);
                return self.in_sender.send(WorkerInData::Stop).map_err(|_| {
                    TaskQueueError::ThreadError(format!(
                        "Cannot stop worker thread: {:?}",
                        self.name
                    ))
                });
            }
        } else {
            debug!("Worker {:?} is already stopped", self.name);
        }
        Ok(())
    }

    /// Send a task to the worker thread.
    pub fn send(&mut self, data: IN) -> Result<TaskId, TaskQueueError> {
        let task_id = self.next_task_id;

        self.in_sender
            .send(WorkerInData::Data(task_id, data))
            .map_err(|_| {
                TaskQueueError::ThreadError(format!(
                    "Cannot send data to the worker thread: {:?}",
                    self.name
                ))
            })?;

        self.tasks_in_progress += 1;

        self.next_task_id += 1;

        Ok(task_id)
    }

    /// Get result from the worker thread if any.
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
                RecvTimeoutError::Disconnected => Err(TaskQueueError::ThreadError(format!(
                    "Worker thread {:?} is disconnected",
                    self.name
                ))),
            },
        }
    }
}

impl<IN, OUT> Drop for WorkerThread<IN, OUT> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            if !handle.is_finished() {
                // TODO: check leftover data in the out receiver

                if self.in_sender.send(WorkerInData::Stop).is_err() {
                    error!("Cannot stop worker thread: {:?}", self.name);
                }

                // TODO: what to do with a hanging thread?
                match handle.join() {
                    Ok(res) => {
                        if let Err(err) = res {
                            error!("Worker thread {:?} failed. Error: {err}", self.name);
                        }
                    }
                    Err(err) => {
                        error!(
                            "Cannot finish worker thread {:?} properly. Error: {err:?}",
                            self.name
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::WorkerThread;

    #[test]
    fn test_worker_new_native() {
        let mut worker: WorkerThread<String, String> =
            WorkerThread::new_native("TestWorker", |in_data| {
                std::thread::sleep(Duration::from_millis(500));
                in_data
            });

        assert_eq!("TestWorker", worker.name());

        let task_id = worker.send("TestData".into()).expect("Cannot send task");

        assert_eq!(1, worker.tasks_in_progress());

        let (result_task_id, result_data) = worker
            .recv()
            .expect("Cannot receive task result")
            .expect("Task result is not ready");

        assert_eq!(task_id, result_task_id, "Task id is different");
        assert_eq!("TestData", result_data, "Task result data");

        worker.stop().expect("Cannot stop worker");

        std::thread::sleep(Duration::from_millis(500));

        assert!(worker.is_finished());
    }
}
