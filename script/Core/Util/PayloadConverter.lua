---@class PayloadConverter
local PayloadConverter = {
    Cache = {},
    CacheIndex = 1, -- TODO: reusable indices/cache slots
}
PayloadConverter.__index = PayloadConverter

-- Convert Lua table into payload one
---@param value table
---@return PayloadTable
function PayloadConverter:valueToPayloadTable(value)
    local result = PayloadTable.Create()
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
---@return Payload?
function PayloadConverter:valueToPayload(value, rustPayload)
    if rustPayload then
        if type(value) == "nil" then
            return nil
        end
        if type(value) == "boolean" then
            return Payload.FromBool(value)
        end
        if type(value) == "integer" then
            return Payload.FromI64(value) -- TODO: can we distinguish other integer types?
        end
        if type(value) == "number" then
            return Payload.FromF64(value) -- TODO: can we distinguish other numeric types?
        end
        if type(value) == "string" then
            return Payload.FromString(value)
        end

        -- check if this is an array
        if value[1] ~= nil then
            if type(value[1]) == "boolean" then
                local array = ffi.new("bool[?]", #value, value)
                return Payload.FromBoolArray(array, #value)
            end
            if type(value[1]) == "integer" then
                local array = ffi.new("i64[?]", #value, value)
                return Payload.FromI64Array(array, #value) -- TODO: can we distinguish other integer types?
            end
            if type(value[1]) == "number" then
                local array = ffi.new("double[?]", #value, value)
                return Payload.FromF64Array(array, #value) -- TODO: can we distinguish other numeric types?
            end
            if type(value[1]) == "string" then
                local array = ffi.new("cstr[?]", #value, value)
                return Payload.FromStringArray(array, #value)
            end
        end

        if type(value) == "table" then
            return Payload.FromTable(self:valueToPayloadTable(value))
        end

        Log.Error("Unsupported payload type: " .. tostring(type(value)) .. ". Value: " .. tostring(value))
    else
        -- process Lua only payload
        local payloadId = PayloadConverter.CacheIndex
        PayloadConverter.Cache[payloadId] = value
        PayloadConverter.CacheIndex = PayloadConverter.CacheIndex + 1
        return Payload.FromLua(payloadId)
    end
end

-- Convert payload table into Lua one.
---@param payloadTable PayloadTable
function PayloadConverter:tablePayloadToValue(payloadTable)
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
---@param payload Payload?
function PayloadConverter:payloadToValue(payload)
    if payload == nil then
        return nil
    end

    local payloadType = payload:getType()
    if payloadType == PayloadType.Lua then
        local payloadId = tonumber(payload:getLua())
        -- TODO: clean cache?
        return PayloadConverter.Cache[payloadId]
    end
    if payloadType == PayloadType.Bool then return payload:getBool() end
    if payloadType == PayloadType.I8 then return payload:getI8() end
    if payloadType == PayloadType.U8 then return payload:getU8() end
    if payloadType == PayloadType.I16 then return payload:getI16() end
    if payloadType == PayloadType.U16 then return payload:getU16() end
    if payloadType == PayloadType.I32 then return payload:getI32() end
    if payloadType == PayloadType.U32 then return payload:getU32() end
    if payloadType == PayloadType.I64 then return payload:getI64() end
    if payloadType == PayloadType.U64 then return payload:getU64() end
    if payloadType == PayloadType.F32 then return payload:getF32() end
    if payloadType == PayloadType.F64 then return payload:getF64() end
    if payloadType == PayloadType.String then return ffi.string(payload:getString()) end
    if payloadType == PayloadType.Table then return self:tablePayloadToValue(payload:getTable()) end

    -- array types
    local result = {}
    local f = function(value)
        insert(result, value)
    end

    if payloadType == PayloadType.BoolArray then
        payload:forEachBool(f)
        return result
    end
    if payloadType == PayloadType.I8Array then
        payload:forEachI8(f)
        return result
    end
    if payloadType == PayloadType.U8Array then
        payload:forEachU8(f)
        return result
    end
    if payloadType == PayloadType.I16Array then
        payload:forEachI16(f)
        return result
    end
    if payloadType == PayloadType.U16Array then
        payload:forEachU16(f)
        return result
    end
    if payloadType == PayloadType.I32Array then
        payload:forEachI32(f)
        return result
    end
    if payloadType == PayloadType.U32Array then
        payload:forEachU32(f)
        return result
    end
    if payloadType == PayloadType.I64Array then
        payload:forEachI64(f)
        return result
    end
    if payloadType == PayloadType.U64Array then
        payload:forEachU64(f)
        return result
    end
    if payloadType == PayloadType.F32Array then
        payload:forEachF32(f)
        return result
    end
    if payloadType == PayloadType.F64Array then
        payload:forEachF64(f)
        return result
    end
    if payloadType == PayloadType.StringArray then
        payload:forEachString(function(value)
            insert(result, ffi.string(value))
        end)
        return result
    end

    Log.Error("Unexpected payload type: " .. tostring(payloadType))
end

return PayloadConverter
