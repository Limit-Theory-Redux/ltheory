---@meta

---@class TouchpadState
TouchpadState = {}

---@param axis TouchpadAxis
---@return number
function TouchpadState:value(axis) end

---@return Vec2
function TouchpadState:position() end

---@return number
function TouchpadState:magnifyDelta() end

---@return number
function TouchpadState:rotateDelta() end

