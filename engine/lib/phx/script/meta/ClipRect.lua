-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class ClipRect
ClipRect = {}

---@param r Renderer
---@param x number
---@param y number
---@param sx number
---@param sy number
function ClipRect.Push(r, x, y, sx, sy) end

---@param r Renderer
---@param x number
---@param y number
---@param sx number
---@param sy number
function ClipRect.PushCombined(r, x, y, sx, sy) end

---@param r Renderer
function ClipRect.PushDisabled(r) end

---@param r Renderer
---@param tx number
---@param ty number
---@param sx number
---@param sy number
function ClipRect.PushTransform(r, tx, ty, sx, sy) end

---@param r Renderer
function ClipRect.Pop(r) end

---@param r Renderer
function ClipRect.PopTransform(r) end

