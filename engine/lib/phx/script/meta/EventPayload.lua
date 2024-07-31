---@meta

---@class EventPayload
EventPayload = {}

---@param value integer
---@return EventPayload
function EventPayload.FromLua(value) end

---@return integer
function EventPayload:getLua() end

---@param value boolean
---@return EventPayload
function EventPayload.FromBool(value) end

---@return boolean
function EventPayload:getBool() end

---@param value integer
---@return EventPayload
function EventPayload.FromI8(value) end

---@return integer
function EventPayload:getI8() end

---@param value integer
---@return EventPayload
function EventPayload.FromU8(value) end

---@return integer
function EventPayload:getU8() end

---@param value integer
---@return EventPayload
function EventPayload.FromI16(value) end

---@return integer
function EventPayload:getI16() end

---@param value integer
---@return EventPayload
function EventPayload.FromU16(value) end

---@return integer
function EventPayload:getU16() end

---@param value integer
---@return EventPayload
function EventPayload.FromI32(value) end

---@return integer
function EventPayload:getI32() end

---@param value integer
---@return EventPayload
function EventPayload.FromU32(value) end

---@return integer
function EventPayload:getU32() end

---@param value integer
---@return EventPayload
function EventPayload.FromI64(value) end

---@return integer
function EventPayload:getI64() end

---@param value integer
---@return EventPayload
function EventPayload.FromU64(value) end

---@return integer
function EventPayload:getU64() end

---@param value number
---@return EventPayload
function EventPayload.FromF32(value) end

---@return number
function EventPayload:getF32() end

---@param value number
---@return EventPayload
function EventPayload.FromF64(value) end

---@return number
function EventPayload:getF64() end

---@param value string
---@return EventPayload
function EventPayload.FromString(value) end

---@return string
function EventPayload:getString() end

---@param value EventPayloadTable
---@return EventPayload
function EventPayload.FromTable(value) end

---@return EventPayloadTable
function EventPayload:getTable() end

---@return EventPayloadType
function EventPayload:getType() end

