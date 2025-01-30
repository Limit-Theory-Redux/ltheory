# Task Queue and Workers Documentation

## Overview

The task queue and workers system provides a way to manage worker threads in an application. It allows starting either custom Lua scripts in separate threads or predefined engine workers. When started, workers can accept tasks and return their results.

This documentation covers the main components of the system:

- **TaskQueue**: A manager for worker threads.
- **Worker**: An abstraction for individual worker instances that process tasks.
- **TaskResult**: The result of a task execution, including payload or error information.

## Task Queue

The `TaskQueue` struct manages worker threads and handles their lifecycle. It provides methods to start, stop, and interact with workers.

### Methods

#### `new()`

```rust
pub fn new() -> Self;
```

- Creates a new task queue instance with an initial echo worker.

#### `start_worker()`

```rust
pub fn start_worker(
    &mut self,
    worker_id: u16,
    worker_name: &str,
    script_path: &str,
    instances_count: usize,
) -> bool;
```

- Starts a new Lua worker with the specified ID, name, script path, and number of instances.
- Returns `true` if successful, `false` otherwise.

#### `stop_worker()`

```rust
pub fn stop_worker(&mut self, worker_id: u16) -> bool;
```

- Stops a worker with the specified ID.
- Returns `true` if successful, `false` otherwise.

#### `send_task()`

```rust
pub fn send_task(&mut self, worker_id: u16, data: Payload) -> Option<usize>;
```

- Sends a task to the specified worker.
- Returns the task ID on success or `None` on failure.

#### `next_task_result()`

```rust
pub fn next_task_result(&mut self, worker_id: u16) -> Option<TaskResult>;
```

- Retrieves the next result of a finished task from the specified worker.

## Workers

A `Worker` represents an individual thread or process that can handle tasks. The system supports both native Rust workers and Lua-based workers.

### Native Workers

Native workers are implemented directly in Rust. They provide a simple way to execute synchronous tasks in separate threads.

#### Example Usage
```rust
let mut worker = Worker::new_native("MyWorker", 2, |data| {
    // Process data here
});
```

#### Echo Worker

The echo worker is a predefined native worker that echoes input data back. It's automatically created when the task queue is initialized.

### Lua Workers

Lua workers allow running Lua scripts as separate threads. They're started via the `start_worker` method of the task queue.

#### Example Usage

```rust
queue.start_worker(1, "LuaScriptWorker", "script.lua", 2);
```

This will start 2 instances of worker with id 1, each loads `script.lua` file.

## Worker IDs

Worker IDs are used to uniquely identify workers in the system.

Worker ids can be extended in Lua scripts to be used with Lua workers.

### Enum Variants

```rust
#[luajit_ffi_gen::luajit_ffi(repr = "u16")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerId {
    Echo,
    EngineWorkersCount,
}
```

- `Echo`: Represents the echo worker.
- `EngineWorkersCount`: Marks the end of predefined engine workers.

## Worker Instances

Each worker can have multiple instances to handle parallel tasks.

### Spawning Instances

```rust
let instance = WorkerInstance::new(
    id,
    worker_name,
    in_receiver,
    out_sender,
    function_arc,
);
```

- Creates a new worker instance with the specified ID, name, and processing function.

## Conclusion

The task queue and workers system provides a flexible way to manage both native Rust workers and Lua-based scripts. By using this system, you can efficiently handle tasks in parallel while maintaining clear communication channels between workers and the main application thread.
