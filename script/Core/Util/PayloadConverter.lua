---@class PayloadConverter
local PayloadConverter = {
    Cache = {},
    CacheIndex = 1, -- TODO: reusable indices/cache slots
}
PayloadConverter.__index = PayloadConverter

local hasNewTable, newTable = pcall(require, 'table.new')
if not hasNewTable then
    newTable = function() return {} end
end

-- payload type -> element access description (built lazily: PayloadType is a runtime global)
PayloadConverter.ArrayTypes = nil
local function initArrayTypes()
    local function info(getter, ctype, convert)
        return { getter = getter, ctype = ctype, convert = convert }
    end
    PayloadConverter.ArrayTypes = {
        [PayloadType.BoolArray] = info("getBoolArrayAddr", "uint8_t const*", "bool"),
        [PayloadType.I8Array] = info("getI8ArrayAddr", "int8_t const*"),
        [PayloadType.U8Array] = info("getU8ArrayAddr", "uint8_t const*"),
        [PayloadType.I16Array] = info("getI16ArrayAddr", "int16_t const*"),
        [PayloadType.U16Array] = info("getU16ArrayAddr", "uint16_t const*"),
        [PayloadType.I32Array] = info("getI32ArrayAddr", "int32_t const*"),
        [PayloadType.U32Array] = info("getU32ArrayAddr", "uint32_t const*"),
        [PayloadType.I64Array] = info("getI64ArrayAddr", "int64_t const*", "number"),
        [PayloadType.U64Array] = info("getU64ArrayAddr", "uint64_t const*", "number"),
        [PayloadType.F32Array] = info("getF32ArrayAddr", "float const*"),
        [PayloadType.F64Array] = info("getF64ArrayAddr", "double const*"),
        [PayloadType.StringArray] = info(nil, nil),
    }
end

-- Classifies a table.
-- Returns the element kind ("boolean" | "number" | "string") and the length for a proper non-empty
-- sequence (keys are exactly 1..n, all elements of the same kind), "mixed" for a sequence with
-- mixed/unsupported element types and nil for anything else (empty or has other keys).
---@param value table
---@return string? kind
---@return integer? count
function PayloadConverter.SequenceKind(value)
    local n = #value
    if n == 0 then
        return nil
    end
    -- every key must be an integer in 1..n: count them
    local keys = 0
    for _ in pairs(value) do
        keys = keys + 1
    end
    if keys ~= n then
        return nil
    end
    local kind = rawtype(value[1])
    if kind ~= "boolean" and kind ~= "number" and kind ~= "string" then
        return "mixed", n
    end
    for i = 2, n do
        if rawtype(value[i]) ~= kind then
            return "mixed", n
        end
    end
    return kind, n
end

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
        if rawtype(value) == "nil" then
            return nil
        end
        if rawtype(value) == "boolean" then
            return Payload.FromBool(value)
        end
        -- LuaJIT has a single number type, so every number is sent as F64 (lossless for doubles).
        if rawtype(value) == "number" then
            return Payload.FromF64(value)
        end
        if rawtype(value) == "string" then
            return Payload.FromString(value)
        end

        if rawtype(value) == "table" then
            local kind, count = PayloadConverter.SequenceKind(value)
            if kind == "boolean" then
                local array = ffi.new("bool[?]", count, value)
                return Payload.FromBoolArray(array, count)
            end
            if kind == "number" then
                local array = ffi.new("double[?]", count, value)
                return Payload.FromF64Array(array, count)
            end
            if kind == "string" then
                local array = ffi.new("cstr[?]", count, value)
                return Payload.FromStringArray(array, count)
            end
            if kind == "mixed" then
                Log.Error("Unsupported payload: array with mixed or unsupported element types")
                return nil
            end
            -- kind == nil: not a sequence (empty or has other keys): send as a table
            return Payload.FromTable(self:valueToPayloadTable(value))
        end

        -- TODO: ffi.istype(Payload, value)
        if tostring(ffi.typeof(value)) == "ctype<struct Payload *>" then
            return value
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

        result[ffi.string(name)] = self:payloadToValue(payload)
    end
    return result
end

-- Convert payload into the lua value.
---@param payload Payload?
function PayloadConverter:payloadToValue(payload)
    if payload == nil then
        return nil
    end

    if PayloadConverter.ArrayTypes == nil then
        initArrayTypes()
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

    -- array types: bulk access through the raw element address, no per-element FFI calls or callbacks
    local arrayInfo = PayloadConverter.ArrayTypes[payloadType]
    if arrayInfo ~= nil then
        local count = tonumber(payload:arrayLen())
        local result = newTable(count, 0)
        if count == 0 then
            return result
        end
        if payloadType == PayloadType.StringArray then
            for i = 0, count - 1 do
                result[i + 1] = ffi.string(payload:getStringArrayItem(i))
            end
            return result
        end
        local ptr = ffi.cast(arrayInfo.ctype, payload[arrayInfo.getter](payload))
        if arrayInfo.convert == "bool" then
            for i = 0, count - 1 do result[i + 1] = ptr[i] ~= 0 end
        elseif arrayInfo.convert == "number" then
            for i = 0, count - 1 do result[i + 1] = tonumber(ptr[i]) end
        else
            for i = 0, count - 1 do result[i + 1] = ptr[i] end
        end
        return result
    end

    Log.Error("Unexpected payload type: " .. tostring(payloadType))
end

return PayloadConverter
