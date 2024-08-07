local libphx = require('libphx').lib

function onDef_EventBus_t(t, mt)
    -- todo should return a handler
    mt.__index.subscribe = function(self, eventName, ctxTable, callback)
        local entityIdPtr = nil
        local entityId = ctxTable and ctxTable.getGuid and ctxTable:getGuid()

        if entityId then
            entityIdPtr = ffi.new("uint64[1]") -- convert to pointer since we use rust option
            entityIdPtr[0] = entityId
        end
        local tunnelId = libphx.EventBus_Subscribe(self, eventName, entityIdPtr)
        EventTunnels[tunnelId] = function(...) callback(ctxTable, ...) end
        return tunnelId
    end

    mt.__index.unsubscribe = function(self, tunnelId)
        libphx.EventBus_Unsubscribe(self, tunnelId)
        EventTunnels[tunnelId] = nil
    end

    mt.__index.send = function(self, eventName, ctxTable)
        local entityId = ctxTable and ctxTable.getGuid and ctxTable:getGuid()
        libphx.EventBus_Send(self, eventName, entityId, nil)
    end

    mt.__index.dispatch = function(self, eventName)
        libphx.EventBus_Send(self, eventName, nil, nil)
    end
end
