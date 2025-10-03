local Component = require("Core.ECS.Component")
local ItemType = require("Modules.Economy.Components.ItemTypeComponent")

---@class InventoryComponent: Component
---@field items table<integer, table<Entity, Entity>>
---@overload fun(self: InventoryComponent, playerId: integer|nil): InventoryComponent subclass internal
---@overload fun(playerId: integer|nil): InventoryComponent subclass external
local InventoryComponent = Subclass("InventoryComponent", Component, function(self)
    self:setComponentName("EconomyInventory")

    self:init()
end)

function InventoryComponent:init()
    self.items = {}
    SetLengthMetamethod(self.items)
end

function InventoryComponent:getInventory()
    return self.items
end

---@param itemEntity Entity
function InventoryComponent:addItem(itemEntity)
    local itemType = itemEntity:get(ItemType):getItemType()

    if not self.items[itemType] then
        self.items[itemType] = {}
    end
    self.items[itemType][itemEntity.id] = itemEntity
end

---@param itemEntity Entity
function InventoryComponent:removeItem(itemEntity)
    local itemType = itemEntity:get(ItemType):getItemType()

    if self.items[itemType] and self.items[itemType][itemEntity.id] then
        local removed = self.items[itemType][itemEntity.id]
        self.items[itemType][itemEntity.id] = nil
        if next(self.items[itemType]) == nil then
            self.items[itemType] = nil
        end
        return removed
    end
    return nil
end

function InventoryComponent:getItemsOfType(type)
    return self.items[type] or {}
end

return InventoryComponent
