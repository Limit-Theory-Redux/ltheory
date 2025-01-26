-- AUTO GENERATED. DO NOT MODIFY!
---@meta

-- Payload value.
---@class Payload
Payload = {}

---@param value integer
---@return Payload
function Payload.FromLua(value) end

---@return integer
function Payload:getLua() end

---@param value boolean
---@return Payload
function Payload.FromBool(value) end

---@return boolean
function Payload:getBool() end

---@param value integer
---@return Payload
function Payload.FromI8(value) end

---@return integer
function Payload:getI8() end

---@param value integer
---@return Payload
function Payload.FromU8(value) end

---@return integer
function Payload:getU8() end

---@param value integer
---@return Payload
function Payload.FromI16(value) end

---@return integer
function Payload:getI16() end

---@param value integer
---@return Payload
function Payload.FromU16(value) end

---@return integer
function Payload:getU16() end

---@param value integer
---@return Payload
function Payload.FromI32(value) end

---@return integer
function Payload:getI32() end

---@param value integer
---@return Payload
function Payload.FromU32(value) end

---@return integer
function Payload:getU32() end

---@param value integer
---@return Payload
function Payload.FromI64(value) end

---@return integer
function Payload:getI64() end

---@param value integer
---@return Payload
function Payload.FromU64(value) end

---@return integer
function Payload:getU64() end

---@param value number
---@return Payload
function Payload.FromF32(value) end

---@return number
function Payload:getF32() end

---@param value number
---@return Payload
function Payload.FromF64(value) end

---@return number
function Payload:getF64() end

---@param value string
---@return Payload
function Payload.FromString(value) end

---@return string
function Payload:getString() end

---@param value boolean[]
---@param value_size integer
---@return Payload
function Payload.FromBoolArray(value, value_size) end

---@param f fun(arg1: boolean): nil
function Payload:forEachBool(f) end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromI8Array(value, value_size) end

---@param f fun(arg1: integer): nil
function Payload:forEachI8(f) end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromU8Array(value, value_size) end

---@param f fun(arg1: integer): nil
function Payload:forEachU8(f) end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromI16Array(value, value_size) end

---@param f fun(arg1: integer): nil
function Payload:forEachI16(f) end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromU16Array(value, value_size) end

---@param f fun(arg1: integer): nil
function Payload:forEachU16(f) end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromI32Array(value, value_size) end

---@param f fun(arg1: integer): nil
function Payload:forEachI32(f) end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromU32Array(value, value_size) end

---@param f fun(arg1: integer): nil
function Payload:forEachU32(f) end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromI64Array(value, value_size) end

---@param f fun(arg1: integer): nil
function Payload:forEachI64(f) end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromU64Array(value, value_size) end

---@param f fun(arg1: integer): nil
function Payload:forEachU64(f) end

---@param value number[]
---@param value_size integer
---@return Payload
function Payload.FromF32Array(value, value_size) end

---@param f fun(arg1: number): nil
function Payload:forEachF32(f) end

---@param value number[]
---@param value_size integer
---@return Payload
function Payload.FromF64Array(value, value_size) end

---@param f fun(arg1: number): nil
function Payload:forEachF64(f) end

---@param value string[]
---@param value_size integer
---@return Payload
function Payload.FromStringArray(value, value_size) end

---@param f fun(arg1: string): nil
function Payload:forEachString(f) end

---@param value PayloadTable
---@return Payload
function Payload.FromTable(value) end

---@return PayloadTable
function Payload:getTable() end

---@return PayloadType
function Payload:getType() end

