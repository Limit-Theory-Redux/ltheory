local Component = require("Core.ECS.Component")

---@class LuminosityComponent: Component
---@overload fun(self: LuminosityComponent): LuminosityComponent subclass internal
---@overload fun(): LuminosityComponent subclass external
local LuminosityComponent = Subclass("LuminosityComponent", Component, function(self)
    self:setComponentName("CelestialLuminosityComponent")
end)

return LuminosityComponent
