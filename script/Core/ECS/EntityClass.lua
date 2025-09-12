---@class Entity A non-owning handle to an entity in the ECS.
---@overload fun(self: Entity, id: EntityId): Entity class internal
---@overload fun(id: EntityId): Entity class external
return Class("Entity", function(self, id)
    self.id = id
end)
