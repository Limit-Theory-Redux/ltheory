local Component = require("Core.ECS.Component")

---@class ZoneComponent: Component
---@overload fun(self: ZoneComponent): ZoneComponent subclass internal
---@overload fun(): ZoneComponent subclass external
--- Marks an entity as a zone, a region of space. It holds no data: systems find zones by viewing it.
local ZoneComponent = Subclass("ZoneComponent", Component, function(self)
    self:setComponentName("Zone")
end)

return ZoneComponent
