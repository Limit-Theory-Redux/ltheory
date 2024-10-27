---@meta

---@class Input
Input = {}

---@return KeyboardState
function Input:keyboard() end

---@return MouseState
function Input:mouse() end

---@return TouchpadState
function Input:touchpad() end

---@return GamepadState
function Input:gamepad() end

---@return DragAndDropState
function Input:dragAndDrop() end

---@return InputDevice?
function Input:activeDevice() end

---@return InputDeviceType?
function Input:activeDeviceType() end

---@return InputDeviceId?
function Input:activeDeviceId() end

---@param visible boolean
function Input:setCursorVisible(visible) end

function Input:setCursorVisibleAuto() end

---@param x number
---@param y number
function Input:setCursorPosition(x, y) end

---@param button Button
---@return boolean
function Input:isPressed(button) end

---@param button Button
---@return boolean
function Input:isDown(button) end

---@param button Button
---@return boolean
function Input:isReleased(button) end

---@param button Button
---@return number
function Input:getValue(button) end

---@return boolean
function Input:isKeyboardAltPressed() end

---@return boolean
function Input:isKeyboardAltDown() end

---@return boolean
function Input:isKeyboardCtrlPressed() end

---@return boolean
function Input:isKeyboardCtrlDown() end

---@return boolean
function Input:isKeyboardShiftPressed() end

---@return boolean
function Input:isKeyboardShiftDown() end

