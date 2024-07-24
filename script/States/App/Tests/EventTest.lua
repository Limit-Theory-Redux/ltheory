local EventTest = require('States.Application')

local iteration = 0

function EventTest:onInit()
    local fakeEntity = { getGuid = function() return 0 end }

    EventBus:subscribe(FrameStage.ToString(FrameStage.PreSim), fakeEntity, function() Log.Debug("onPreSim") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.Sim), fakeEntity, function() Log.Debug("onSim") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.PostSim), fakeEntity, function() Log.Debug("onPostSim") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.PreRender), fakeEntity, function() Log.Debug("onPreRender") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.Render), fakeEntity, function() Log.Debug("onRender") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.PostRender), fakeEntity, function() Log.Debug("onPostRender") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.PreInput), fakeEntity, function() Log.Debug("onPreInput") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.Input), fakeEntity, function() Log.Debug("onInput") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.PostInput), fakeEntity, function() Log.Debug("onPostInput") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.PreSim), fakeEntity, function() Log.Debug("onPreSim2") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.Sim), fakeEntity, function() Log.Debug("onSim2") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.PostSim), fakeEntity, function() Log.Debug("onPostSim2") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.PreRender), fakeEntity, function() Log.Debug("onPreRender2") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.Render), fakeEntity, function() Log.Debug("onRender2") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.PostRender), fakeEntity, function() Log.Debug("onPostRender2") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.PreInput), fakeEntity, function() Log.Debug("onPreInput2") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.Input), fakeEntity, function() Log.Debug("onInput2") end)
    EventBus:subscribe(FrameStage.ToString(FrameStage.PostInput), fakeEntity, function()
        Log.Debug("onPostInput2")
        Log.Debug("========")
        iteration = iteration + 1
        if iteration == 3 then
            os.exit()
        end
    end)

    for _, frameStage in pairs(FrameStage) do
        if type(frameStage) ~= "cdata" then -- prevent ToString
            for priorityName, priority in pairs(Enums.EventPriority) do
                if priority ~= Enums.EventPriority.Highest then
                    local frameStageName = tostring(FrameStage.ToString(frameStage))
                    frameStageName = frameStageName:gsub('"', '')
                    local eventName = "MyCustomEvent" .. frameStageName .. priorityName
                    EventBus:register(eventName, priority, frameStage, false)
                    EventBus:subscribe(eventName, fakeEntity, function(data) Log.Debug("Received " .. eventName) end)
                    EventBus:send(eventName, fakeEntity)
                end
            end
        end
    end
end

function EventTest:onPreRender() end

function EventTest:onRender() end

function EventTest:onPostRender() end

return EventTest
