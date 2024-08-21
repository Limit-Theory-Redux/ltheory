---@meta

---@class DragAndDropState
DragAndDropState = {}

---@return string?
function DragAndDropState:getDroppedFile() end

---@return string?
function DragAndDropState:getHoveredFile() end

---@return boolean
function DragAndDropState:ifHoveredFileCancelled() end

