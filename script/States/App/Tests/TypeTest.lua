local TypeTest = require('States.Application')

local EntityInfo = require('_ECS_WIP_TEMP.Shared.Types.EntityInfo')
local ComponentInfo = require('_ECS_WIP_TEMP.Shared.Types.ComponentInfo')

local QuickProfiler = require('_ECS_WIP_TEMP.Shared.Tools.QuickProfiler')
local profiler = QuickProfiler("Type Performance", true, false, true)
local profiler2 = QuickProfiler("Without Type Performance", true, false, true)

local someId = 0
local someArchetype = 0
local function testInfo(id, archetype)
    someId = id
    someArchetype = archetype
end

---@diagnostic disable-next-line: duplicate-set-field
function TypeTest:onInit()
    -- Make sure vanilla type still works
    Log.Debug("Vanilla types: %s|%s|%s|%s|%s", type(), type(0), type(""), type(true), type(function() end))

    -- FFI type checking example
    local timestamp = TimeStamp.Now()
    Log.Debug("FFI type: %s | isValid: %s", timestamp, ffi.istype("TimeStamp", timestamp))
    if not ffi.istype("TimeStamp", timestamp) then Log.Error("FFI Type not found") end

    -- Custom type checking example
    local entityInfo = EntityInfo { id = 0, archetype = 0 }
    Log.Debug("Custom type: %s | typeInt: %s", entityInfo, type(entityInfo))
    if type(entityInfo) ~= Enums.Type.EntityInfo then Log.Error("Custom Type not found") end

    local testRuns = 50000
    collectgarbage("stop")    -- Stop garbage collection
    collectgarbage("collect") -- Do one collection run

    -- Warm-up Custom Types
    --* Probably does not really do anything but well
    for i = 1, 1000 do
        local entityInfo = EntityInfo { id = 0, archetype = 0 }
        local componentInfo = ComponentInfo { id = 0, archetype = 0 }
    end

    -- Warm-up Table-Based Types
    --* Probably does not really do anything but well
    for i = 1, 1000 do
        local entityInfo = { id = 0, archetype = 0 }
        local componentInfo = { id = 0, archetype = 0 }
    end

    -- Custom Types
    local customTypesTotalTimeWithJIT = 0
    local customTypesTotalTimeWithoutJIT = 0
    local noTypesTotalTimeWithJIT = 0
    local noTypesTotalTimeWithoutJIT = 0

    -- Measure performance with JIT enabled
    jit.on()

    -- Custom Types with JIT
    for i = 1, testRuns do
        profiler:start()
        local entityInfo = EntityInfo { id = 0, archetype = 0 }
        local componentInfo = ComponentInfo { id = 0, archetype = 0 }
        testInfo(entityInfo, componentInfo)
        local time = profiler:stop()
        if time then
            customTypesTotalTimeWithJIT = customTypesTotalTimeWithJIT + time
        end
    end
    local customTypesAverageTimeWithJIT = customTypesTotalTimeWithJIT / testRuns

    -- No Types with JIT
    for i = 1, testRuns do
        profiler2:start()
        local entityInfo = { id = 0, archetype = 0 }
        local componentInfo = { id = 0, archetype = 0 }
        testInfo(entityInfo, componentInfo)
        local time = profiler2:stop()
        if time then
            noTypesTotalTimeWithJIT = noTypesTotalTimeWithJIT + time
        end
    end
    local noTypesAverageTimeWithJIT = noTypesTotalTimeWithJIT / testRuns

    -- Do one collection run
    collectgarbage("collect")

    -- reset testInfo
    testInfo(0, 0)

    -- Measure performance with JIT disabled
    jit.off()

    -- Custom Types without JIT
    for i = 1, testRuns do
        profiler:start()
        local entityInfo = EntityInfo { id = 0, archetype = 0 }
        local componentInfo = ComponentInfo { id = 0, archetype = 0 }
        testInfo(entityInfo, componentInfo)
        local time = profiler:stop()
        if time then
            customTypesTotalTimeWithoutJIT = customTypesTotalTimeWithoutJIT + time
        end
    end
    local customTypesAverageTimeWithoutJIT = customTypesTotalTimeWithoutJIT / testRuns

    -- No Types without JIT
    for i = 1, testRuns do
        profiler2:start()
        local entityInfo = { id = 0, archetype = 0 }
        local componentInfo = { id = 0, archetype = 0 }
        testInfo(entityInfo, componentInfo)
        local time = profiler2:stop()
        if time then
            noTypesTotalTimeWithoutJIT = noTypesTotalTimeWithoutJIT + time
        end
    end
    local noTypesAverageTimeWithoutJIT = noTypesTotalTimeWithoutJIT / testRuns

    collectgarbage("restart") -- Restart garbage collection

    -- Calculate performance hits
    local customTypesHitWithJIT = (customTypesAverageTimeWithJIT - noTypesAverageTimeWithJIT) / noTypesAverageTimeWithJIT * 100
    local customTypesHitWithoutJIT = (customTypesAverageTimeWithoutJIT - noTypesAverageTimeWithoutJIT) / noTypesAverageTimeWithoutJIT * 100

    -- Log the results with the average times and their differences
    Log.Debug("With JIT: Custom Types: %.10f ms | No Types: %.10f ms | Custom Types Performance Hit: %.4f %%",
        customTypesAverageTimeWithJIT, noTypesAverageTimeWithJIT, customTypesHitWithJIT)
    Log.Debug("Without JIT: Custom Types: %.10f ms | No Types: %.10f ms | Custom Types Performance Hit: %.4f %%",
        customTypesAverageTimeWithoutJIT, noTypesAverageTimeWithoutJIT, customTypesHitWithoutJIT)

    self:quit()
end

return TypeTest