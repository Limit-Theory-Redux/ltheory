local EventPayloadTest = require('States.Application')

local iteration = 0

function EventPayloadTest:onInit()
    local fakeEntity = { getGuid = function() return 0 end }

    EventType.AddEventTypes({ "TestEvent", "ExitEvent" })

    EventBus:register(EventType.TestEvent, "TestEvent", FrameStage.Render)
    EventBus:register(EventType.ExitEvent, "ExitEvent", FrameStage.PostInput)

    EventBus:subscribe(EventType.TestEvent, fakeEntity, function(self, eventData, payload)
        Log.Debug("TestEvent: " .. tostring(payload) .. "/" .. tostring(type(payload)))
    end)
    EventBus:subscribe(EventType.ExitEvent, fakeEntity, function()
        Log.Debug("ExitEvent")
        os.exit()
    end)

    EventBus:send(EventType.TestEvent, fakeEntity, false)
    EventBus:send(EventType.TestEvent, fakeEntity, 1)
    EventBus:send(EventType.TestEvent, fakeEntity, 2.0)
    EventBus:send(EventType.TestEvent, fakeEntity, "TestPayload1")
    EventBus:send(EventType.TestEvent, fakeEntity, {
        boolVal = true,
        intVal = 3,
        floatVal = 4.0,
        strVal = "TestPayload2",
        tableVal = {
            boolVal = true,
            intVal = 5,
            floatVal = 6.0,
            strVal = "TestPayload3",
        }
    })

    EventBus:send(EventType.ExitEvent, fakeEntity)
end

function EventPayloadTest:onPreRender() end

function EventPayloadTest:onRender() end

function EventPayloadTest:onPostRender() end

return EventPayloadTest
