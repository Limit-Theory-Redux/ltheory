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

