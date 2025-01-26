-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class TexCube
TexCube = {}

---@param size integer
---@param format TexFormat
---@return TexCube
function TexCube.Create(size, format) end

---@param path string
---@return TexCube
function TexCube.Load(path) end

---@param r number
---@param g number
---@param b number
---@param a number
function TexCube:clear(r, g, b, a) end

---@param path string
function TexCube:save(path) end

---@param path string
---@param level integer
function TexCube:saveLevel(path, level) end

---@param face CubeFace
---@param level integer
---@param pf PixelFormat
---@param df DataFormat
---@return Bytes
function TexCube:getDataBytes(face, level, pf, df) end

---@return TexFormat
function TexCube:getFormat() end

---@return integer
function TexCube:getHandle() end

---@return integer
function TexCube:getSize() end

---@param state ShaderState
function TexCube:generate(state) end

function TexCube:genMipmap() end

---@param data Bytes
---@param face CubeFace
---@param level integer
---@param pf PixelFormat
---@param df DataFormat
function TexCube:setDataBytes(data, face, level, pf, df) end

---@param filter TexFilter
function TexCube:setMagFilter(filter) end

---@param filter TexFilter
function TexCube:setMinFilter(filter) end

---@param sampleCount integer
---@return TexCube
function TexCube:genIRMap(sampleCount) end

