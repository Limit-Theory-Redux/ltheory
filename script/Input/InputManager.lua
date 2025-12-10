---@class InputManager
local InputManager = Class("InputManager", function(self) end)

InputManager.applicationContextGroups = {}
InputManager.gameContextGroups = {}

function InputManager:registerApplicationContextGroup(name, contextGroup)
    self.applicationContextGroups[name] = contextGroup
end

function InputManager:registerGameContextGroup(name, contextGroup)
    self.gameContextGroups[name] = contextGroup
end