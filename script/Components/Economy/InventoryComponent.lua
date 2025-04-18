local Component = require('Components.Component')

---@class InventoryComponent: Component
---@field items table<integer, table<EntityId, EntityId>>
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

---@param itemId integer
---@param itemEntityId EntityId
function InventoryComponent:addItem(itemId, itemEntityId)
    if not self.items[itemId] then
        self.items[itemId] = {}
    end
    self.items[itemId][itemEntityId] = itemEntityId
end

---@param itemId integer
---@param id Guid
function InventoryComponent:removeItem(itemId, id)
    if self.items[itemId] and self.items[itemId][id] then
        local removed = self.items[itemId][id]
        self.items[itemId][id] = nil
        if next(self.items[itemId]) == nil then
            self.items[itemId] = nil
        end
        return removed
    end
    return nil
end

function InventoryComponent:getItemsOfType(type)
    return self.items[type] or {}
end

return InventoryComponent
