--todo: render white screen
-- Systems
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.GlobalStorage") --!temp path

-- Utilities
local QuickProfiler = require("_ECS_WIP_TEMP.Shared.Tools.QuickProfiler")

---@class GameViewSystem
---@overload fun(self: GameViewSystem) class internal
---@overload fun() class external
local GameViewSystem = Class(function(self)
    ---@diagnostic disable-next-line: invisible
    self:registerVars()
    ---@diagnostic disable-next-line: invisible
    self:registerEvents()
end)

---@private
function GameViewSystem:registerVars()
    self.profiler = QuickProfiler("GameViewSystem", false, false)
end

---@private
function GameViewSystem:registerEvents()
end

return GameViewSystem()
