local EventTest = require('States.Application')

local iteration = 0

function EventTest:onInit()
    local fakeEntity = { getGuid = function() return 0 end }

    EventBus:subscribe(Event.PreSim, fakeEntity, function() Log.Debug("onPreSim") end)
    EventBus:subscribe(Event.Sim, fakeEntity, function() Log.Debug("onSim") end)
    EventBus:subscribe(Event.PostSim, fakeEntity, function() Log.Debug("onPostSim") end)
    EventBus:subscribe(Event.PreRender, fakeEntity, function() Log.Debug("onPreRender") end)
    EventBus:subscribe(Event.Render, fakeEntity, function() Log.Debug("onRender") end)
    EventBus:subscribe(Event.PostRender, fakeEntity, function() Log.Debug("onPostRender") end)
    EventBus:subscribe(Event.PreInput, fakeEntity, function() Log.Debug("onPreInput") end)
    EventBus:subscribe(Event.Input, fakeEntity, function() Log.Debug("onInput") end)
    EventBus:subscribe(Event.PostInput, fakeEntity, function() Log.Debug("onPostInput") end)
    EventBus:subscribe(Event.PreSim, fakeEntity, function() Log.Debug("onPreSim2") end)
    EventBus:subscribe(Event.Sim, fakeEntity, function() Log.Debug("onSim2") end)
    EventBus:subscribe(Event.PostSim, fakeEntity, function() Log.Debug("onPostSim2") end)
    EventBus:subscribe(Event.PreRender, fakeEntity, function() Log.Debug("onPreRender2") end)
    EventBus:subscribe(Event.Render, fakeEntity, function() Log.Debug("onRender2") end)
    EventBus:subscribe(Event.PostRender, fakeEntity, function() Log.Debug("onPostRender2") end)
    EventBus:subscribe(Event.PreInput, fakeEntity, function() Log.Debug("onPreInput2") end)
    EventBus:subscribe(Event.Input, fakeEntity, function() Log.Debug("onInput2") end)
    EventBus:subscribe(Event.PostInput, fakeEntity, function()
        Log.Debug("onPostInput2")
        Log.Debug("========")
        iteration = iteration + 1
        if iteration == 3 then
            os.exit()
        end
    end)

    Event.AddEvents({ "TestEvent" })

    EventBus:register(Event.TestEvent, "TestEvent", FrameStage.Render)
    EventBus:subscribe(Event.TestEvent, fakeEntity, function() Log.Debug("TestEvent") end)

    for event = 0, Event.EngineEventsCount - 1 do
        EventBus:send(event, fakeEntity)
    end

    EventBus:send(Event.TestEvent, fakeEntity)
end

function EventTest:onPreRender() end

function EventTest:onRender() end

function EventTest:onPostRender() end

return EventTest
