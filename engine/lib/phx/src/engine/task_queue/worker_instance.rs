use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crossbeam::channel::{Receiver, Sender};
use tracing::error;

use super::{TaskQueueError, WorkerInData, WorkerOutData};

/// Worker instance template.
pub struct WorkerInstance {
    handle: Option<JoinHandle<Result<(), TaskQueueError>>>,
}

impl WorkerInstance {
    /// Creates custom worker instance.
    pub fn new<F, IN, OUT>(
        in_receiver: Receiver<WorkerInData<IN>>,
        out_sender: Sender<WorkerOutData<OUT>>,
        f: Arc<F>,
    ) -> Self
    where
        IN: Send + 'static,
        OUT: Send + 'static,
        F: Fn(Receiver<WorkerInData<IN>>, Sender<WorkerOutData<OUT>>) -> Result<(), TaskQueueError>,
        F: Send + Sync + 'static, // TODO: check if Sync is really needed
    {
        let handle = thread::spawn(move || {
            let res = f(in_receiver, out_sender);

            if let Err(err) = &res {
                error!("Failed to execute task in the worker instance. Error: {err}");
            }

            res
        });

        Self {
            handle: Some(handle),
        }
    }

    /// Checks if the associated worker instance has finished running its main function.
    pub fn is_finished(&self) -> bool {
        self.handle
            .as_ref()
            .map(|h| h.is_finished())
            .unwrap_or(true)
    }
}

impl Drop for WorkerInstance {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            if !handle.is_finished() {
                // TODO: what to do with a hanging thread?
                match handle.join() {
                    Ok(res) => {
                        if let Err(err) = res {
                            error!("Worker instance failed on drop. Error: {err}");
                        }
                    }
                    Err(err) => {
                        error!("Cannot finish worker instance properly. Error: {err:?}");
                    }
                }
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use std::time::Duration;

//     use super::{WorkerInstance, RECEIVE_TIMEOUT};

//     #[test]
//     fn test_worker_new_native() {
//         let mut worker: WorkerInstance<String, String> =
//             WorkerInstance::new_native("TestWorker", |in_data| {
//                 std::thread::sleep(Duration::from_millis(300));
//                 in_data
//             });

//         assert_eq!("TestWorker", worker.name());

//         let task_id = worker.send("TestData".into()).expect("Cannot send task");

//         assert_eq!(1, worker.tasks_in_progress());

//         let (result_task_id, result_data) = worker
//             .recv()
//             .expect("Cannot receive task result")
//             .expect("Task result is not ready");

//         assert_eq!(task_id, result_task_id, "Task id is different");
//         assert_eq!("TestData", result_data, "Task result data");

//         worker.stop().expect("Cannot stop worker");

//         std::thread::sleep(RECEIVE_TIMEOUT);

//         assert!(worker.is_finished());
//     }
// }
