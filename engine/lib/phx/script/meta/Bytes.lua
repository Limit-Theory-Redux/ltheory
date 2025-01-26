-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Bytes
Bytes = {}

---@param size integer
---@return Bytes
function Bytes.Create(size) end

---@param capacity integer
---@return Bytes
function Bytes.CreateWithCapacity(capacity) end

---@param data integer[]
---@param data_size integer
---@return Bytes
function Bytes.FromData(data, data_size) end

---@param path string
---@return Bytes
function Bytes.Load(path) end

---@return integer
function Bytes:getSize() end

---@return boolean
function Bytes:isEmpty() end

---@return Bytes
function Bytes:compress() end

---@return Bytes
function Bytes:decompress() end

---@param path string
function Bytes:save(path) end

---@return integer
function Bytes:getCursor() end

function Bytes:rewind() end

---@param cursor integer
function Bytes:setCursor(cursor) end

---@param data integer[]
---@param data_size integer
function Bytes:read(data, data_size) end

---@return integer
function Bytes:readU8() end

---@return integer
function Bytes:readU16() end

---@return integer
function Bytes:readU32() end

---@return integer
function Bytes:readU64() end

---@return integer
function Bytes:readI8() end

---@return integer
function Bytes:readI16() end

---@return integer
function Bytes:readI32() end

---@return integer
function Bytes:readI64() end

---@return number
function Bytes:readF32() end

---@return number
function Bytes:readF64() end

---@param data integer[]
---@param data_size integer
function Bytes:write(data, data_size) end

---@param data string
function Bytes:writeStr(data) end

---@param value integer
function Bytes:writeU8(value) end

---@param value integer
function Bytes:writeU16(value) end

---@param value integer
function Bytes:writeU32(value) end

---@param value integer
function Bytes:writeU64(value) end

---@param value integer
function Bytes:writeI8(value) end

---@param value integer
function Bytes:writeI16(value) end

---@param value integer
function Bytes:writeI32(value) end

---@param value integer
function Bytes:writeI64(value) end

---@param value number
function Bytes:writeF32(value) end

---@param value number
function Bytes:writeF64(value) end

