-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Tex3D
Tex3D = {}

---@param r Renderer
---@param sx integer
---@param sy integer
---@param sz integer
---@param format TexFormat
---@return Tex3D
function Tex3D.Create(r, sx, sy, sz, format) end

---@param r Renderer
function Tex3D:pop(r) end

---@param r Renderer
---@param layer integer
function Tex3D:push(r, layer) end

---@param r Renderer
---@param layer integer
---@param level integer
function Tex3D:pushLevel(r, layer, level) end

---@param r Renderer
function Tex3D:genMipmap(r) end

---@param r Renderer
---@param pf PixelFormat
---@param df DataFormat
---@return Bytes
function Tex3D:getDataBytes(r, pf, df) end

---@return TexFormat
function Tex3D:getFormat() end

---@return Vec3i
function Tex3D:getSize() end

---@param level integer
---@return Vec3i
function Tex3D:getSizeLevel(level) end

---@param r Renderer
---@param data Bytes
---@param pf PixelFormat
---@param df DataFormat
function Tex3D:setDataBytes(r, data, pf, df) end

---@param r Renderer
---@param filter TexFilter
function Tex3D:setMagFilter(r, filter) end

---@param r Renderer
---@param filter TexFilter
function Tex3D:setMinFilter(r, filter) end

---@param r Renderer
---@param mode TexWrapMode
function Tex3D:setWrapMode(r, mode) end

