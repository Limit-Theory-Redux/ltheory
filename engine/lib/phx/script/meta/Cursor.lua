---@meta

---@class Cursor
Cursor = {}

---@return CursorIcon
function Cursor:icon() end

---@param icon CursorIcon
function Cursor:setIcon(icon) end

---@return boolean
function Cursor:isVisible() end

---@param visible boolean
function Cursor:setVisible(visible) end

---@return CursorGrabMode
function Cursor:grabMode() end

---@param grabMode CursorGrabMode
function Cursor:setGrabMode(grabMode) end

---@return boolean
function Cursor:isHitTest() end

---@param hitTest boolean
function Cursor:setHitTest(hitTest) end

