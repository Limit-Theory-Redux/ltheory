local Component = require("Core.ECS.Component")

---@class StarSystemComponent: Component
---@overload fun(self: StarSystemComponent): StarSystemComponent subclass internal
---@overload fun(): StarSystemComponent subclass external
--- Marks an entity as a star system. It holds no data: systems find star systems by viewing it.
local StarSystemComponent = Subclass("StarSystemComponent", Component, function(self)
    self:setComponentName("StarSystem")
end)

return StarSystemComponent
