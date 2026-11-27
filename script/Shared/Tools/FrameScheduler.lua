-- Work spread over frames, within one budget: a system that has more to do than a frame should
-- spend hands it to the scheduler as a coroutine (`spawn`), which calls `checkpoint` now and then;
-- each frame `run` resumes the tasks in turn, and a task's `checkpoint` yields once the frame's budget
-- is spent, so the work goes on at the next frame where it stopped. One budget for all such work,
-- tuned in one place (`budgetMs`), instead of every system spreading its own.
--
-- It is for work on the main thread (the ECS, physics, the engine). Work that needs none of them
-- goes to a worker thread instead (`TaskQueue`), where it costs the frame nothing.
--
--   local task = FrameScheduler:spawn("JobBoard rebuild", function()
--       for _, station in ipairs(stations) do
--           rebuild(station)
--           FrameScheduler:checkpoint()
--       end
--       return result
--   end)
--   ... later: if task:isDone() then use(task:getResult()) end
local SubscriberProfile = require("Shared.Tools.SubscriberProfile")

---A piece of work the scheduler runs: a coroutine of a function, its result once it has returned.
---@class FrameTask
---@overload fun(self: FrameTask, name: string, fn: function, args: table): FrameTask class internal
---@overload fun(name: string, fn: function, args: table): FrameTask class external
local FrameTask = Class("FrameTask", function(self, name, fn, args)
    self.name = name
    self.done = false
    self.result = nil
    self.failed = nil
    self.co = coroutine.create(function() return fn(unpack(args, 1, args.n)) end)
end)

---@return boolean
function FrameTask:isDone()
    return self.done
end

---@return any result What the task's function returned
function FrameTask:getResult()
    return self.result
end

---@return string|nil err The error, when it failed
function FrameTask:getError()
    return self.failed
end

---@class FrameScheduler
---@overload fun(self: FrameScheduler): FrameScheduler class internal
---@overload fun(): FrameScheduler class external
local FrameScheduler = Class("FrameScheduler", function(self)
    -- Milliseconds of the main thread's frame the tasks may use together.
    self.budgetMs = 2
    ---@type FrameTask[]
    self.tasks = {}
    self.deadline = 0
    self.cursor = 1
end)

---Starts a task: `fn(...)` runs in a coroutine from the next `run` on.
---@param name string For the profile and errors
---@param fn function
---@return FrameTask
function FrameScheduler:spawn(name, fn, ...)
    local task = FrameTask(name, fn, { n = select("#", ...), ... })
    self.tasks[#self.tasks + 1] = task
    return task
end

---Yields when the frame's budget is spent; called by a task between pieces of its work. Outside a
---task, or with budget left, it does nothing.
function FrameScheduler:checkpoint()
    if coroutine.running() and os.clock() >= self.deadline then coroutine.yield() end
end

---Resumes the tasks in turn until the budget is spent or every task has had a turn; called once a
---frame. Tasks that end are dropped; their result (or error) is kept on the task.
function FrameScheduler:run()
    local tasks = self.tasks
    if #tasks == 0 then return end
    self.deadline = os.clock() + self.budgetMs / 1000
    for _ = 1, #tasks do
        if os.clock() >= self.deadline then break end
        if self.cursor > #tasks then self.cursor = 1 end
        local task = tasks[self.cursor]
        local t = os.clock()
        local ok, value = coroutine.resume(task.co)
        SubscriberProfile:mark("scheduler: " .. task.name, t)
        if not ok then
            task.done, task.failed = true, tostring(value)
            Log.Error("FrameScheduler: %s failed: %s\n%s", task.name, tostring(value), debug.traceback(task.co))
        elseif coroutine.status(task.co) == "dead" then
            task.done, task.result = true, value
        end
        if task.done then table.remove(tasks, self.cursor) else self.cursor = self.cursor + 1 end
    end
end

---@return integer count The tasks waiting or running
function FrameScheduler:count()
    return #self.tasks
end

return FrameScheduler()
