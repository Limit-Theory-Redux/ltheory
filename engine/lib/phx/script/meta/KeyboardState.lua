---@meta

KeyboardState = KeyboardState

---@param button KeyboardButton
---@return boolean
function KeyboardState:isPressed(self, button) end

---@param button KeyboardButton
---@return boolean
function KeyboardState:isDown(self, button) end

---@param button KeyboardButton
---@return boolean
function KeyboardState:isReleased(self, button) end

---@return boolean
function KeyboardState:altPressed(self) end

---@return boolean
function KeyboardState:ctrlPressed(self) end

---@return boolean
function KeyboardState:shiftPressed(self) end

---@param button KeyboardButton
---@return number
function KeyboardState:value(self, button) end

