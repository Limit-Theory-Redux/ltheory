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
end

function InventoryComponent:getInventory()
    return self.items
end

function InventoryComponent:put(itemEntityInfo)
    if self.items[itemEntityInfo.archetype] then
        self.items[itemEntityInfo.archetype][itemEntityInfo.id] = itemEntityInfo
        return true
    elseif not self.items[itemEntityInfo.archetype] then
        self.items[itemEntityInfo.archetype] = {}
        self.items[itemEntityInfo.archetype][itemEntityInfo.id] = itemEntityInfo
        return true
    end
    return false
end

function InventoryComponent:take(itemEntityInfo)
    if self.items[itemEntityInfo.archetype] and self.items[itemEntityInfo.archetype][itemEntityInfo.id] then
        return self.items[itemEntityInfo.archetype][itemEntityInfo.id]
    end
    return false
end

return InventoryComponent
