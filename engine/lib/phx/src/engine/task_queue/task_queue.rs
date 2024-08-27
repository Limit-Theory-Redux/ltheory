use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use mlua::{Function, Lua};
use tracing::error;

use super::{WorkerId, WorkerInData, WorkerOutData, WorkerThread};
use crate::engine::Payload;

pub struct TaskQueue {
    lua_workers: HashMap<WorkerId, WorkerThread<Payload, Payload>>,
    echo_worker: WorkerThread<String, String>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            lua_workers: HashMap::new(),
            echo_worker: WorkerThread::new_native(|data| data),
        }
    }
}

impl TaskQueue {
    pub fn start_worker(&mut self, worker_id: u8, script_path: &Path) {
        let script_path = script_path.to_path_buf();
        let worker_thread = WorkerThread::new(move |in_receiver, out_sender| {
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

        self.lua_workers.insert(worker_id, worker_thread);
    }

    pub fn send_job(&self, worker_id: u8, job_id: usize, data: Payload) -> bool {
        if let Some(worker) = self.lua_workers.get(&worker_id) {
            if let Err(err) = worker.send(data) {
                error!("Cannot send job to worker {worker_id}. Error: {err}");
                false
            } else {
                true
            }
        } else {
            error!("Unknown worker: {worker_id}");
            false
        }
    }
}
