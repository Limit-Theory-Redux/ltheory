---@meta

---@class MouseState
MouseState = {}

---@param control MouseControl
---@return number
function MouseState:value(self, control) end

---@param control MouseControl
---@return boolean
function MouseState:isPressed(self, control) end

---@param control MouseControl
---@return boolean
function MouseState:isDown(self, control) end

---@param control MouseControl
---@return boolean
function MouseState:isReleased(self, control) end

---@return Vec2
function MouseState:delta(self) end

---@return Vec2
function MouseState:scroll(self) end

---@return Vec2
function MouseState:scrollPixel(self) end

---@return Vec2
function MouseState:position(self) end

---@return boolean
function MouseState:inWindow(self) end

