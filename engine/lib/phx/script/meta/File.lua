-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class File
File = {}

---@param path string
---@return boolean
function File.Exists(path) end

---@param path string
---@return boolean
function File.IsDir(path) end

---@param path string
---@return File?
function File.Create(path) end

---@param path string
---@return File?
function File.Open(path) end

---@return boolean
function File:close() end

---@param path string
---@return Bytes?
function File.ReadBytes(path) end

---@param path string
---@return string?
function File.ReadCstr(path) end

---@param path string
---@return integer?
function File.Size(path) end

---@param data integer[]
---@param data_size integer
---@return integer?
function File:read(data, data_size) end

---@param data integer[]
---@param data_size integer
---@return integer?
function File:write(data, data_size) end

---@param data string
---@return integer?
function File:writeStr(data) end

---@return integer
function File:readU8() end

---@return integer
function File:readU16() end

---@return integer
function File:readU32() end

---@return integer
function File:readU64() end

---@return integer
function File:readI8() end

---@return integer
function File:readI16() end

---@return integer
function File:readI32() end

---@return integer
function File:readI64() end

---@return number
function File:readF32() end

---@return number
function File:readF64() end

---@param value integer
function File:writeU8(value) end

---@param value integer
function File:writeU16(value) end

---@param value integer
function File:writeU32(value) end

---@param value integer
function File:writeU64(value) end

---@param value integer
function File:writeI8(value) end

---@param value integer
function File:writeI16(value) end

---@param value integer
function File:writeI32(value) end

---@param value integer
function File:write64(value) end

---@param value number
function File:writeF32(value) end

---@param value number
function File:writeF64(value) end

