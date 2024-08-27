use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tracing::error;

use super::{TaskQueueError, WorkerInData, WorkerOutData, WorkerThread};

pub struct NativeWorker<IN, OUT> {
    in_sender: Sender<WorkerInData<IN>>,
    out_receiver: Receiver<WorkerOutData<OUT>>,
    handle: Option<JoinHandle<()>>,
}

impl<IN, OUT> Drop for NativeWorker<IN, OUT> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            // TODO: check leftover data in the out receiver
            if self.in_sender.send(WorkerInData::Stop).is_err() {
                error!("Cannot stop thread");
                return;
            }
            if let Err(err) = handle.join() {
                error!("Cannot finish worker thread properly. Error: {err:?}");
            }
        }
    }
}

impl<IN: Send + 'static, OUT: Send + 'static> NativeWorker<IN, OUT> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(IN) -> OUT,
        F: Send + 'static,
    {
        let (in_sender, in_receiver) = channel();
        let (out_sender, out_receiver) = channel();

        let handle = thread::spawn(move || loop {
            let res: Result<WorkerInData<IN>, _> =
                in_receiver.recv_timeout(Duration::from_millis(500));
            match res {
                Ok(in_data) => {
                    let data = match in_data {
                        WorkerInData::Ping => WorkerOutData::Pong,
                        WorkerInData::Data(data) => WorkerOutData::Data(f(data)),
                        WorkerInData::Stop => break,
                    };

                    if out_sender.send(data).is_err() {
                        break;
                    }
                }
                Err(err) => match err {
                    RecvTimeoutError::Timeout => continue,
                    RecvTimeoutError::Disconnected => break,
                },
            }
        });

        Self {
            in_sender,
            out_receiver,
            handle: Some(handle),
        }
    }
}

impl<IN, OUT> WorkerThread<IN, OUT> for NativeWorker<IN, OUT> {
    fn send(&self, data: IN) -> Result<(), TaskQueueError> {
        self.in_sender.send(WorkerInData::Data(data)).map_err(|_| {
            TaskQueueError::ThreadError("Cannot send data to the worker thread".into())
        })
    }

    fn recv(&self) -> Result<Option<OUT>, TaskQueueError> {
        match self.out_receiver.recv_timeout(Duration::from_millis(500)) {
            Ok(out_data) => match out_data {
                WorkerOutData::Pong => Ok(None),
                WorkerOutData::Data(data) => Ok(Some(data)),
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
