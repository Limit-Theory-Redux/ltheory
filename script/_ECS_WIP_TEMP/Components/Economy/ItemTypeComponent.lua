local Component = require('_ECS_WIP_TEMP.Components.Component')

---@class ItemTypeComponent: Component
---@overload fun(self: ItemTypeComponent, type: integer): ItemTypeComponent subclass internal
---@overload fun(type: integer): ItemTypeComponent subclass external
local ItemTypeComponent = Subclass(Component, function(self, type)
    self:setComponentName("EconomyItemType")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.ItemTypeComponent)

    self:setItemType(type)
end)

---@param type integer
function ItemTypeComponent:setItemType(type)
    self.type = type
end

---@return integer type
function ItemTypeComponent:getItemType()
    return self.type
end

return ItemTypeComponent
