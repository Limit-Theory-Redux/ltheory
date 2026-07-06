---
name: lua-workers
description: Running Lua code on background threads with TaskQueue, WorkerFunction, and Payload — the send-task/poll-result pattern. Use when offloading heavy game-side computation off the main thread.
---

# Lua Workers

A Lua Worker runs a Lua function in a fresh Lua state on a separate engine thread. The Rust side is `engine/lib/phx/src/engine/task_queue/` (see its README); docs: `doc/script/workers.md` and `doc/engine/README.md`.

Main objects (all FFI types; annotations in `engine/lib/phx/script/meta/TaskQueue.lua` and `Payload.lua`, extensions in `ffi_ext/TaskQueue.lua`):

- **TaskQueue** — global (set in `Main.lua` from `Engine:taskQueue()`); starts/stops workers, sends tasks, polls results.
- **WorkerFunction** — wraps the callback that runs on the worker thread.
- **Payload** — the data envelope for messages between main thread and workers (simple types — integer, string, etc. — or structured Payload).

## Pattern

1. Worker function in its own file (it runs in a separate Lua state — it cannot see main-state globals):

```lua
-- MyWorkerFunction.lua
return WorkerFunction.Create(function(payload)
    -- payload: simple type or Payload; return value of the same kind (optional)
    return result
end)
```

2. Start the worker (multiple instances allowed for load balancing). This registers `WorkerId.MyWorker`:

```lua
local workerId = TaskQueue:startWorker("MyWorker", "MyWorkerFunction.lua", 1)
```

3. Send tasks and poll for results each frame in the game loop:

```lua
local taskId = TaskQueue:sendTask(workerId, payload)
-- later, per frame:
local taskId, payload = TaskQueue:nextTaskResult(workerId)  -- nil taskId = nothing ready
```

## Examples

- `script/States/App/Tests/TestWorkerFunction.lua` — worker function definition.
- `script/States/App/Tests/WorkerTest.lua` — full start/send/poll lifecycle.
- `script/States/App/Workers/` and `script/States/App/Tests/Workers/` — real usages.

## Notes

- Results are polled, not pushed: check `nextTaskResult` every frame; correlate via `taskId`.
- The engine also has predefined Rust-side workers ("engine workers") managed by the same TaskQueue; `TaskResult` carries payload or error info.
- Keep per-task payloads small; workers pay serialization cost at the Payload boundary.
