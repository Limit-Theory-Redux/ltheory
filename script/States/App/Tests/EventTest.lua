local EventTest = require('States.Application')

local iteration = 0

function EventTest:onInit()
    local fakeEntity = { getGuid = function() return 0 end }

    EventBus:subscribe(EventType.PreSim, fakeEntity, function() Log.Debug("onPreSim") end)
    EventBus:subscribe(EventType.Sim, fakeEntity, function() Log.Debug("onSim") end)
    EventBus:subscribe(EventType.PostSim, fakeEntity, function() Log.Debug("onPostSim") end)
    EventBus:subscribe(EventType.PreRender, fakeEntity, function() Log.Debug("onPreRender") end)
    EventBus:subscribe(EventType.Render, fakeEntity, function() Log.Debug("onRender") end)
    EventBus:subscribe(EventType.PostRender, fakeEntity, function() Log.Debug("onPostRender") end)
    EventBus:subscribe(EventType.PreInput, fakeEntity, function() Log.Debug("onPreInput") end)
    EventBus:subscribe(EventType.Input, fakeEntity, function() Log.Debug("onInput") end)
    EventBus:subscribe(EventType.PostInput, fakeEntity, function() Log.Debug("onPostInput") end)
    EventBus:subscribe(EventType.PreSim, fakeEntity, function() Log.Debug("onPreSim2") end)
    EventBus:subscribe(EventType.Sim, fakeEntity, function() Log.Debug("onSim2") end)
    EventBus:subscribe(EventType.PostSim, fakeEntity, function() Log.Debug("onPostSim2") end)
    EventBus:subscribe(EventType.PreRender, fakeEntity, function() Log.Debug("onPreRender2") end)
    EventBus:subscribe(EventType.Render, fakeEntity, function() Log.Debug("onRender2") end)
    EventBus:subscribe(EventType.PostRender, fakeEntity, function() Log.Debug("onPostRender2") end)
    EventBus:subscribe(EventType.PreInput, fakeEntity, function() Log.Debug("onPreInput2") end)
    EventBus:subscribe(EventType.Input, fakeEntity, function() Log.Debug("onInput2") end)
    EventBus:subscribe(EventType.PostInput, fakeEntity, function()
        Log.Debug("onPostInput2")
        Log.Debug("========")
        iteration = iteration + 1
        if iteration == 3 then
            os.exit()
        end
    end)

    for eventType = 0, EventType.EngineEventTypesCount - 1 do
        EventBus:send(eventType, fakeEntity)
    end
end

function EventTest:onPreRender() end

function EventTest:onRender() end

function EventTest:onPostRender() end

return EventTest
