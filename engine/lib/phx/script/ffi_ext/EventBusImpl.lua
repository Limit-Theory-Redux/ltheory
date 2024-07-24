local libphx = require('libphx').lib

function onDef_EventBusImpl_t(t, mt)
    -- todo should return a handler
    mt.__index.subscribe = function(self, eventName, ctxTable, callback)
        local entityIdPtr = nil
        local entityId = ctxTable and ctxTable.getGuid and ctxTable:getGuid()

        if entityId then
            entityIdPtr = ffi.new("uint64[1]") -- convert to pointer since we use rust option
            entityIdPtr[0] = entityId
        end
        local tunnelId = libphx.EventBusImpl_Subscribe(self, eventName, entityIdPtr)
        EventTunnels[tunnelId] = function(...) callback(ctxTable, ...) end
        return tunnelId
    end

    mt.__index.unsubscribe = function(self, tunnelId)
        libphx.EventBusImpl_Unsubscribe(self, tunnelId)
        EventTunnels[tunnelId] = nil
    end

    mt.__index.send = function(self, eventName, ctxTable)
        local entityId = ctxTable and ctxTable.getGuid and ctxTable:getGuid()
        libphx.EventBusImpl_Send(self, eventName, entityId)
    end
end
