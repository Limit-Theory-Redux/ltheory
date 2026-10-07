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

-- View of one z-slice at mip level 0, usable as a render attachment.
---@param layer integer
---@return TexView
function Tex3D:layerView(layer) end

-- View of one z-slice at the given mip level, usable as a render attachment.
---@param layer integer
---@param level integer
---@return TexView
function Tex3D:layerMipView(layer, level) end

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

