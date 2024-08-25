--todo: render white screen
-- Systems
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.Storage.GlobalStorage") --!temp path
local RenderState = require("_ECS_WIP_TEMP.Shared.Rendering.RenderState")

-- Utilities
local QuickProfiler = require("_ECS_WIP_TEMP.Shared.Tools.QuickProfiler") --!temp path

---@class GameViewSystem

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
    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
    EventBus:subscribe(Event.Render, self, self.onRender)
    EventBus:subscribe(Event.PostRender, self, self.onPostRender)
end


function GameViewSystem:onPreRender(data)

end

function GameViewSystem:onRender(data)
    --[[
        
    ]]--
    RenderState.cameraEye = self.activeCamera.transform:getPosition()
    

end

function GameViewSystem:onPostRender(data)
    --[[
        
    ]]--
end

return GameViewSystem()
