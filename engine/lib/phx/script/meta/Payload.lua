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

-- Address of the first element of the array (cast with ffi.cast) (valid for `array_len` elements while the payload lives).
---@return integer
function Payload:getBoolArrayAddr() end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromI8Array(value, value_size) end

-- Address of the first element of the array (cast with ffi.cast) (valid for `array_len` elements while the payload lives).
---@return integer
function Payload:getI8ArrayAddr() end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromU8Array(value, value_size) end

-- Address of the first element of the array (cast with ffi.cast) (valid for `array_len` elements while the payload lives).
---@return integer
function Payload:getU8ArrayAddr() end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromI16Array(value, value_size) end

-- Address of the first element of the array (cast with ffi.cast) (valid for `array_len` elements while the payload lives).
---@return integer
function Payload:getI16ArrayAddr() end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromU16Array(value, value_size) end

-- Address of the first element of the array (cast with ffi.cast) (valid for `array_len` elements while the payload lives).
---@return integer
function Payload:getU16ArrayAddr() end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromI32Array(value, value_size) end

-- Address of the first element of the array (cast with ffi.cast) (valid for `array_len` elements while the payload lives).
---@return integer
function Payload:getI32ArrayAddr() end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromU32Array(value, value_size) end

-- Address of the first element of the array (cast with ffi.cast) (valid for `array_len` elements while the payload lives).
---@return integer
function Payload:getU32ArrayAddr() end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromI64Array(value, value_size) end

-- Address of the first element of the array (cast with ffi.cast) (valid for `array_len` elements while the payload lives).
---@return integer
function Payload:getI64ArrayAddr() end

---@param value integer[]
---@param value_size integer
---@return Payload
function Payload.FromU64Array(value, value_size) end

-- Address of the first element of the array (cast with ffi.cast) (valid for `array_len` elements while the payload lives).
---@return integer
function Payload:getU64ArrayAddr() end

---@param value number[]
---@param value_size integer
---@return Payload
function Payload.FromF32Array(value, value_size) end

-- Address of the first element of the array (cast with ffi.cast) (valid for `array_len` elements while the payload lives).
---@return integer
function Payload:getF32ArrayAddr() end

---@param value number[]
---@param value_size integer
---@return Payload
function Payload.FromF64Array(value, value_size) end

-- Address of the first element of the array (cast with ffi.cast) (valid for `array_len` elements while the payload lives).
---@return integer
function Payload:getF64ArrayAddr() end

---@param value string[]
---@param value_size integer
---@return Payload
function Payload.FromStringArray(value, value_size) end

-- Returns the string at `index` of a string array. Panics if out of range.
---@param index integer
---@return string
function Payload:getStringArrayItem(index) end

-- Number of elements of any array payload type. Zero for non-array payloads.
---@return integer
function Payload:arrayLen() end

---@param value PayloadTable
---@return Payload
function Payload.FromTable(value) end

---@return PayloadTable
function Payload:getTable() end

---@return PayloadType
function Payload:getType() end

