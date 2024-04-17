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

---@param grab_mode CursorGrabMode
function Cursor:setGrabMode(grab_mode) end

---@return boolean
function Cursor:isHitTest() end

---@param hit_test boolean
function Cursor:setHitTest(hit_test) end

