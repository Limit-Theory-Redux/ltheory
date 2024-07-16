local libphx = require('libphx').lib

function onDef_EventBus_t(t, mt)
    -- todo add LS definitions
    -- todo should also return a handler
    mt.__index.subscribe = function(self, eventName, ctxTable, callback)
        local entityId = ctxTable.getGuid and ctxTable.getGuid()
        local tunnelId = libphx.EventBus_Subscribe(self, eventName, entityId)
        EventTunnels[tunnelId] = function() callback(ctxTable) end
    end

    mt.__index.unsubscribe = function(self, tunnelId)
        local tunnelId = libphx.EventBus_Unubscribe(self, tunnelId)
        EventTunnels[tunnelId] = nil
    end
end
