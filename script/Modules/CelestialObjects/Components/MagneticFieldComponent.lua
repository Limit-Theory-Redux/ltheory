local Component = require("Core.ECS.Component")

---@class MagneticFieldComponent: Component
---@overload fun(self: MagneticFieldComponent): MagneticFieldComponent subclass internal
---@overload fun(): MagneticFieldComponent subclass external
local MagneticFieldComponent = Subclass("MagneticFieldComponent", Component, function(self)
    self:setComponentName("CelestialMagneticFieldComponent")
end)

return MagneticFieldComponent
