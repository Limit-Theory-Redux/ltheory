-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Tex2D
Tex2D = {}

---@param sx integer
---@param sy integer
---@param format TexFormat
---@return Tex2D
function Tex2D.Create(sx, sy, format) end

---@param name string
---@return Tex2D
function Tex2D.Load(name) end

---@return Tex2D
function Tex2D:clone() end

---@param r Renderer
---@return Tex2D
function Tex2D.ScreenCapture(r) end

---@param path string
function Tex2D:save(path) end

---@param r Renderer
function Tex2D:pop(r) end

---@param r Renderer
function Tex2D:push(r) end

---@param r Renderer
---@param level integer
function Tex2D:pushLevel(r, level) end

---@param r Renderer
---@param red number
---@param green number
---@param blue number
---@param alpha number
function Tex2D:clear(r, red, green, blue, alpha) end

---@param r Renderer
---@return Tex2D
function Tex2D:deepClone(r) end

function Tex2D:genMipmap() end

---@param pf PixelFormat
---@param df DataFormat
---@return Bytes
function Tex2D:getDataBytes(pf, df) end

---@return TexFormat
function Tex2D:getFormat() end

---@return integer
function Tex2D:getHandle() end

---@return Vec2i
function Tex2D:getSize() end

---@param level integer
---@return Vec2i
function Tex2D:getSizeLevel(level) end

---@param factor number
function Tex2D:setAnisotropy(factor) end

---@param data Bytes
---@param pf PixelFormat
---@param df DataFormat
function Tex2D:setDataBytes(data, pf, df) end

---@param filter TexFilter
function Tex2D:setMagFilter(filter) end

---@param filter TexFilter
function Tex2D:setMinFilter(filter) end

---@param minLevel integer
---@param maxLevel integer
function Tex2D:setMipRange(minLevel, maxLevel) end

---@param x integer
---@param y integer
---@param r number
---@param g number
---@param b number
---@param a number
function Tex2D:setTexel(x, y, r, g, b, a) end

---@param mode TexWrapMode
function Tex2D:setWrapMode(mode) end

-- Sample a single pixel at integer coordinates (x, y)
-- Coordinates are in OpenGL convention: (0,0) = bottom-left
-- Returns Vec3f with RGB in [0.0, 1.0] range
---@param x integer
---@param y integer
---@return Vec3f
function Tex2D:sample(x, y) end

