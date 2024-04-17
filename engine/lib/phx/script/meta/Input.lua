---@meta

---@class Input
Input = {}

---@return KeyboardState
function Input:keyboard(self) end

---@return MouseState
function Input:mouse(self) end

---@return TouchpadState
function Input:touchpad(self) end

---@return GamepadState
function Input:gamepad(self) end

---@return DragAndDropState
function Input:dragAndDrop(self) end

---@return InputDevice
function Input:activeDevice(self) end

---@return InputDeviceType
function Input:activeDeviceType(self) end

---@return InputDeviceId
function Input:activeDeviceId(self) end

---@param visible boolean
function Input:setCursorVisible(self, visible) end

function Input:setCursorVisibleAuto(self) end

---@param x number
---@param y number
function Input:setCursorPosition(self, x, y) end

---@param button Button
---@return boolean
function Input:isPressed(self, button) end

---@param button Button
---@return boolean
function Input:isDown(self, button) end

---@param button Button
---@return boolean
function Input:isReleased(self, button) end

---@param button Button
---@return number
function Input:getValue(self, button) end

---@return boolean
function Input:isKeyboardAltPressed(self) end

---@return boolean
function Input:isKeyboardCtrlPressed(self) end

---@return boolean
function Input:isKeyboardShiftPressed(self) end

