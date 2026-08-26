--- CursorManager — centralized cursor visibility and grab mode control.
--- All cursor state changes should go through this manager to prevent conflicts
--- between camera controllers, UI, maps, and game states.
---@class CursorManager
local CursorManager = {}

local currentVisible = true
local currentGrabMode = CursorGrabMode.None

--- Set cursor visibility and grab mode in one call
---@param visible boolean
---@param grabMode integer CursorGrabMode value
function CursorManager:set(visible, grabMode)
    if visible ~= currentVisible then
        Input:setCursorVisible(visible)
        currentVisible = visible
    end
    if grabMode ~= currentGrabMode then
        GameState.render.gameWindow:cursor():setGrabMode(grabMode)
        currentGrabMode = grabMode
    end
end

--- Convenience: free cursor (visible, no grab)
function CursorManager:free()
    self:set(true, CursorGrabMode.None)
end

--- Convenience: confined cursor (visible, confined to window)
function CursorManager:confined()
    self:set(true, CursorGrabMode.Confined)
end

--- Convenience: locked cursor (hidden, locked for FPS-style input)
function CursorManager:locked()
    self:set(false, CursorGrabMode.Locked)
end

---@return boolean
function CursorManager:isVisible()
    return currentVisible
end

---@return integer
function CursorManager:getGrabMode()
    return currentGrabMode
end

return CursorManager
