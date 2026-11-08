-- Worker script of the sandbox: runs the system planner kernel on a `TaskQueue` worker thread.
-- A task is a table of numbers (see `Run`); the result carries the kernel's status, its waypoints
-- and the time the kernel took. Payload arrays arrive as Lua tables, so they are copied into the
-- `double[?]` buffers the kernel takes; the buffers are kept between tasks of this worker.
-- Design: doc/script/system-planner.md
package.path = package.path .. ';./engine/lib/phx/script/?.lua'
package.path = package.path .. ';./script/?.lua'

require('Init')

local WorkerFunction = require("Core.Util.WorkerFunction")
local SystemPlanner  = require("Shared.Helpers.SystemPlanner")

local BODY_STRIDE = SystemPlanner.BODY_STRIDE
local OUT_STRIDE  = SystemPlanner.OUT_STRIDE

local bodiesBuffer, bodiesCapacity = nil, 0
local outBuffer, outCapacity = nil, 0

---Copies `values` (1-based) into the shared bodies buffer, grown when it is too small.
---@param values number[]
---@param count integer
local function fillBodies(values, count)
    if count > bodiesCapacity then
        bodiesBuffer, bodiesCapacity = ffi.new("double[?]", count * BODY_STRIDE), count
    end
    for i = 1, count * BODY_STRIDE do bodiesBuffer[i - 1] = values[i] end
end

---@param capacity integer Waypoints
local function reserveOutput(capacity)
    if capacity > outCapacity then
        outBuffer, outCapacity = ffi.new("double[?]", capacity * OUT_STRIDE), capacity
    end
end

---Plans one path.
---@param request table `bodies`: count * BODY_STRIDE numbers, `count`, `start` and `goal`: 3 numbers each, `speed`, `depth`, `capacity`
---@return table `status`, `count`, `waypoints` (count * OUT_STRIDE numbers, empty for none), `kernelMs`
Run = WorkerFunction.Create(function(request)
    local count = request.count
    fillBodies(request.bodies or {}, count)
    reserveOutput(request.capacity)
    local start, goal = request.start, request.goal
    local begun = InstantTime.Now()
    local status, written = SystemPlanner.plan(bodiesBuffer, count, start[1], start[2], start[3],
        goal[1], goal[2], goal[3], request.speed, request.depth, outBuffer, request.capacity)
    local kernelMs = InstantTime.Now():durationSince(begun)
    local waypoints = {}
    for i = 1, written * OUT_STRIDE do waypoints[i] = outBuffer[i - 1] end
    return { status = status, count = written, waypoints = waypoints, kernelMs = kernelMs }
end)
