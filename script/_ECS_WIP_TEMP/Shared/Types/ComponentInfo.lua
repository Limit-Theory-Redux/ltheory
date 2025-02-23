--- Getting a component: `Registry:getComponent(componentInfo)`
---@class ComponentInfo
---@field id integer
---@field archetype ComponentArchetype
---@field entity EntityInfo
---@overload fun(args: {id: integer, archetype: ComponentArchetype, entity: EntityInfo}): ComponentInfo
local ComponentInfo = Class("ComponentInfo", function(self, args)
    self.id = args.id
    self.archetype = args.archetype
    self.entity = args.entity
end)

return ComponentInfo
