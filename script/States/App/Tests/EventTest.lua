local EventTest = require('States.Application')

local iteration = 0

function EventTest:onInit()
    local fakeEntity = { getGuid = function() return 0 end }

    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PreSim), fakeEntity, function() Log.Debug("onPreSim") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.Sim), fakeEntity,  function() Log.Debug("onSim") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PostSim), fakeEntity,  function() Log.Debug("onPostSim") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PreRender), fakeEntity,  function() Log.Debug("onPreRender") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.Render), fakeEntity,  function() Log.Debug("onRender") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PostRender), fakeEntity,  function() Log.Debug("onPostRender") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PreInput), fakeEntity,  function() Log.Debug("onPreInput") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.Input), fakeEntity,  function() Log.Debug("onInput") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PostInput), fakeEntity,  function() Log.Debug("onPostInput") end)

    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PreSim), fakeEntity, function() Log.Debug("onPreSim2") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.Sim), fakeEntity,  function() Log.Debug("onSim2") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PostSim), fakeEntity,  function() Log.Debug("onPostSim2") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PreRender), fakeEntity,  function() Log.Debug("onPreRender2") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.Render), fakeEntity,  function() Log.Debug("onRender2") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PostRender), fakeEntity,  function() Log.Debug("onPostRender2") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PreInput), fakeEntity,  function() Log.Debug("onPreInput2") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.Input), fakeEntity,  function() Log.Debug("onInput2") end)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PostInput), fakeEntity,  function()
        Log.Debug("onPostInput2")
        Log.Debug("========")
        iteration = iteration + 1
        if iteration == 3 then
            os.exit()
        end
    end)
end

function EventTest:onPreRender()
end

function EventTest:onRender()
end

function EventTest:onPostRender()
end

return EventTest
