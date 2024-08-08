local libphx = require('libphx').lib

local ToValuePtr, ValueToPayload, PayloadToValue

function onDef_EventBus_t(t, mt)
    -- TODO: should return a handler
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
        libphx.EventBus_Send(self, event, entityIdPtr, ValueToPayload(payload))
    end

    mt.__index.dispatch = function(self, event, payload)
        libphx.EventBus_Send(self, event, nil, ValueToPayload(payload))
    end

    mt.__index.nextEvent = function(self)
        local eventData = libphx.EventBus_NextEvent(self)
        if eventData == nil then
            return nil, nil
        end

        local payload = eventData:payload()
        local payloadValue = nil
        if payload ~= nil then
            payloadValue = PayloadToValue(payload)
        end
        return eventData, payloadValue
    end
end

function ToValuePtr(value, type)
    local valuePtr = nil
    if value then
        valuePtr = ffi.new(type .. "[1]") -- convert to pointer since we use rust option
        valuePtr[0] = value
    end
    return valuePtr
end

-- Convert Lua table into payload one
---@param value table
---@return EventPayloadTable
function ValueToPayloadTable(value)
    local result = EventPayloadTable.Create()
    for name, payload in pairs(value) do
        local payload = ValueToPayload(payload)
        if payload ~= nil then
            result:add(name, payload)
        end
    end
    return result
end

-- Convert Lua value into payload
---@param value any
---@return EventPayload|nil
function ValueToPayload(value)
    -- TODO: process Lua only payload
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
        return EventPayload.FromTable(ValueToPayloadTable(value))
    end

    Log.Error("Unsupported payload type: " .. tostring(type(value)) .. ". Value: " .. tostring(value))
end

-- Convert payload table into Lua one.
---@param payloadTable EventPayloadTable
function PayloadTableToValue(payloadTable)
    local result = {}
    local fieldsCount = tonumber(payloadTable:len())
    for index = 0, fieldsCount - 1 do
        local name = payloadTable:getName(index)
        local payload = payloadTable:getPayload(index)

        result[tostring(name)] = PayloadToValue(payload)
    end
    return result
end

-- Convert payload into the lua value.
---@param payload EventPayload
function PayloadToValue(payload)
    if payload == nil then
        return nil
    end

    local payloadType = payload:getType()
    if payloadType == EventPayloadType.Lua then
        -- TODO: implement
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
    if payloadType == EventPayloadType.Table then return PayloadTableToValue(payload:getTable()) end

    Log.Error("Unexpected payload type: " .. tostring(payloadType))
end
