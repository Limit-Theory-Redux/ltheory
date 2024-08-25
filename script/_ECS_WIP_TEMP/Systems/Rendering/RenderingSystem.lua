--todo: render white screen
-- Systems
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.GlobalStorage") --!temp path

-- Utilities
local QuickProfiler = require("_ECS_WIP_TEMP.Shared.Tools.QuickProfiler") --!temp path

---@class RenderCoreSystem
---@overload fun(self: RenderCoreSystem) class internal
---@overload fun() class external
local RenderCoreSystem = Class(function(self)
    ---@diagnostic disable-next-line: invisible
    self:registerVars()
    ---@diagnostic disable-next-line: invisible
    self:registerEvents()
end)

---@private
function RenderCoreSystem:registerVars()
    self.profiler = QuickProfiler("RenderCoreSystem", false, false)
end

---@private
function RenderCoreSystem:registerEvents()
    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
    EventBus:subscribe(Event.Render, self, self.onRender)
    EventBus:subscribe(Event.PostRender, self, self.onPostRender)
end

function RenderCoreSystem:onPreRender(data)
    --[[

    ]]--
    -- LightList = AllEntitiesWithComponent(LightComponent)
end

function RenderCoreSystem:onRender(data)
    --[[
        
    ]]--
    -- Reset RenderState and ClipRect at Start of Render
    ClipRect.PushDisabled()
    RenderState.PushAllDefaults()

    -- Reset Camera Camera:draw

    -- < Opaque Pass > --


    -- < Lighting > --
        -- < Global Lighting > --
        -- Use Cached World Light Material ?
        -- < Local Lighting > --
        -- Use Cached Light Material
        -- < Aldebo & accumulated light buffer > --
        -- Use Cached Light Material 
    

    -- < Alpha (Additive Pass) > --


    -- < Alpha Pass > --

end

function RenderCoreSystem:onPostRender(data)
    --[[
        
    ]]--
end

return RenderCoreSystem()
