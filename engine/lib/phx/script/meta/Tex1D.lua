-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Tex1D
Tex1D = {}

---@param r Renderer
---@param size integer
---@param format TexFormat
---@return Tex1D
function Tex1D.Create(r, size, format) end

---@return Tex1D
function Tex1D:clone() end

---@param r Renderer
function Tex1D:genMipmap(r) end

---@return TexFormat
function Tex1D:getFormat() end

---@param r Renderer
---@param pf PixelFormat
---@param df DataFormat
---@return Bytes
function Tex1D:getDataBytes(r, pf, df) end

---@return integer
function Tex1D:getSize() end

---@param r Renderer
---@param data Bytes
---@param pf PixelFormat
---@param df DataFormat
function Tex1D:setDataBytes(r, data, pf, df) end

---@param r Renderer
---@param filter TexFilter
function Tex1D:setMagFilter(r, filter) end

---@param r Renderer
---@param filter TexFilter
function Tex1D:setMinFilter(r, filter) end

---@param r Renderer
---@param x integer
---@param red number
---@param green number
---@param blue number
---@param alpha number
function Tex1D:setTexel(r, x, red, green, blue, alpha) end

---@param r Renderer
---@param mode TexWrapMode
function Tex1D:setWrapMode(r, mode) end

