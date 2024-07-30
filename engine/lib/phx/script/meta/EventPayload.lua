---@meta

---@class EventPayload
EventPayload = {}

---@param value integer
---@return EventPayload
function EventPayload.FromLua(value) end

---@return integer
function EventPayload:asLua() end

---@param value boolean
---@return EventPayload
function EventPayload.FromBool(value) end

---@return boolean
function EventPayload:asBool() end

---@param value integer
---@return EventPayload
function EventPayload.FromI8(value) end

---@return integer
function EventPayload:asI8() end

---@param value integer
---@return EventPayload
function EventPayload.FromU8(value) end

---@return integer
function EventPayload:asU8() end

---@param value integer
---@return EventPayload
function EventPayload.FromI16(value) end

---@return integer
function EventPayload:asI16() end

---@param value integer
---@return EventPayload
function EventPayload.FromU16(value) end

---@return integer
function EventPayload:asU16() end

---@param value integer
---@return EventPayload
function EventPayload.FromI32(value) end

---@return integer
function EventPayload:asI32() end

---@param value integer
---@return EventPayload
function EventPayload.FromU32(value) end

---@return integer
function EventPayload:asU32() end

---@param value integer
---@return EventPayload
function EventPayload.FromI64(value) end

---@return integer
function EventPayload:asI64() end

---@param value integer
---@return EventPayload
function EventPayload.FromU64(value) end

---@return integer
function EventPayload:asU64() end

---@param value number
---@return EventPayload
function EventPayload.FromF32(value) end

---@return number
function EventPayload:asF32() end

---@param value number
---@return EventPayload
function EventPayload.FromF64(value) end

---@return number
function EventPayload:asF64() end

---@param value string
---@return EventPayload
function EventPayload.FromString(value) end

---@return string
function EventPayload:asString() end

---@param value EventPayloadTable
---@return EventPayload
function EventPayload.FromTable(value) end

---@return EventPayloadTable
function EventPayload:asTable() end

---@return EventPayloadType
function EventPayload:getType() end

