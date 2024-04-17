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
---@param r number
---@param g number
---@param b number
---@param a number
function Font:draw(text, x, y, r, g, b, a) end

---@param text string
---@param x number
---@param y number
function Font:drawShaded(text, x, y) end

---@return integer
function Font:getLineHeight() end

---@param text string
---@param out IVec4
function Font:getSize(text, out) end

---@param text string
---@return IVec2
function Font:getSize2(text) end

