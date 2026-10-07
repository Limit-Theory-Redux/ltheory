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

-- A texture with `mips` levels (0 = the full chain) and the `TexUsage`
-- bits in `usage` (0 = the default for the kind).
---@param r Renderer
---@param sx integer
---@param sy integer
---@param sz integer
---@param format TexFormat
---@param mips integer
---@param usage integer
---@return Tex3D
function Tex3D.CreateDesc(r, sx, sy, sz, format, mips, usage) end

-- View of the whole volume, for sampling.
---@return TexView
function Tex3D:view() end

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

