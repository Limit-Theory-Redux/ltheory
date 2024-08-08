local EventPayloadTest = require('States.Application')

local iteration = 0

local function TableToString(tbl, indent)
    if type(tbl) ~= "table" then
        return tostring(tbl)
    end
    if not indent then indent = 0 end
    local toprint = "{\r\n"
    indent = indent + 2
    for k, v in pairs(tbl) do
        toprint = toprint .. string.rep(" ", indent)
        if (type(k) == "number") then
            toprint = toprint .. "[" .. k .. "] = "
        elseif (type(k) == "string") then
            toprint = toprint .. k .. " = "
        end
        if (type(v) == "number") then
            toprint = toprint .. v .. "/number,\r\n"
        elseif (type(v) == "string") then
            toprint = toprint .. v .. "/string,\r\n"
        elseif (type(v) == "table") then
            toprint = toprint .. TableToString(v, indent + 2) .. ",\r\n"
        elseif (type(v) == "boolean") then
            toprint = toprint .. tostring(v) .. "/boolean,\r\n"
        else
            toprint = toprint .. "\"" .. tostring(v) .. "\",\r\n"
        end
    end
    toprint = toprint .. string.rep(" ", indent - 2) .. "}"
    return toprint
end

function EventPayloadTest:onInit()
    local fakeEntity = { getGuid = function() return 0 end }

    EventType.AddEventTypes({ "TestEvent", "ExitEvent" })

    EventBus:register(EventType.TestEvent, "TestEvent", FrameStage.Render)
    EventBus:register(EventType.ExitEvent, "ExitEvent", FrameStage.PostInput)

    EventBus:subscribe(EventType.TestEvent, fakeEntity, function(self, eventData, payload)
        Log.Debug("TestEvent: " .. TableToString(payload) .. "/" .. tostring(type(payload)))
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
