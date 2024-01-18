CooperativeEventLoop = class(function(self) end)

function CooperativeEventLoop:init()
    self.coroutines = {}
    self.groups = {}
end

-- For individual tasks:
function CooperativeEventLoop:addTask(func)
    local co = coroutine.create(func)
    local taskId = #self.coroutines + 1  -- Simple unique identifier
    self.coroutines[taskId] = { coroutine = co, completed = false, result = nil, progress = 0 }
    Log.Debug("Added new coroutine to async event loop with ID: " .. taskId)
    return taskId  -- Return the unique identifier
end

function CooperativeEventLoop:addTaskAsync(func, ctx, callback)
    local taskId = #self.coroutines + 1  -- Simple unique identifier
    local wrappedFunc = function()
        func(taskId)  -- Pass taskId to the original function
    end
    local co = coroutine.create(wrappedFunc)
    self.coroutines[taskId] = { coroutine = co, completed = false, result = nil, progress = 0, ctx = ctx, callback = callback }
    Log.Debug("Added new coroutine to async event loop with ID: " .. taskId)
    return taskId  -- Return the unique identifier
end

function Cooperative(taskFunc)
    return CooperativeEventLoop:addTask(taskFunc)
end

function CooperativeAsync(taskFunc, ctx, callback)
    return CooperativeEventLoop:addTaskAsync(taskFunc, ctx, callback)
end

function CooperativeProgress(taskId)
    local task = CooperativeEventLoop.coroutines[taskId]
    if task then
        return task.progress * 100
    else
        error("Task with ID " .. taskId .. " does not exist.")
    end
end

function CooperativeCompleted(taskId)
    local task = CooperativeEventLoop.coroutines[taskId]
    if task then
        return task.completed
    else
        error("Task with ID " .. taskId .. " does not exist.")
    end
end

function CooperativeResult(taskId)
    local task = CooperativeEventLoop.coroutines[taskId]
    if task then
        return task.result
    else
        error("Task with ID " .. taskId .. " does not exist.")
    end
end

-- For group tasks:
function CooperativeEventLoop:addGroupTask(groupName, taskFunc)
    local co = coroutine.create(taskFunc)
    if not self.groups[groupName] then
        self.groups[groupName] = { tasks = {}, completed = false }
    end
    table.insert(self.groups[groupName].tasks, { coroutine = co, completed = false, progress = 0 })
    Log.Debug("Added new coroutine to group: " .. groupName)
end

function CooperativeGroup(groupId, taskFunc)
    CooperativeEventLoop:addGroupTask(groupId, taskFunc)
end

function CooperativeGroupProgress(groupId)
    local group = CooperativeEventLoop.groups[groupId]
    if not group then
        error("Group '" .. groupId .. "' does not exist.")
    end
    local totalProgress = 0
    for _, task in ipairs(group.tasks) do
        totalProgress = ( totalProgress + task.progress ) * 100
    end
    return totalProgress / #group.tasks
end

function CooperativeGroupCompleted(groupId)
    local group = CooperativeEventLoop.groups[groupId]
    if not group then
        return false
    end
    for _, task in ipairs(group.tasks) do
        if not task.completed then
            return false
        end
    end
    return true
end

function CooperativeEventLoop:update(dt)
    -- Update individual tasks
    for taskId, task in pairs(self.coroutines) do
        local co = task.coroutine
        if co and not task.completed then
            local status, resultOrErr = coroutine.resume(co)
            if status then
                if coroutine.status(co) == "dead" then
                    task.completed = true
                    task.result = resultOrErr

                    if task.ctx and task.callback then
                        task.callback(task.ctx, task.result)
                    end
                    Log.Debug("Task completed with ID: " .. taskId)
                end
            else
                Log.Error("Coroutine error: " .. resultOrErr)
            end
        end
    end

    -- Update group tasks
    for groupName, group in pairs(self.groups) do
        for i = #group.tasks, 1, -1 do
            local task = group.tasks[i]
            if task.coroutine and coroutine.status(task.coroutine) == "dead" then
                task.completed = true
                table.remove(group.tasks, i)
            elseif task.coroutine then
                local status, err = coroutine.resume(task.coroutine)
                if not status then
                    Log.Error("Coroutine error in group " .. groupName .. ": " .. err)
                end
            end
        end
        group.completed = CooperativeGroupCompleted(groupName)
    end
end

function CooperativeEventLoop:updateTaskProgress(taskId, progress)
    local task = self.coroutines[taskId]
    if task then
        task.progress = progress
    else
        error("Task with ID " .. taskId .. " does not exist.")
    end
end

function CooperativeUpdateProgress(taskId, progress)
    CooperativeEventLoop:updateTaskProgress(taskId, progress)
end

function CooperativeEventLoop:updateGroupTaskProgress(groupId, coroutineId, progress)
    local group = self.groups[groupId]
    if group then
        for _, task in ipairs(group.tasks) do
            if task.coroutine == coroutineId then
                task.progress = progress
                break
            end
        end
    else
        error("Group '" .. groupId .. "' does not exist.")
    end
end

function CooperativeGroupUpdateProgress(groupId, coroutineId, progress)
    CooperativeEventLoop:updateGroupTaskProgress(groupId, coroutineId, progress)
end

return CooperativeEventLoop:init()
