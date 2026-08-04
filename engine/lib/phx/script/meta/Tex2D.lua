-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Tex2D
Tex2D = {}

---@param r Renderer
---@param sx integer
---@param sy integer
---@param format TexFormat
---@return Tex2D
function Tex2D.Create(r, sx, sy, format) end

---@param r Renderer
---@param name string
---@return Tex2D
function Tex2D.Load(r, name) end

---@return Tex2D
function Tex2D:clone() end

---@param r Renderer
---@return Tex2D
function Tex2D.ScreenCapture(r) end

---@param r Renderer
---@param path string
function Tex2D:save(r, path) end

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

---@param r Renderer
function Tex2D:genMipmap(r) end

---@param r Renderer
---@param pf PixelFormat
---@param df DataFormat
---@return Bytes
function Tex2D:getDataBytes(r, pf, df) end

---@return TexFormat
function Tex2D:getFormat() end

---@return Vec2i
function Tex2D:getSize() end

---@param level integer
---@return Vec2i
function Tex2D:getSizeLevel(level) end

---@param r Renderer
---@param factor number
function Tex2D:setAnisotropy(r, factor) end

---@param r Renderer
---@param data Bytes
---@param pf PixelFormat
---@param df DataFormat
function Tex2D:setDataBytes(r, data, pf, df) end

---@param r Renderer
---@param filter TexFilter
function Tex2D:setMagFilter(r, filter) end

---@param r Renderer
---@param filter TexFilter
function Tex2D:setMinFilter(r, filter) end

---@param r Renderer
---@param minLevel integer
---@param maxLevel integer
function Tex2D:setMipRange(r, minLevel, maxLevel) end

---@param r Renderer
---@param x integer
---@param y integer
---@param red number
---@param green number
---@param blue number
---@param alpha number
function Tex2D:setTexel(r, x, y, red, green, blue, alpha) end

---@param r Renderer
---@param mode TexWrapMode
function Tex2D:setWrapMode(r, mode) end

-- Sample a single pixel at integer coordinates (x, y)
-- Coordinates are in OpenGL convention: (0,0) = bottom-left
-- Returns Vec3f with RGB in [0.0, 1.0] range
---@param r Renderer
---@param x integer
---@param y integer
---@return Vec3f
function Tex2D:sample(r, x, y) end

