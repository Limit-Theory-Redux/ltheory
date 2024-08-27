use std::path::Path;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use mlua::{Function, Lua};
use tracing::error;

use super::{TaskQueueError, WorkerInData, WorkerOutData, WorkerThread};
use crate::engine::Payload;

pub struct LuaWorker {
    in_sender: Sender<WorkerInData<Payload>>,
    out_receiver: Receiver<WorkerOutData<Payload>>,
    handle: Option<JoinHandle<Result<(), mlua::Error>>>,
}

impl Drop for LuaWorker {
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

impl LuaWorker {
    pub fn new(script_path: &Path) -> Result<Self, TaskQueueError> {
        let script_path = script_path.to_path_buf();
        let (in_sender, in_receiver) = channel();
        let (out_sender, out_receiver) = channel();

        let handle = thread::spawn(move || {
            let lua = unsafe { Lua::unsafe_new() };

            lua.load(script_path).exec()?;

            let globals = lua.globals();
            let run_func: Function = globals.get("run")?;

            loop {
                let res: Result<WorkerInData<Payload>, _> =
                    in_receiver.recv_timeout(Duration::from_millis(500));
                match res {
                    Ok(in_data) => {
                        let data = match in_data {
                            WorkerInData::Ping => WorkerOutData::Pong,
                            WorkerInData::Data(data) => {
                                let out_data = run_func.call(data)?;
                                WorkerOutData::Data(out_data)
                            }
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
            }

            Ok(())
        });

        Ok(Self {
            in_sender,
            out_receiver,
            handle: Some(handle),
        })
    }
}

impl WorkerThread<Payload, Payload> for LuaWorker {
    fn send(&self, data: Payload) -> Result<(), TaskQueueError> {
        self.in_sender.send(WorkerInData::Data(data)).map_err(|_| {
            TaskQueueError::ThreadError("Cannot send data to the worker thread".into())
        })
    }

    fn recv(&self) -> Result<Option<Payload>, TaskQueueError> {
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
