---@meta

---@class Tex3D
Tex3D = {}

---@param sx integer
---@param sy integer
---@param sz integer
---@param format TexFormat
---@return Tex3D
function Tex3D.Create(sx, sy, sz, format) end

function Tex3D:pop() end

---@param layer integer
function Tex3D:push(layer) end

---@param layer integer
---@param level integer
function Tex3D:pushLevel(layer, level) end

function Tex3D:genMipmap() end

---@param pf PixelFormat
---@param df DataFormat
---@return Bytes
function Tex3D:getDataBytes(pf, df) end

---@return TexFormat
function Tex3D:getFormat() end

---@return integer
function Tex3D:getHandle() end

---@return Vec3i
function Tex3D:getSize() end

---@param level integer
---@return Vec3i
function Tex3D:getSizeLevel(level) end

---@param data Bytes
---@param pf PixelFormat
---@param df DataFormat
function Tex3D:setDataBytes(data, pf, df) end

---@param filter TexFilter
function Tex3D:setMagFilter(filter) end

---@param filter TexFilter
function Tex3D:setMinFilter(filter) end

---@param mode TexWrapMode
function Tex3D:setWrapMode(mode) end

