local Component = require('_ECS_WIP_TEMP.Components.Component')

---@class InventoryComponent: Component
---@overload fun(self: InventoryComponent, playerId: integer|nil): InventoryComponent subclass internal
---@overload fun(playerId: integer|nil): InventoryComponent subclass external
local InventoryComponent = Subclass(Component, function(self)
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

function InventoryComponent:addItem(type, itemEntityInfo)
    if not self.items[type] then
        self.items[type] = {}
    end
    self.items[type][itemEntityInfo.id] = itemEntityInfo
end

function InventoryComponent:removeItem(type, id)
    if self.items[type] and self.items[type][id] then
        local removed = self.items[type][id]
        self.items[type][id] = nil
        if next(self.items[type]) == nil then
            self.items[type] = nil
        end
        return removed
    end
    return nil
end

function InventoryComponent:getItemsOfType(type)
    return self.items[type] or {}
end

return InventoryComponent
