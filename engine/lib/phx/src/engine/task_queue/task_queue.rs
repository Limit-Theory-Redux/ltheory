use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use mlua::{Function, Lua};
use tracing::error;

use super::{TaskResult, Worker, WorkerId, WorkerInData, WorkerOutData, WorkerThread};
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

#[luajit_ffi_gen::luajit_ffi]
impl TaskQueue {
    pub fn start_worker(&mut self, worker_id: u8, script_path: &str) -> bool {
        if self.lua_workers.contains_key(&worker_id) {
            error!("Worker with id {worker_id} already exists");
            return false;
        }

        let script_path = PathBuf::from(script_path);
        if !script_path.exists() {
            error!("Script path doesn't exist: {}", script_path.display());
            return false;
        }

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
                            WorkerInData::Data(task_id, data) => {
                                let out_data = run_func.call(data)?;
                                WorkerOutData::Data(task_id, out_data)
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

        true
    }

    pub fn stop_worker(&self, worker_id: u8) -> bool {
        if let Some(worker) = self.lua_workers.get(&worker_id) {
            if let Err(err) = worker.stop() {
                error!("Cannot stop worker {worker_id}. Error: {err}");
                false
            } else {
                true
            }
        } else {
            error!("Unknown worker: {worker_id}");
            false
        }
    }

    pub fn tasks_in_progress(&self, worker_id: u8) -> Option<usize> {
        if let Some(worker) = Worker::from_worker_id(worker_id) {
            match worker {
                Worker::Echo => Some(self.echo_worker.tasks_in_progress()),
                Worker::EngineWorkersCount => unreachable!(),
            }
        } else if let Some(worker) = self.lua_workers.get(&worker_id) {
            Some(worker.tasks_in_progress())
        } else {
            error!("Unknown worker: {worker_id}");
            None
        }
    }

    pub fn send_task(&mut self, worker_id: u8, task_id: usize, data: Payload) -> bool {
        if let Some(worker) = self.lua_workers.get_mut(&worker_id) {
            if let Err(err) = worker.send(task_id, data) {
                error!("Cannot send task to worker {worker_id}. Error: {err}");
                false
            } else {
                true
            }
        } else {
            error!("Unknown worker: {worker_id}");
            false
        }
    }

    pub fn next_task_result(&mut self, worker_id: u8) -> Option<TaskResult> {
        if let Some(worker) = self.lua_workers.get_mut(&worker_id) {
            match worker.recv() {
                Ok(res) => res.map(|(task_id, data)| TaskResult::new(worker_id, task_id, data)),
                Err(err) => {
                    error!("Cannot send task to worker {worker_id}. Error: {err}");
                    None
                }
            }
        } else {
            error!("Unknown worker: {worker_id}");
            None
        }
    }

    pub fn send_echo(&mut self, data: &str) -> bool {
        if let Err(err) = self.echo_worker.send(0, data.into()) {
            error!("Cannot send message to the echo worker. Error: {err}");
            false
        } else {
            true
        }
    }

    pub fn get_echo(&mut self) -> Option<String> {
        match self.echo_worker.recv() {
            Ok(res) => res.map(|(_, data)| data),
            Err(err) => {
                error!("Cannot get echo message. Error: {err}");
                None
            }
        }
    }
}
