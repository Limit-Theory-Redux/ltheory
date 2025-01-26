local libphx = require('libphx').lib
local Converter = require('Core.Util.Converter')
local PayloadConverter = require('Core.Util.PayloadConverter')

function onDef_EventBus_t(t, mt)
    -- TODO: should return a handler
    mt.__index.register = function(self, event, eventName, frameStage, rustPayload)
        local rustPayload = rustPayload == nil or rustPayload
        -- Log.Debug("Rust payload: " .. tostring(rustPayload))
        libphx.EventBus_Register(self, event, eventName, frameStage, rustPayload)
    end

    mt.__index.subscribe = function(self, event, ctxTable, callback)
        local entityId = ctxTable and ctxTable.getGuid and ctxTable:getGuid()
        local entityIdPtr = Converter.ToValuePtr(entityId, "uint64")

        local tunnelId = libphx.EventBus_Subscribe(self, event, entityIdPtr)
        EventTunnels[tunnelId] = function(...) callback(ctxTable, ...) end
        return tunnelId
    end

    mt.__index.unsubscribe = function(self, tunnelId)
        libphx.EventBus_Unsubscribe(self, tunnelId)
        EventTunnels[tunnelId] = nil
    end

    mt.__index.send = function(self, event, ctxTable, payload)
        local entityId = ctxTable and ctxTable.getGuid and ctxTable:getGuid()
        local entityIdPtr = Converter.ToValuePtr(entityId, "uint64")
        local rustPayload = self:hasRustPayload(event)
        libphx.EventBus_Send(self, event, entityIdPtr, PayloadConverter:valueToPayload(payload, rustPayload))
    end

    mt.__index.dispatch = function(self, event, payload)
        local rustPayload = self:hasRustPayload(event)
        libphx.EventBus_Send(self, event, nil, PayloadConverter:valueToPayload(payload, rustPayload))
    end

    mt.__index.nextEvent = function(self)
        local eventData = libphx.EventBus_NextEvent(self)
        if eventData == nil then
            return nil, nil
        end

        local payload = eventData:payload()
        local payloadValue = nil
        if payload ~= nil then
            payloadValue = PayloadConverter:payloadToValue(payload)
        end
        return eventData, payloadValue
    end
end
