-- Worker script of the system planner: runs the kernel on a `TaskQueue` worker thread.
-- A task is a table of numbers (see `Run`); the result carries the kernel's status, its waypoints
-- and the time the kernel took. Payload arrays arrive as Lua tables, so they are copied into the
-- `double[?]` buffers the kernel takes; the buffers are kept between tasks of this worker.
-- Design: doc/script/nav/system-planner.md
package.path = package.path .. ';./engine/lib/phx/script/?.lua'
package.path = package.path .. ';./script/?.lua'

require('Init')

local WorkerFunction = require("Core.Util.WorkerFunction")
local SystemPlanner  = require("Shared.Helpers.SystemPlanner")

local BODY_STRIDE  = SystemPlanner.BODY_STRIDE
local OUT_STRIDE   = SystemPlanner.OUT_STRIDE
local DRIVE_STRIDE = SystemPlanner.DRIVE_STRIDE

local ZONE_STRIDE  = SystemPlanner.ZONE_STRIDE

local bodiesBuffer, bodiesCapacity = nil, 0
local zonesBuffer, zonesCapacity = nil, 0
local outBuffer, outCapacity = nil, 0
local driveBuffer = nil

---Copies `values` (1-based) into the shared bodies buffer, grown when it is too small.
---@param values number[]
---@param count integer
local function fillBodies(values, count)
    if count > bodiesCapacity then
        bodiesBuffer, bodiesCapacity = ffi.new("double[?]", count * BODY_STRIDE), count
    end
    for i = 1, count * BODY_STRIDE do bodiesBuffer[i - 1] = values[i] end
end

---Copies `values` (1-based) into the shared zones buffer, grown when it is too small.
---@param values number[]|nil
---@param count integer|nil Zones, nil for none
---@return userdata|nil buffer Nil when there are no zones
local function fillZones(values, count)
    if not values or not count or count == 0 then return nil end
    if count > zonesCapacity then
        zonesBuffer, zonesCapacity = ffi.new("double[?]", count * ZONE_STRIDE), count
    end
    for i = 1, count * ZONE_STRIDE do zonesBuffer[i - 1] = values[i] end
    return zonesBuffer
end

---@param capacity integer Waypoints
local function reserveOutput(capacity)
    if capacity > outCapacity then
        outBuffer, outCapacity = ffi.new("double[?]", capacity * OUT_STRIDE), capacity
    end
end

---@param values number[]|nil `DRIVE_STRIDE` numbers, nil for no drive
---@return userdata|nil
local function fillDrive(values)
    if not values then return nil end
    driveBuffer = driveBuffer or ffi.new("double[?]", DRIVE_STRIDE)
    for i = 1, DRIVE_STRIDE do driveBuffer[i - 1] = values[i] end
    return driveBuffer
end

---Plans one path.
---@param request table `bodies`: count * BODY_STRIDE numbers, `count`, `start` and `goal`: 3 numbers each, `goalFrame`, `speed`, `accel`, `drag`, `depth`, `capacity`, `drive`: DRIVE_STRIDE numbers or nil, `arrivalRadius` and `standOff`: a number or nil, and `zones`: zoneCount * ZONE_STRIDE numbers or nil, with `zoneCount`
---@return table `status`, `count`, `waypoints` (count * OUT_STRIDE numbers, empty for none), `kernelMs`
Run = WorkerFunction.Create(function(request)
    local count = request.count
    fillBodies(request.bodies or {}, count)
    reserveOutput(request.capacity)
    local start, goal = request.start, request.goal
    local begun = InstantTime.Now()
    local status, written = SystemPlanner.plan(bodiesBuffer, count, start[1], start[2], start[3],
        request.goalFrame, goal[1], goal[2], goal[3], request.speed, request.accel, request.drag,
        request.depth, outBuffer, request.capacity, fillDrive(request.drive), request.arrivalRadius,
        fillZones(request.zones, request.zoneCount), request.zoneCount, request.standOff)
    local kernelMs = InstantTime.Now():durationSince(begun)
    local waypoints = {}
    for i = 1, written * OUT_STRIDE do waypoints[i] = outBuffer[i - 1] end
    return { status = status, count = written, waypoints = waypoints, kernelMs = kernelMs }
end)
