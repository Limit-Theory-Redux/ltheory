local PayloadTest = require('States.Application')
local Converter = require('Core.Util.Converter')
local PayloadConverter = require('Core.Util.PayloadConverter')

function PayloadTest:onInit()
    local fakeEntity = { getGuid = function() return 0 end }

    Event.AddEvents({ "TestEvent", "TestEventLuaPayload", "ExitEvent" })

    EventBus:register(Event.TestEvent, "TestEvent", FrameStage.Render)
    EventBus:register(Event.TestEventLuaPayload, "TestEventLuaPayload", FrameStage.Render, false)
    EventBus:register(Event.ExitEvent, "ExitEvent", FrameStage.PostInput)

    EventBus:subscribe(Event.TestEvent, fakeEntity, function(self, eventData, payload)
        Log.Debug("TestEvent: " .. Converter.TableToString(payload) .. "/" .. tostring(type(payload)))
    end)
    EventBus:subscribe(Event.TestEventLuaPayload, fakeEntity, function(self, eventData, payload)
        Log.Debug("TestEventLuaPayload: " .. Converter.TableToString(payload) .. "/" .. tostring(type(payload)))
    end)
    EventBus:subscribe(Event.ExitEvent, fakeEntity, function()
        Log.Debug("ExitEvent")
        os.exit()
    end)

    EventBus:send(Event.TestEvent, fakeEntity, false)
    EventBus:send(Event.TestEvent, fakeEntity, 1)
    EventBus:send(Event.TestEvent, fakeEntity, 2.0)
    EventBus:send(Event.TestEvent, fakeEntity, "TestPayload1")
    EventBus:send(Event.TestEvent, fakeEntity, { false, true, false })
    EventBus:send(Event.TestEvent, fakeEntity, { 1, 2, 3 })
    EventBus:send(Event.TestEvent, fakeEntity, { 4.0, 5.0, 6.0, 7.0 })
    EventBus:send(Event.TestEvent, fakeEntity, { "TestPayload1", "TestPayload2" })
    EventBus:send(Event.TestEvent, fakeEntity, {
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
    EventBus:send(Event.TestEventLuaPayload, fakeEntity, {
        boolVal = true,
        intVal = 3,
        floatVal = 4.0,
        strVal = "TestPayload4",
        tableVal = {
            boolVal = true,
            intVal = 5,
            floatVal = 6.0,
            strVal = "TestPayload5",
        }
    })
    EventBus:send(Event.TestEvent, fakeEntity, PayloadConverter:valueToPayload("TestExplicitPayload", true))

    EventBus:send(Event.ExitEvent, fakeEntity)
end

function PayloadTest:onPreRender() end

function PayloadTest:onRender() end

function PayloadTest:onPostRender() end

return PayloadTest
