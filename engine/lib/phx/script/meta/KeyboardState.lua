---@meta

---@class KeyboardState
KeyboardState = {}

---@param button KeyboardButton
---@return boolean
function KeyboardState:isPressed(button) end

---@param button KeyboardButton
---@return boolean
function KeyboardState:isDown(button) end

---@param button KeyboardButton
---@return boolean
function KeyboardState:isReleased(button) end

---@return boolean
function KeyboardState:altPressed() end

---@return boolean
function KeyboardState:ctrlPressed() end

---@return boolean
function KeyboardState:shiftPressed() end

---@param button KeyboardButton
---@return number
function KeyboardState:value(button) end

-- Text entered in the current frame. Usually a single symbol.
---@return string
function KeyboardState:text() end

