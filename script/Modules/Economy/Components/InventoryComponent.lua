local Component = require("Core.ECS.Component")
local Item = require("Modules.Economy.Components.ItemComponent")

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
    local itemId = itemEntity:get(Item):getItem()

    if not self.items[itemId] then
        self.items[itemId] = {}
    end
    self.items[itemId][itemEntity.id] = itemEntity
end

---@param itemEntity Entity
function InventoryComponent:removeItem(itemEntity)
    local itemId = itemEntity:get(Item):getItem()

    if self.items[itemId] and self.items[itemId][itemEntity.id] then
        local removed = self.items[itemId][itemEntity.id]
        self.items[itemId][itemEntity.id] = nil
        if next(self.items[itemId]) == nil then
            self.items[itemId] = nil
        end
        return removed
    end
    return nil
end

---@param id integer<ItemId>
function InventoryComponent:getItems(id)
    return self.items[id] or {}
end

return InventoryComponent
