local Component = require('_ECS_WIP_TEMP.Components.Component')

---@class InventoryComponent: Component
---@overload fun(self: InventoryComponent, playerId: integer|nil): InventoryComponent subclass internal
---@overload fun(playerId: integer|nil): InventoryComponent subclass external
local InventoryComponent = Subclass("InventoryComponent", Component, function(self)
    self:setComponentName("EconomyInventory")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.InventoryComponent)

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
---@param itemEntityInfo EntityInfo
function InventoryComponent:addItem(itemId, itemEntityInfo)
    if not self.items[itemId] then
        self.items[itemId] = {}
    end
    self.items[itemId][itemEntityInfo.id] = itemEntityInfo
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
