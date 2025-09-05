local Component = require('Components.Component')

--- A component that stores a reference to the entity object that this entity ID was constructed from.
---@class EntityComponent: Component
---@field entity Entity
---@overload fun(self: EntityComponent) : EntityComponent subclass internal
---@overload fun(): EntityComponent subclass external
local EntityComponent = Subclass("EntityComponent", Component, function(self, entity)
    self:setComponentName("Entity")
    self.entity = entity
end)

return EntityComponent
