local Registry = require("Core.ECS.Registry")
local DeltaTimer = require("Shared.Tools.DeltaTimer")

local Camera = require("Modules.Cameras.Components")

---@class CameraSystem
local CameraSystem = Class("CameraSystem", function(self)
    self:registerEvents()
end)

function CameraSystem:registerEvents()
    EventBus:subscribe(Event.Input, self, self.onInput)
    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
end

---@param e EventData
function CameraSystem:onInput(e)
    local dt = e:deltaTime()

    for entity, component in Registry:iterEntities(Camera.CameraData) do
        if not entity:isValid() or not component then
            goto continue
        end

        -- check if has controller
        if not component:hasController() then
            goto continue
        end

        local controller = component:getController()
        ---@cast controller CameraController
        -- update controller
        if controller:isEnabled() then
            controller:onInput(dt)
        end
        ::continue::
    end
end

---@param e EventData
function CameraSystem:onPreRender(e)
    local dt = e:deltaTime()

    for entity, component in Registry:iterEntities(Camera.CameraData) do
        if not entity:isValid() or not component then
            goto continue
        end

        -- check if has controller
        if not component:hasController() then
            goto continue
        end

        local controller = component:getController()
        ---@cast controller CameraController
        -- update controller
        if controller:isEnabled() then
            controller:onPreRender(dt)
        end
        ::continue::
    end
end

return CameraSystem()
