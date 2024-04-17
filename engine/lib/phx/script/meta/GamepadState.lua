---@meta

---@class GamepadState
GamepadState = {}

---@return integer
function GamepadState:gamepadsCount() end

---@param index integer
---@return GamepadId
function GamepadState:gamepadId(index) end

---@param gamepad_id GamepadId
---@return string
function GamepadState:gamepadName(gamepad_id) end

---@param axis GamepadAxis
---@return number
function GamepadState:value(axis) end

---@param button GamepadButton
---@return boolean
function GamepadState:isPressed(button) end

---@param button GamepadButton
---@return boolean
function GamepadState:isDown(button) end

---@param button GamepadButton
---@return boolean
function GamepadState:isReleased(button) end

---@param gamepad_id GamepadId
---@param axis GamepadAxis
---@return number
function GamepadState:valueById(gamepad_id, axis) end

---@param gamepad_id GamepadId
---@param button GamepadButton
---@return boolean
function GamepadState:isPressedById(gamepad_id, button) end

---@param gamepad_id GamepadId
---@param button GamepadButton
---@return boolean
function GamepadState:isDownById(gamepad_id, button) end

---@param gamepad_id GamepadId
---@param button GamepadButton
---@return boolean
function GamepadState:isReleasedById(gamepad_id, button) end

