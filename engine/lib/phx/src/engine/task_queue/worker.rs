use std::sync::Arc;
use std::time::Duration;

use crossbeam::channel::{unbounded, Receiver, RecvTimeoutError, Sender};
use tracing::debug;

use super::{TaskId, TaskQueueError, WorkerInData, WorkerInstance, WorkerOutData};

const RECEIVE_TIMEOUT: Duration = Duration::from_millis(500);

/// Worker thread template.
pub struct Worker<IN, OUT> {
    name: String,
    next_task_id: TaskId,
    tasks_in_work: usize,

    instances: Vec<WorkerInstance>,

    in_sender: Sender<WorkerInData<IN>>,
    out_receiver: Receiver<WorkerOutData<OUT>>,
}

impl<IN: Send + 'static, OUT: Send + 'static> Worker<IN, OUT> {
    /// Creates custom worker.
    pub fn new<F>(name: &str, instances_count: usize, f: F) -> Self
    where
        F: Fn(Receiver<WorkerInData<IN>>, Sender<WorkerOutData<OUT>>) -> Result<(), TaskQueueError>,
        F: Send + Sync + 'static, // TODO: check if Sync is really needed
    {
        let (in_sender, in_receiver) = unbounded();
        let (out_sender, out_receiver) = unbounded();

        debug!("Starting worker: {name:?}");

        let mut instances = Vec::with_capacity(instances_count);

        let f_arc = Arc::new(f);

        for instance_id in 0..instances_count {
            let instance = WorkerInstance::new(
                instance_id,
                in_receiver.clone(),
                out_sender.clone(),
                f_arc.clone(),
            );

            instances.push(instance);
        }

        debug!("Worker {name:?} was successfully started");

        Self {
            name: name.into(),
            next_task_id: 0,
            tasks_in_work: 0,
            instances,

            in_sender,
            out_receiver,
        }
    }

    /// Create function based native worker.
    pub fn new_native<F>(name: &str, instances_count: usize, f: F) -> Self
    where
        F: Fn(IN) -> OUT,
        F: Send + Sync + 'static,
    {
        let worker_name = name.to_string();
        Self::new(name, instances_count, move |in_receiver, out_sender| {
            loop {
                let res: Result<_, _> = in_receiver.recv_timeout(RECEIVE_TIMEOUT);
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

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn tasks_waiting(&self)->usize{
        self.in_sender.len()
    }

    #[inline]
    pub fn tasks_in_progress(&self) -> usize {
        self.tasks_in_work - self.tasks_waiting() - self.tasks_ready()
    }

    #[inline]
    pub fn tasks_ready(&self)->usize{
        self.out_receiver.len()
    }

    /// Checks if the associated worker thread has finished running its main function.
    pub fn is_finished(&self) -> bool {
        self.instances.iter().all(|instance| instance.is_finished())
    }

    /// Send stop signal to the worker thread.
    pub fn stop(&self) -> Result<(), TaskQueueError> {
        // send stop signal as many times as there are instances, so each instance receives one
        for _ in 0..self.instances.len() {
            self.in_sender
                .send(WorkerInData::Stop)
                .map_err(|_| TaskQueueError::ThreadError("Cannot send stop message".into()))?;
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
                    "Cannot send data to the worker: {:?}",
                    self.name
                ))
            })?;

        self.tasks_in_work += 1;

        self.next_task_id += 1;

        Ok(task_id)
    }

    /// Get result from the worker thread if any.
    pub fn recv(&mut self) -> Result<Option<(TaskId, OUT)>, TaskQueueError> {
        match self.out_receiver.recv_timeout(RECEIVE_TIMEOUT) {
            Ok(out_data) => match out_data {
                WorkerOutData::Pong => Ok(None),
                WorkerOutData::Data(task_id, data) => {
                    self.tasks_in_work -= 1;
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Worker, RECEIVE_TIMEOUT};

    #[test]
    fn test_worker_new_native() {
        let mut worker: Worker<String, String> = Worker::new_native("TestWorker", 1, |in_data| {
            std::thread::sleep(Duration::from_millis(300));
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

        std::thread::sleep(RECEIVE_TIMEOUT);

        assert!(worker.is_finished());
    }
}
