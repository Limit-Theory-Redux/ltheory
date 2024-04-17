---@meta

TouchpadState = TouchpadState

---@param axis TouchpadAxis
---@return number
function TouchpadState:value(self, axis) end

---@return Vec2
function TouchpadState:position(self) end

---@return number
function TouchpadState:magnifyDelta(self) end

---@return number
function TouchpadState:rotateDelta(self) end

