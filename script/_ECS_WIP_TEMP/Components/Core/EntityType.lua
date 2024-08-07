local Component = require('_ECS_WIP_TEMP.Components.Component') --!temp path

---@class TypeComponent: Component
---@overload fun(type: EntityType): TypeComponent subclass external
local TypeComponent = Subclass(Component, function(self, type)
    ---@cast self TypeComponent
    self:setComponentName("Type")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.TypeComponent)

    self:setType(type)
end)

---@param entityType EntityType
function TypeComponent:setType(entityType)
    if not entityType or type(entityType) ~= "number" then
        Log.Warn("Did not provide a valid object type for type component")
        return
    end

    self.type = entityType
end

---@return EntityType
function TypeComponent:getType()
    return self.type
end

---@return string
function TypeComponent:getTypeName()
    return Enums.EntityTypeNames[self.type]
end

return TypeComponent
