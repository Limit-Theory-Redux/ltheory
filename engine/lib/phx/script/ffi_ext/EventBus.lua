local libphx = require('libphx').lib

function ToValuePtr(value, type)
    local valuePtr = nil
    if value then
        valuePtr = ffi.new(type .. "[1]") -- convert to pointer since we use rust option
        valuePtr[0] = value
    end
    return valuePtr
end

function onDef_EventBus_t(t, mt)
    local EventBus = t

    -- TODO: should return a handler
    mt.__index.register = function(self, event, eventName, frameStage, rustPayload)
        local rustPayload = rustPayload == nil or rustPayload
        -- Log.Debug("Rust payload: " .. tostring(rustPayload))
        libphx.EventBus_Register(self, event, eventName, frameStage, rustPayload)
    end

    mt.__index.subscribe = function(self, event, ctxTable, callback)
        local entityId = ctxTable and ctxTable.getGuid and ctxTable:getGuid()
        local entityIdPtr = ToValuePtr(entityId, "uint64")

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
        local entityIdPtr = ToValuePtr(entityId, "uint64")
        local rustPayload = self:hasRustPayload(event)
        libphx.EventBus_Send(self, event, entityIdPtr, EventBus.ValueToPayload(payload, rustPayload))
    end

    mt.__index.dispatch = function(self, event, payload)
        local rustPayload = self:hasRustPayload(event)
        libphx.EventBus_Send(self, event, nil, EventBus.ValueToPayload(payload, rustPayload))
    end

    mt.__index.nextEvent = function(self)
        local eventData = libphx.EventBus_NextEvent(self)
        if eventData == nil then
            return nil, nil
        end

        local payload = eventData:payload()
        local payloadValue = nil
        if payload ~= nil then
            payloadValue = EventBus.PayloadToValue(payload)
        end
        return eventData, payloadValue
    end
end

function onDef_EventBus(t, mt)
    local EventBus = t

    t.PayloadCache = {}
    t.NextPayloadCacheIndex = 1 -- TODO: reusable indices/cache slots

    -- Convert Lua table into payload one
    ---@param value table
    ---@return EventPayloadTable
    t.ValueToPayloadTable = function(value)
        local result = EventPayloadTable.Create()
        for name, payload in pairs(value) do
            local payload = EventBus.ValueToPayload(payload, true)
            if payload ~= nil then
                result:add(name, payload)
            end
        end
        return result
    end

    -- Convert Lua value into payload
    ---@param value any
    ---@param rustPayload boolean
    ---@return EventPayload|nil
    t.ValueToPayload = function(value, rustPayload)
        if rustPayload then
            if type(value) == "nil" then
                return nil
            end
            if type(value) == "boolean" then
                return EventPayload.FromBool(value)
            end
            if type(value) == "integer" then
                return EventPayload.FromI64(value) -- TODO: can we distinguish other integer types?
            end
            if type(value) == "number" then
                return EventPayload.FromF64(value) -- TODO: can we distinguish other numeric types?
            end
            if type(value) == "string" then
                return EventPayload.FromString(value)
            end
            if type(value) == "string" then
                return EventPayload.FromString(value)
            end
            if type(value) == "table" then
                return EventPayload.FromTable(EventBus.ValueToPayloadTable(value))
            end

            Log.Error("Unsupported payload type: " .. tostring(type(value)) .. ". Value: " .. tostring(value))
        else
            -- process Lua only payload
            local payloadId = EventBus.NextPayloadCacheIndex
            EventBus.PayloadCache[payloadId] = value
            EventBus.NextPayloadCacheIndex = EventBus.NextPayloadCacheIndex + 1
            return EventPayload.FromLua(payloadId)
        end
    end

    -- Convert payload table into Lua one.
    ---@param payloadTable EventPayloadTable
    t.PayloadTableToValue = function(payloadTable)
        local result = {}
        local fieldsCount = tonumber(payloadTable:len())
        for index = 0, fieldsCount - 1 do
            local name = payloadTable:getName(index)
            local payload = payloadTable:getPayload(index)

            result[tostring(name)] = EventBus.PayloadToValue(payload)
        end
        return result
    end

    -- Convert payload into the lua value.
    ---@param payload EventPayload|nil
    t.PayloadToValue = function(payload)
        if payload == nil then
            return nil
        end

        local payloadType = payload:getType()
        if payloadType == EventPayloadType.Lua then
            local payloadId = payload:getLua()
            -- TODO: clean cache?
            return EventBus.PayloadCache[payloadId]
        end
        if payloadType == EventPayloadType.Bool then return payload:getBool() end
        if payloadType == EventPayloadType.I8 then return payload:getI8() end
        if payloadType == EventPayloadType.U8 then return payload:getU8() end
        if payloadType == EventPayloadType.I16 then return payload:getI16() end
        if payloadType == EventPayloadType.U16 then return payload:getU16() end
        if payloadType == EventPayloadType.I32 then return payload:getI32() end
        if payloadType == EventPayloadType.U32 then return payload:getU32() end
        if payloadType == EventPayloadType.I64 then return payload:getI64() end
        if payloadType == EventPayloadType.U64 then return payload:getU64() end
        if payloadType == EventPayloadType.F32 then return payload:getF32() end
        if payloadType == EventPayloadType.F64 then return payload:getF64() end
        if payloadType == EventPayloadType.String then return tostring(payload:getString()) end
        if payloadType == EventPayloadType.Table then return EventBus.PayloadTableToValue(payload:getTable()) end

        Log.Error("Unexpected payload type: " .. tostring(payloadType))
    end
end
