-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Tex1D
Tex1D = {}

---@param size integer
---@param format TexFormat
---@return Tex1D
function Tex1D.Create(size, format) end

---@return Tex1D
function Tex1D:clone() end

function Tex1D:genMipmap() end

---@return TexFormat
function Tex1D:getFormat() end

---@param pf PixelFormat
---@param df DataFormat
---@return Bytes
function Tex1D:getDataBytes(pf, df) end

---@return integer
function Tex1D:getHandle() end

---@return integer
function Tex1D:getSize() end

---@param data Bytes
---@param pf PixelFormat
---@param df DataFormat
function Tex1D:setDataBytes(data, pf, df) end

---@param filter TexFilter
function Tex1D:setMagFilter(filter) end

---@param filter TexFilter
function Tex1D:setMinFilter(filter) end

---@param x integer
---@param r number
---@param g number
---@param b number
---@param a number
function Tex1D:setTexel(x, r, g, b, a) end

---@param mode TexWrapMode
function Tex1D:setWrapMode(mode) end

