use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crossbeam::channel::RecvTimeoutError;
use mlua::{Function, Lua};
use tracing::{debug, error};

use super::{TaskResult, Worker, WorkerId, WorkerInData, WorkerIndex, WorkerOutData};
use crate::engine::{Payload, PayloadType};

/// Task queue is a worker threads manager.
/// It can be used to start either custom Lua scripts in a separate threads or predefined engine workers.
/// When started workers can accept tasks and return their results.
pub struct TaskQueue {
    lua_workers: HashMap<WorkerIndex, Worker<Payload, Box<Payload>>>,
    echo_worker: Worker<String, String>,
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
            echo_worker: Worker::new_native("Echo", 1, |data| data),
        }
    }
}

/// Task queue is a worker threads manager.
/// It can be used to start either custom Lua scripts in a separate threads or predefined engine workers.
/// When started workers can accept tasks and return their results.
#[luajit_ffi_gen::luajit_ffi]
impl TaskQueue {
    /// Start Lua worker with provided script file.
    pub fn start_worker(
        &mut self,
        worker_id: u16,
        worker_name: &str,
        script_path: &str,
        instances_count: usize,
    ) -> bool {
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
        let worker_thread = Worker::new(
            worker_name,
            instances_count,
            move |in_receiver, out_sender| {
                let lua = unsafe { Lua::unsafe_new() };

                lua.load(script_path.as_path()).exec()?;

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
                                    debug!(
                                        "Worker {worker_name_copy} received[{task_id}]: {data:?}"
                                    );

                                    // put data on the heap
                                    let boxed_data = Box::new(data);

                                    // send data pointer to the Lua script and transfer ownership
                                    // receive a pointer to the response payload in form of integer
                                    let boxed_out_data: usize = run_func
                                        .call(Box::leak(boxed_data) as *mut Payload as usize)?;

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
            },
        );

        self.lua_workers.insert(worker_id, worker_thread);

        debug!("Worker: {worker_name:?} started");

        true
    }

    /// Stop Lua worker and remove it from the queue.
    pub fn stop_worker(&mut self, worker_id: u16) -> bool {
        if let Some(worker) = self.lua_workers.remove(&worker_id) {
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

    /// Stop all Lua workers and remove them from the queue.
    pub fn stop_all_workers(&mut self) {
        debug!("Stopping all Lua workers");

        for (_, worker) in self.lua_workers.drain() {
            worker.stop().unwrap_or_else(|err| {
                error!("Cannot stop worker: {}. Error: {err}", worker.name())
            });
        }

        debug!("All Lua workers were stopped");
    }

    /// Returns number of tasks that were sent to the worker and whose results are not retrieved yet.
    pub fn tasks_in_work(&self, worker_id: u16) -> Option<usize> {
        if let Some(worker) = WorkerId::from_worker_id(worker_id) {
            match worker {
                WorkerId::Echo => Some(self.echo_worker.tasks_in_work()),
                WorkerId::EngineWorkersCount => unreachable!(),
            }
        } else if let Some(worker) = self.lua_workers.get(&worker_id) {
            Some(worker.tasks_in_work())
        } else {
            error!("Unknown worker: {worker_id}");
            None
        }
    }

    /// Returns number of tasks waiting to be processed by the worker.
    pub fn tasks_waiting(&self, worker_id: u16) -> Option<usize> {
        if let Some(worker) = WorkerId::from_worker_id(worker_id) {
            match worker {
                WorkerId::Echo => Some(self.echo_worker.tasks_waiting()),
                WorkerId::EngineWorkersCount => unreachable!(),
            }
        } else if let Some(worker) = self.lua_workers.get(&worker_id) {
            Some(worker.tasks_waiting())
        } else {
            error!("Unknown worker: {worker_id}");
            None
        }
    }

    /// Returns number of tasks the worker is busy with.
    pub fn tasks_in_progress(&self, worker_id: u16) -> Option<usize> {
        if let Some(worker) = WorkerId::from_worker_id(worker_id) {
            match worker {
                WorkerId::Echo => Some(self.echo_worker.tasks_in_progress()),
                WorkerId::EngineWorkersCount => unreachable!(),
            }
        } else if let Some(worker) = self.lua_workers.get(&worker_id) {
            Some(worker.tasks_in_progress())
        } else {
            error!("Unknown worker: {worker_id}");
            None
        }
    }

    /// Returns number of tasks finished by the worker and whose results can be retrieved.
    pub fn tasks_ready(&self, worker_id: u16) -> Option<usize> {
        if let Some(worker) = WorkerId::from_worker_id(worker_id) {
            match worker {
                WorkerId::Echo => Some(self.echo_worker.tasks_ready()),
                WorkerId::EngineWorkersCount => unreachable!(),
            }
        } else if let Some(worker) = self.lua_workers.get(&worker_id) {
            Some(worker.tasks_ready())
        } else {
            error!("Unknown worker: {worker_id}");
            None
        }
    }

    /// Send a task to the Lua worker.
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

    /// Returns next result of the finished worker task if any.
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

    /// Send a message to the echo worker.
    pub fn send_echo(&mut self, data: &str) -> bool {
        if let Err(err) = self.echo_worker.send(data.into()) {
            error!("Cannot send message to the echo worker. Error: {err}");
            false
        } else {
            true
        }
    }

    /// Get a response from the echo worker.
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
