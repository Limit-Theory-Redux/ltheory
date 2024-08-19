local Component = require('_ECS_WIP_TEMP.Components.Component') --!temp path

---@class NameComponent: Component
---@overload fun(self: NameComponent, name: string): NameComponent subclass internal
---@overload fun(name: string): NameComponent subclass external
local NameComponent = Subclass(Component, function(self, name)
    self:setComponentName("EntityName")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.NameComponent)

    self:setName(name)
end)

---@param name string
function NameComponent:setName(name)
    self.name = name or "Undefined"
end

---@return string
function NameComponent:getName()
    return self.name
end

return NameComponent
