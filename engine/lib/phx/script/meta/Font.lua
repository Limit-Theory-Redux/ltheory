---@meta

---@class Font
Font = {}

---@param name string
---@param size integer
---@return Font
function Font.Load(name, size) end

---@param text string
---@param x number
---@param y number
---@param color Color
function Font:draw(text, x, y, color) end

---@param text string
---@param x number
---@param y number
function Font:drawShaded(text, x, y) end

---@return integer
function Font:getLineHeight() end

---@param text string
---@param out Vec4i
function Font:getSize(text, out) end

---@param text string
---@return Vec2i
function Font:getSize2(text) end

