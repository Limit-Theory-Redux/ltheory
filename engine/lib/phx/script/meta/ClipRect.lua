-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class ClipRect
ClipRect = {}

---@param x number
---@param y number
---@param sx number
---@param sy number
function ClipRect.Push(x, y, sx, sy) end

---@param x number
---@param y number
---@param sx number
---@param sy number
function ClipRect.PushCombined(x, y, sx, sy) end

function ClipRect.PushDisabled() end

---@param tx number
---@param ty number
---@param sx number
---@param sy number
function ClipRect.PushTransform(tx, ty, sx, sy) end

function ClipRect.Pop() end

function ClipRect.PopTransform() end

