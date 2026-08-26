local Component = require('Core.ECS.Component')

-- This component provides a way to reference a Legacy.Entity that an Entity is associated with.
---@class LegacyEntityComponent: Component
---@field entity any
---@overload fun(self: LegacyEntityComponent, name: string) : LegacyEntityComponent subclass internal
---@overload fun(name: string): LegacyEntityComponent subclass external
local LegacyEntityComponent = Subclass("LegacyEntityComponent", Component, function(self, entity)
    self:setComponentName("LegacyEntityComponent")
    self.entity = entity
end)

return LegacyEntityComponent
