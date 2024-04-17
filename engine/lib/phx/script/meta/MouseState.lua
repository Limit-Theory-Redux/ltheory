---@meta

---@class MouseState
MouseState = {}

---@param control MouseControl
---@return number
function MouseState:value(control) end

---@param control MouseControl
---@return boolean
function MouseState:isPressed(control) end

---@param control MouseControl
---@return boolean
function MouseState:isDown(control) end

---@param control MouseControl
---@return boolean
function MouseState:isReleased(control) end

---@return Vec2
function MouseState:delta() end

---@return Vec2
function MouseState:scroll() end

---@return Vec2
function MouseState:scrollPixel() end

---@return Vec2
function MouseState:position() end

---@return boolean
function MouseState:inWindow() end

