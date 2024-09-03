use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use mlua::{Function, Lua};
use tracing::{debug, error};

use super::{TaskResult, Worker, WorkerId, WorkerInData, WorkerOutData, WorkerThread};
use crate::engine::{Payload, PayloadType};

pub struct TaskQueue {
    lua_workers: HashMap<WorkerId, WorkerThread<Payload, Box<Payload>>>,
    echo_worker: WorkerThread<String, String>,
}

impl Drop for TaskQueue {
    fn drop(&mut self) {
        self.stop_all_workers();
    }
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            lua_workers: HashMap::new(),
            echo_worker: WorkerThread::new_native("Echo", |data| data),
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl TaskQueue {
    pub fn start_worker(&mut self, worker_id: u16, worker_name: &str, script_path: &str) -> bool {
        debug!("Starting worker: {worker_name:?}");

        if self.lua_workers.contains_key(&worker_id) {
            error!("Worker with id {worker_id} already exists");
            return false;
        }

        let script_path = PathBuf::from(script_path);
        if !script_path.exists() {
            error!(
                "Script path doesn't exist: {}. Current directory: {:?}",
                script_path.display(),
                std::env::current_dir()
            );
            return false;
        }

        let worker_name_copy = worker_name.to_string();
        let worker_thread = WorkerThread::new(worker_name, move |in_receiver, out_sender| {
            let lua = unsafe { Lua::unsafe_new() };

            lua.load(script_path).exec()?;

            let globals = lua.globals();
            let run_func: Function = globals.get("Run")?;

            loop {
                let res: Result<WorkerInData<Payload>, _> =
                    in_receiver.recv_timeout(Duration::from_millis(500));
                match res {
                    Ok(in_data) => {
                        let data = match in_data {
                            WorkerInData::Ping => WorkerOutData::Pong,
                            WorkerInData::Data(task_id, data) => {
                                debug!("Worker {worker_name_copy} received[{task_id}]: {data:?}");

                                // put data on the heap
                                let boxed_data = Box::new(data);

                                // send data pointer to the Lua script and transfer ownership
                                // receive a pointer to the response payload in form of integer
                                let boxed_out_data: usize =
                                    run_func.call(Box::leak(boxed_data) as *mut Payload as usize)?;

                                // transfer ownership of the payload from the script to the engine in form of boxed data
                                let out_data =
                                    unsafe { Box::from_raw(boxed_out_data as *mut Payload) };

                                WorkerOutData::Data(task_id, out_data)
                            }
                            WorkerInData::Stop => {
                                debug!("Worker {worker_name_copy:?} was stopped");
                                break;
                            }
                        };

                        if out_sender.send(data).is_err() {
                            error!("Cannot send response. Worker: {worker_name_copy}");
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

        debug!("Worker: {worker_name:?} started");

        true
    }

    pub fn stop_worker(&self, worker_id: u16) -> bool {
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

    pub fn is_worker_finished(&self, worker_id: u16) -> bool {
        if let Some(worker) = Worker::from_worker_id(worker_id) {
            match worker {
                Worker::Echo => self.echo_worker.is_finished(),
                Worker::EngineWorkersCount => unreachable!(),
            }
        } else if let Some(worker) = self.lua_workers.get(&worker_id) {
            worker.is_finished()
        } else {
            error!("Unknown worker: {worker_id}");
            true
        }
    }

    pub fn stop_all_workers(&self) {
        debug!("Stopping all workers");

        self.echo_worker
            .stop()
            .unwrap_or_else(|err| error!("Cannot stop echo worker. Error: {err}"));

        for worker in self.lua_workers.values() {
            worker.stop().unwrap_or_else(|err| {
                error!("Cannot stop worker: {}. Error: {err}", worker.name())
            });
        }

        debug!("All workers were stopped");
    }

    pub fn tasks_in_progress(&self, worker_id: u16) -> Option<usize> {
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

    pub fn send_task(&mut self, worker_id: u16, data: Payload) -> Option<usize> {
        if data.get_type() == PayloadType::Lua {
            error!("Cannot send cached Lua payload to the worker");
            return None;
        }

        if let Some(worker) = self.lua_workers.get_mut(&worker_id) {
            match worker.send(data) {
                Ok(task_id) => {
                    debug!("Task {task_id} sent to worker {:?}", worker.name());
                    Some(task_id)
                }
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

    pub fn next_task_result(&mut self, worker_id: u16) -> Option<TaskResult> {
        if let Some(worker) = self.lua_workers.get_mut(&worker_id) {
            match worker.recv() {
                Ok(res) => res.map(|(task_id, data)| {
                    if data.get_type() == PayloadType::Lua {
                        error!("Cannot receive cached Lua payload from the worker");
                        TaskResult::new_error(
                            worker_id,
                            task_id,
                            "Cannot receive cached Lua payload from the worker",
                        )
                    } else {
                        debug!(
                            "Received task {task_id} result for worker {:?}",
                            worker.name()
                        );
                        TaskResult::new(worker_id, task_id, data)
                    }
                }),
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
        if let Err(err) = self.echo_worker.send(data.into()) {
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
