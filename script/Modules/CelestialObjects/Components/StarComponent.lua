local Component = require("Core.ECS.Component")

---@class StarComponent: Component
---@overload fun(self: StarComponent): StarComponent subclass internal
---@overload fun(): StarComponent subclass external
--- Marks an entity as a star. It holds no data: systems find stars by viewing it.
local StarComponent = Subclass("StarComponent", Component, function(self)
    self:setComponentName("Star")
end)

return StarComponent
