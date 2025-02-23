# LTR Lua Workers

Lua Worker is based on the engine worker and allows to run Lua code in the new Lua state in a separate thread.

[TaskQueue](../../engine/lib/phx/script/meta/TaskQueue.lua)([ext](../../engine/lib/phx/script/ffi_ext/TaskQueue.lua)) object is used to create and manage workers. To send data between workers and main thread [Payload](../../engine/lib/phx/script/meta/Payload.lua) object is used.

Steps to create a new worker and run it:
1. Create a new worker function in a separate file, see [TestWorkerFunction](../../script/States/App/Tests/TestWorkerFunction.lua) for example.
   - Use `WorkerFunction.Create()` to create a worker function. As an input parameter it accepts a callback function with a single input parameter that can be either a simple type (integer, string, etc.) or **Payload**, and that returns an optional output of the same type (simple or **Payload**).
2. Create and run worker, see [WorkerTest](../../script/States/App/Tests/WorkerTest.lua) for example.
   1. Create and start worker with a new worker type:
        ```lua
        local workerId = TaskQueue:startWorker("MyWorker", "MyWorkerFunction.lua", 1)
        ```
        It is possible to run multiple instances of the worker for load balancing purposes.
        Creation of the worker adds new element to the `WorkerId` table: `WorkerId.MyWorker`.
   2. To send message to the worker use `TaskQueue:sendTask()`:
        ```lua
        local taskId = TaskQueue:sendTask(workerId, payload)
        ```
   3. To read message from the worker:
        ```lua
        local taskId, payload = TaskQueue:nextTaskResult(workerId)
        ```
        If `taskId == nil` then worker queue doesn't have any output messages.
    
    In the main game loop send messages to corresponding workers and then check each frame for response from them.
