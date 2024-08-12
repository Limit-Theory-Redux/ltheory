local Converter = require('Core.Util.Converter')

---@class EventPayloadConverter
local EventPayloadConverter = {
    Cache = {},
    CacheIndex = 1, -- TODO: reusable indices/cache slots
}
EventPayloadConverter.__index = EventPayloadConverter

-- Convert Lua table into payload one
---@param value table
---@return EventPayloadTable
function EventPayloadConverter:valueToPayloadTable(value)
    local result = EventPayloadTable.Create()
    for name, payload in pairs(value) do
        local payload = self:valueToPayload(payload, true)
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
function EventPayloadConverter:valueToPayload(value, rustPayload)
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
            return EventPayload.FromTable(self:valueToPayloadTable(value))
        end

        Log.Error("Unsupported payload type: " .. tostring(type(value)) .. ". Value: " .. tostring(value))
    else
        -- process Lua only payload
        local payloadId = EventPayloadConverter.CacheIndex
        EventPayloadConverter.Cache[payloadId] = value
        EventPayloadConverter.CacheIndex = EventPayloadConverter.CacheIndex + 1
        return EventPayload.FromLua(payloadId)
    end
end

-- Convert payload table into Lua one.
---@param payloadTable EventPayloadTable
function EventPayloadConverter:payloadTableToValue(payloadTable)
    local result = {}
    local fieldsCount = tonumber(payloadTable:len())
    for index = 0, fieldsCount - 1 do
        local name = payloadTable:getName(index)
        local payload = payloadTable:getPayload(index)

        result[tostring(name)] = self:payloadToValue(payload)
    end
    return result
end

-- Convert payload into the lua value.
---@param payload EventPayload|nil
function EventPayloadConverter:payloadToValue(payload)
    if payload == nil then
        return nil
    end

    local payloadType = payload:getType()
    if payloadType == EventPayloadType.Lua then
        local payloadId = tonumber(payload:getLua())
        -- TODO: clean cache?
        return EventPayloadConverter.Cache[payloadId]
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
    if payloadType == EventPayloadType.Table then return self:payloadTableToValue(payload:getTable()) end

    Log.Error("Unexpected payload type: " .. tostring(payloadType))
end

return EventPayloadConverter
