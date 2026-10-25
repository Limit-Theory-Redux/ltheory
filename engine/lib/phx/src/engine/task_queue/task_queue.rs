use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use tracing::{debug, error};

use super::{
    LuaTaskOutput, TaskQueueError, TaskResult, Worker, WorkerBase, WorkerId, WorkerIndex,
    lua_worker_body,
};
use crate::engine::{Payload, PayloadType};

/// Task queue is a worker threads manager.
/// It can be used to start either custom Lua scripts in a separate threads or predefined engine workers.
/// When started workers can accept tasks and return their results.
pub struct TaskQueue {
    lua_workers: HashMap<WorkerIndex, Worker<Payload, LuaTaskOutput>>,
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

    fn process_worker<F, R>(&self, worker_id: WorkerIndex, f: F) -> Result<R, TaskQueueError>
    where
        F: FnOnce(&dyn WorkerBase) -> Result<R, TaskQueueError>,
    {
        if let Some(worker) = WorkerId::from_worker_id(worker_id) {
            match worker {
                WorkerId::Echo => f(&self.echo_worker),
                WorkerId::EngineWorkersCount => unreachable!(),
            }
        } else if let Some(worker) = self.lua_workers.get(&worker_id) {
            f(worker)
        } else {
            Err(TaskQueueError::ThreadError(format!(
                "Unknown worker: {worker_id}"
            )))
        }
    }
}

/// Task queue is a worker threads manager.
/// It can be used to start either custom Lua scripts in a separate threads or predefined engine workers.
/// When started workers can accept tasks and return their results.
#[luajit_ffi_gen::luajit_ffi]
impl TaskQueue {
    /// Start Lua worker with provided script file.
    /// @overload fun(self: table, workerName: string, scriptPath: string, instancesCount: integer): integer
    pub fn start_worker(
        &mut self,
        worker_id: u16,
        worker_name: &str,
        script_path: &str,
        instances_count: usize,
    ) -> bool {
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

        let worker_thread = Worker::new(
            worker_name,
            instances_count,
            lua_worker_body(worker_name.to_string(), script_path, |_| Ok(())),
        );

        self.lua_workers.insert(worker_id, worker_thread);

        true
    }

    /// Stop Lua worker and remove it from the queue.
    pub fn stop_worker(&mut self, worker_id: u16) -> bool {
        match self.process_worker(worker_id, |worker| worker.stop()) {
            Ok(_) => true,
            Err(err) => {
                error!("{err}");
                false
            }
        }
    }

    /// Stop all Lua workers and remove them from the queue.
    pub fn stop_all_workers(&mut self) {
        debug!("Stopping all Lua workers");

        self.echo_worker.stop().unwrap_or_else(|err| {
            error!("Cannot stop Echo worker. {err}");
        });

        for (_, worker) in self.lua_workers.drain() {
            worker.stop().unwrap_or_else(|err| {
                error!("Cannot stop worker: {}. {err}", worker.name());
            });
        }

        debug!("All Lua workers were stopped");
    }

    /// Returns number of tasks that were sent to the worker and whose results are not retrieved yet.
    pub fn tasks_in_work(&self, worker_id: u16) -> Option<usize> {
        match self.process_worker(worker_id, |worker| Ok(worker.tasks_in_work())) {
            Ok(res) => Some(res),
            Err(err) => {
                error!("{err}");
                None
            }
        }
    }

    /// Returns number of tasks waiting to be processed by the worker.
    pub fn tasks_waiting(&self, worker_id: u16) -> Option<usize> {
        match self.process_worker(worker_id, |worker| Ok(worker.tasks_waiting())) {
            Ok(res) => Some(res),
            Err(err) => {
                error!("{err}");
                None
            }
        }
    }

    /// Returns number of tasks the worker is busy with.
    pub fn tasks_in_progress(&self, worker_id: u16) -> Option<usize> {
        match self.process_worker(worker_id, |worker| Ok(worker.tasks_in_progress())) {
            Ok(res) => Some(res),
            Err(err) => {
                error!("{err}");
                None
            }
        }
    }

    /// Returns number of tasks finished by the worker and whose results can be retrieved.
    pub fn tasks_ready(&self, worker_id: u16) -> Option<usize> {
        match self.process_worker(worker_id, |worker| Ok(worker.tasks_ready())) {
            Ok(res) => Some(res),
            Err(err) => {
                error!("{err}");
                None
            }
        }
    }

    /// Send a task to the Lua worker.
    /// @overload fun(workerId: integer, data: Payload|boolean|integer|number|string): integer?
    pub fn send_task(&mut self, worker_id: u16, data: Payload) -> Option<usize> {
        if data.get_type() == PayloadType::Lua {
            error!("Cannot send cached Lua payload to the worker");
            return None;
        }

        if let Some(worker) = self.lua_workers.get_mut(&worker_id) {
            match worker.send(data) {
                Ok(task_id) => {
                    // debug!("Task {task_id} sent to worker {:?}", worker.name());
                    Some(task_id)
                }
                Err(err) => {
                    error!("Cannot send task to worker {worker_id}. {err}");
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
                Ok(res) => res.map(|(task_id, data)| match data {
                    Ok(Some(payload)) if payload.get_type() == PayloadType::Lua => {
                        error!("Cannot receive cached Lua payload from the worker");
                        TaskResult::new_error(
                            worker_id,
                            task_id,
                            "Cannot receive cached Lua payload from the worker",
                        )
                    }
                    Ok(Some(payload)) => TaskResult::new(worker_id, task_id, payload),
                    Ok(None) => TaskResult::new_empty(worker_id, task_id),
                    Err(msg) => {
                        error!("Worker {worker_id} task {task_id} failed: {msg}");
                        TaskResult::new_error(worker_id, task_id, &msg)
                    }
                }),
                Err(err) => {
                    error!("Cannot send task to worker {worker_id}. {err}");
                    None
                }
            }
        } else {
            error!("Unknown worker: {worker_id}");
            None
        }
    }

    /// Like `next_task_result`, but waits up to `timeout_ms` milliseconds for a result.
    pub fn wait_task_result(&mut self, worker_id: u16, timeout_ms: u32) -> Option<TaskResult> {
        if self.lua_workers.contains_key(&worker_id) {
            let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);
            loop {
                if let Some(res) = self.next_task_result(worker_id) {
                    return Some(res);
                }
                if Instant::now() >= deadline {
                    return None;
                }
                std::thread::sleep(Duration::from_micros(200));
            }
        } else {
            error!("Unknown worker: {worker_id}");
            None
        }
    }

    /// Send a message to the echo worker.
    pub fn send_echo(&mut self, data: &str) -> bool {
        if let Err(err) = self.echo_worker.send(data.into()) {
            error!("Cannot send message to the echo worker. {err}");
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
                error!("Cannot get echo message. {err}");
                None
            }
        }
    }
}
