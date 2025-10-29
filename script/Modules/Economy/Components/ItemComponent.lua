local Component = require("Core.ECS.Component")

---@class ItemComponent: Component
---@overload fun(self: ItemComponent, type: integer): ItemComponent subclass internal
---@overload fun(type: integer): ItemComponent subclass external
local ItemComponent = Subclass("ItemComponent", Component, function(self, type)
    self:setComponentName("EconomyItem")

    self:setItem(type)
end)

---@param id integer<ItemId>
function ItemComponent:setItem(id)
    self.id = id
end

---@return integer<ItemId> id
function ItemComponent:getItem()
    return self.id
end

return ItemComponent
