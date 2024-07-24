---@meta

---@class InputImpl
InputImpl = {}

---@return KeyboardState
function InputImpl:keyboard() end

---@return MouseState
function InputImpl:mouse() end

---@return TouchpadState
function InputImpl:touchpad() end

---@return GamepadState
function InputImpl:gamepad() end

---@return DragAndDropState
function InputImpl:dragAndDrop() end

---@return InputDevice
function InputImpl:activeDevice() end

---@return InputDeviceType
function InputImpl:activeDeviceType() end

---@return InputDeviceId
function InputImpl:activeDeviceId() end

---@param visible boolean
function InputImpl:setCursorVisible(visible) end

function InputImpl:setCursorVisibleAuto() end

---@param x number
---@param y number
function InputImpl:setCursorPosition(x, y) end

---@param button Button
---@return boolean
function InputImpl:isPressed(button) end

---@param button Button
---@return boolean
function InputImpl:isDown(button) end

---@param button Button
---@return boolean
function InputImpl:isReleased(button) end

---@param button Button
---@return number
function InputImpl:getValue(button) end

---@return boolean
function InputImpl:isKeyboardAltPressed() end

---@return boolean
function InputImpl:isKeyboardAltDown() end

---@return boolean
function InputImpl:isKeyboardCtrlPressed() end

---@return boolean
function InputImpl:isKeyboardCtrlDown() end

---@return boolean
function InputImpl:isKeyboardShiftPressed() end

---@return boolean
function InputImpl:isKeyboardShiftDown() end

