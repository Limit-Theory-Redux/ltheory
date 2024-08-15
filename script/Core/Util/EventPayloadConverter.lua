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

        -- check if this is an array
        if value[1] ~= nil then
            if type(value[1]) == "boolean" then
                local array = ffi.new("bool[?]", #value, value)
                return EventPayload.FromBoolArray(array, #value)
            end
            if type(value[1]) == "integer" then
                local array = ffi.new("i64[?]", #value, value)
                return EventPayload.FromI64Array(array, #value) -- TODO: can we distinguish other integer types?
            end
            if type(value[1]) == "number" then
                local array = ffi.new("double[?]", #value, value)
                return EventPayload.FromF64Array(array, #value) -- TODO: can we distinguish other numeric types?
            end
            -- TODO: implement when luajit_gen_ffi supports string array/slice parameters
            -- if type(value[1]) == "string" then
            --     local array = ffi.new("cstr[?]", #value, value)
            --     return EventPayload.FromStringArray(array, #value)
            -- end
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
function EventPayloadConverter:tablePayloadToValue(payloadTable)
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
    if payloadType == EventPayloadType.Table then return self:tablePayloadToValue(payload:getTable()) end

    -- array types
    local result = {}
    local f = function(value)
        insert(result, value)
    end

    if payloadType == EventPayloadType.BoolArray then
        payload:forEachBool(f)
        return result
    end
    if payloadType == EventPayloadType.I8Array then
        payload:forEachI8(f)
        return result
    end
    if payloadType == EventPayloadType.U8Array then
        payload:forEachU8(f)
        return result
    end
    if payloadType == EventPayloadType.I16Array then
        payload:forEachI16(f)
        return result
    end
    if payloadType == EventPayloadType.U16Array then
        payload:forEachU16(f)
        return result
    end
    if payloadType == EventPayloadType.I32Array then
        payload:forEachI32(f)
        return result
    end
    if payloadType == EventPayloadType.U32Array then
        payload:forEachU32(f)
        return result
    end
    if payloadType == EventPayloadType.I64Array then
        payload:forEachI64(f)
        return result
    end
    if payloadType == EventPayloadType.U64Array then
        payload:forEachU64(f)
        return result
    end
    if payloadType == EventPayloadType.F32Array then
        payload:forEachF32(f)
        return result
    end
    if payloadType == EventPayloadType.F64Array then
        payload:forEachF64(f)
        return result
    end
    if payloadType == EventPayloadType.StringArray then
        payload:forEachString(function(value)
            insert(result, tostring(value))
        end)
        return result
    end

    Log.Error("Unexpected payload type: " .. tostring(payloadType))
end

return EventPayloadConverter
