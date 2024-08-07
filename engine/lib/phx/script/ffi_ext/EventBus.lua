local libphx = require('libphx').lib

local function ToValuePtr(value, type)
    local valuePtr = nil
    if value then
        valuePtr = ffi.new(type .. "[1]") -- convert to pointer since we use rust option
        valuePtr[0] = value
    end
    return valuePtr
end

function onDef_EventBus_t(t, mt)
    -- TODO: should return a handler
    mt.__index.subscribe = function(self, eventType, ctxTable, callback)
        local entityId = ctxTable and ctxTable.getGuid and ctxTable:getGuid()
        local entityIdPtr = ToValuePtr(entityId, "uint64")

        local tunnelId = libphx.EventBus_Subscribe(self, eventType, entityIdPtr)
        EventTunnels[tunnelId] = function(...) callback(ctxTable, ...) end
        return tunnelId
    end

    mt.__index.unsubscribe = function(self, tunnelId)
        libphx.EventBus_Unsubscribe(self, tunnelId)
        EventTunnels[tunnelId] = nil
    end

    mt.__index.send = function(self, eventType, ctxTable, payload)
        local entityId = ctxTable and ctxTable.getGuid and ctxTable:getGuid()
        local entityIdPtr = ToValuePtr(entityId, "uint64")
        libphx.EventBus_Send(self, eventType, entityIdPtr, payload)
    end

    mt.__index.dispatch = function(self, eventType, payload)
        libphx.EventBus_Send(self, eventType, nil, payload)
    end

    mt.__index.nextEvent = function(self)
        local eventData = libphx.EventBus_NextEvent(self)
        -- TODO: convert payload into actual value/table
        return eventData
    end
end
