-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Font
Font = {}

---@param r Renderer
---@param name string
---@param size integer
---@return Font
function Font.Load(r, name, size) end

---@param r Renderer
---@param text string
---@param x number
---@param y number
---@param color Color
function Font:draw(r, text, x, y, color) end

---@return integer
function Font:getLineHeight() end

---@param r Renderer
---@param text string
---@param out Vec4i
function Font:getSize(r, text, out) end

---@param r Renderer
---@param text string
---@return Vec2i
function Font:getSize2(r, text) end

