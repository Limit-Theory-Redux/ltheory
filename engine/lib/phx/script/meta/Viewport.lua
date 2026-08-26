-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Viewport
Viewport = {}

---@param r Renderer
---@return number
function Viewport.GetAspect(r) end

---@param r Renderer
---@param result Vec2i [out]
function Viewport.GetSize(r, result) end

---@param r Renderer
---@param x integer
---@param y integer
---@param sx integer
---@param sy integer
---@param isWindow boolean
function Viewport.Push(r, x, y, sx, sy, isWindow) end

---@param r Renderer
function Viewport.Pop(r) end

