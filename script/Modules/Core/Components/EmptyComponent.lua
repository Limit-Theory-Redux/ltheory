local Component = require("Core.ECS.Component")

---@class EmptyComponent: Component
---@overload fun(self: EmptyComponent): EmptyComponent subclass internal
---@overload fun(): EmptyComponent subclass external
local EmptyComponent = Subclass("EmptyComponent", Component, function(self)
    self:setComponentName("EmptyComponent")
end)

return EmptyComponent
