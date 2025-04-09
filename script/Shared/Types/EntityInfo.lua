--- Getting an entity: `Registry:getEntity(entityInfo)`
---@class EntityInfo
---@field id integer
---@field archetype EntityArchetype
---@overload fun(args: {id: integer, archetype: EntityArchetype}): EntityInfo
local EntityInfo = Class("EntityInfo", function(self, args)
    self.id = args.id
    self.archetype = args.archetype
end)

return EntityInfo
