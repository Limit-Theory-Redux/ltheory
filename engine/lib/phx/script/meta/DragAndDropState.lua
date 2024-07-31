---@meta

---@class DragAndDropState
DragAndDropState = {}

---@return string|nil
function DragAndDropState:getDroppedFile() end

---@return string|nil
function DragAndDropState:getHoveredFile() end

---@return boolean
function DragAndDropState:ifHoveredFileCancelled() end

