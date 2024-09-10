use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crossbeam::channel::{Receiver, Sender};
use tracing::error;

use super::{TaskQueueError, WorkerInData, WorkerOutData};

/// Worker instance template.
pub struct WorkerInstance {
    id: usize,
    handle: Option<JoinHandle<Result<(), TaskQueueError>>>,
}

impl WorkerInstance {
    /// Creates custom worker instance.
    pub fn new<F, IN, OUT>(
        id: usize,
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
                error!("Failed to execute task in the worker instance {id}. Error: {err}");
            }

            res
        });

        Self {
            id,
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
                            error!("Worker instance {} failed on drop. Error: {err}", self.id);
                        }
                    }
                    Err(err) => {
                        error!(
                            "Cannot finish worker instance {} properly. Error: {err:?}",
                            self.id
                        );
                    }
                }
            }
        }
    }
}
