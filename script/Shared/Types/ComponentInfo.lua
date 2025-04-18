--- Getting a component: `Registry:getComponent(componentInfo)`
---@class ComponentInfo
---@field archetype any
---@field entity EntityId
---@overload fun(args: {id: integer, archetype: any, entity: EntityId}): ComponentInfo
local ComponentInfo = Class("ComponentInfo", function(self, args)
    self.archetype = args.archetype
    self.entity = args.entity
end)

return ComponentInfo
