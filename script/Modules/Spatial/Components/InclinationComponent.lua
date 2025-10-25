local Component = require("Core.ECS.Component")

---@class InclinationComponent: Component
---@overload fun(self: InclinationComponent): InclinationComponent subclass internal
---@overload fun(): InclinationComponent subclass external
local InclinationComponent = Subclass("InclinationComponent", Component, function(self)
    self:setComponentName("CelestialInclinationComponent")
end)

return InclinationComponent
