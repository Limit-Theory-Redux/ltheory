local Component = require("Core.ECS.Component")

---@class HeightComponent: Component
---@overload fun(self: HeightComponent): HeightComponent subclass internal
---@overload fun(): HeightComponent subclass external
local HeightComponent = Subclass("HeightComponent", Component, function(self)
    self:setComponentName("CelestialHeightComponent")
end)

return HeightComponent
