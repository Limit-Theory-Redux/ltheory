local Component = require("Core.ECS.Component")

---@class CompositionComponent: Component
---@overload fun(self: CompositionComponent): CompositionComponent subclass internal
---@overload fun(): CompositionComponent subclass external
local CompositionComponent = Subclass("CompositionComponent", Component, function(self)
    self:setComponentName("CelestialCompositionComponent")
end)

return CompositionComponent
