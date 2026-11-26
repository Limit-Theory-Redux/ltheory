local Component = require("Core.ECS.Component")

---@class HoldOrderComponent: Component
---@overload fun(self: HoldOrderComponent): HoldOrderComponent subclass internal
---@overload fun(): HoldOrderComponent subclass external
--- A Hold order keeps the ship where it is when the order starts; it never ends by itself.
local HoldOrderComponent = Subclass("HoldOrderComponent", Component, function(self)
    self:setComponentName("HoldOrder")
end)

return HoldOrderComponent
