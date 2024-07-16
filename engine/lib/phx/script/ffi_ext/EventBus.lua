local libphx = require('libphx').lib

function onDef_EventBus_t(t, mt)
    -- todo add LS definitions
    mt.__index.subscribe = function(self, eventName, ctxTable, callback)
        local tunnelId = libphx.EventBus_Subscribe(self, eventName)
        EventTunnels[tunnelId] = function() callback(ctxTable) end
    end

    mt.__index.unsubscribe = function(self, tunnelId)
        local tunnelId = libphx.EventBus_Unubscribe(self, tunnelId)
        EventTunnels[tunnelId] = nil
    end
end
