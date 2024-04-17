---@meta

Cursor = Cursor

---@return CursorIcon
function Cursor:icon(self) end

---@param icon CursorIcon
function Cursor:setIcon(self, icon) end

---@return boolean
function Cursor:isVisible(self) end

---@param visible boolean
function Cursor:setVisible(self, visible) end

---@return CursorGrabMode
function Cursor:grabMode(self) end

---@param grab_mode CursorGrabMode
function Cursor:setGrabMode(self, grab_mode) end

---@return boolean
function Cursor:isHitTest(self) end

---@param hit_test boolean
function Cursor:setHitTest(self, hit_test) end

