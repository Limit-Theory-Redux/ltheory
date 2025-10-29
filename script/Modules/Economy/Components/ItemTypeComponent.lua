local Component = require("Core.ECS.Component")

--* is type a good naming here or nah?

---@class ItemTypeComponent: Component
---@overload fun(self: ItemTypeComponent, type: integer): ItemTypeComponent subclass internal
---@overload fun(type: integer): ItemTypeComponent subclass external
local ItemTypeComponent = Subclass("ItemTypeComponent", Component, function(self, type)
    self:setComponentName("EconomyItemType")

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
