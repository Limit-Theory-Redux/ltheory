local Component = require("Core.ECS.Component")

---@class MoonComponent: Component
---@overload fun(self: MoonComponent): MoonComponent subclass internal
---@overload fun(): MoonComponent subclass external
--- Marks an entity as a moon. It holds no data: systems find moons by viewing it.
local MoonComponent = Subclass("MoonComponent", Component, function(self)
    self:setComponentName("Moon")
end)

return MoonComponent
