-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class GamepadState
GamepadState = {}

---@return integer
function GamepadState:gamepadsCount() end

---@param index integer
---@return GamepadId?
function GamepadState:gamepadId(index) end

---@param gamepadId GamepadId
---@return string?
function GamepadState:gamepadName(gamepadId) end

---@param axis GamepadAxis
---@return number
function GamepadState:value(axis) end

---@param axis GamepadAxis
---@return number
function GamepadState:delta(axis) end

-- Get the analog value of a button (useful for triggers that report pressure 0.0 to 1.0)
---@param button GamepadButton
---@return number
function GamepadState:valueAnalog(button) end

-- Get the delta (change since last frame) of a button's analog value
---@param button GamepadButton
---@return number
function GamepadState:deltaAnalog(button) end

---@param button GamepadButton
---@return boolean
function GamepadState:isPressed(button) end

---@param button GamepadButton
---@return boolean
function GamepadState:isDown(button) end

---@param button GamepadButton
---@return boolean
function GamepadState:isReleased(button) end

---@param gamepadId GamepadId
---@param axis GamepadAxis
---@return number
function GamepadState:valueById(gamepadId, axis) end

---@param gamepadId GamepadId
---@param axis GamepadAxis
---@return number
function GamepadState:deltaById(gamepadId, axis) end

---@param gamepadId GamepadId
---@param button GamepadButton
---@return number
function GamepadState:valueAnalogById(gamepadId, button) end

---@param gamepadId GamepadId
---@param button GamepadButton
---@return number
function GamepadState:deltaAnalogById(gamepadId, button) end

---@param gamepadId GamepadId
---@param button GamepadButton
---@return boolean
function GamepadState:isPressedById(gamepadId, button) end

---@param gamepadId GamepadId
---@param button GamepadButton
---@return boolean
function GamepadState:isDownById(gamepadId, button) end

---@param gamepadId GamepadId
---@param button GamepadButton
---@return boolean
function GamepadState:isReleasedById(gamepadId, button) end

