---@meta

GamepadState = GamepadState

---@return integer
function GamepadState.gamepadsCount(self) end

---@param index integer
---@return GamepadId
function GamepadState.gamepadId(self, index) end

---@param gamepad_id GamepadId
---@return string
function GamepadState.gamepadName(self, gamepad_id) end

---@param axis GamepadAxis
---@return number
function GamepadState.value(self, axis) end

---@param button GamepadButton
---@return boolean
function GamepadState.isPressed(self, button) end

---@param button GamepadButton
---@return boolean
function GamepadState.isDown(self, button) end

---@param button GamepadButton
---@return boolean
function GamepadState.isReleased(self, button) end

---@param gamepad_id GamepadId
---@param axis GamepadAxis
---@return number
function GamepadState.valueById(self, gamepad_id, axis) end

---@param gamepad_id GamepadId
---@param button GamepadButton
---@return boolean
function GamepadState.isPressedById(self, gamepad_id, button) end

---@param gamepad_id GamepadId
---@param button GamepadButton
---@return boolean
function GamepadState.isDownById(self, gamepad_id, button) end

---@param gamepad_id GamepadId
---@param button GamepadButton
---@return boolean
function GamepadState.isReleasedById(self, gamepad_id, button) end

