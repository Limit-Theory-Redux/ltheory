local Component = require("Core.ECS.Component")
local Item = require("Modules.Economy.Components.ItemComponent")

---Item stacks held by an entity: item entities (`ItemComponent` + `QuantityComponent`), by item id.
---A capacity limits the total mass of the stacks; an inventory without one holds any amount. The
---stacks are destroyed with the inventory. Change the contents through `InventoryManager`, which
---keeps one stack per item and the capacity.
---@class InventoryComponent: Component
---@field items table<integer, table<integer, Entity>> Item id -> entity id -> item entity
---@field capacity number|nil Most total mass held; nil for no limit
---@overload fun(self: InventoryComponent, capacity: number|nil): InventoryComponent subclass internal
---@overload fun(capacity: number|nil): InventoryComponent subclass external
local InventoryComponent = Subclass("InventoryComponent", Component, function(self, capacity)
    self:setComponentName("EconomyInventory")

    self:init(capacity)
end)

---@param capacity number|nil
function InventoryComponent:init(capacity)
    self.items = {}
    self.capacity = capacity
    SetLengthMetamethod(self.items)
end

function InventoryComponent:getInventory()
    return self.items
end

---@return number|nil capacity
function InventoryComponent:getCapacity()
    return self.capacity
end

---@param capacity number|nil
function InventoryComponent:setCapacity(capacity)
    self.capacity = capacity
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

---Destroys the stacks with the entity that held them.
function InventoryComponent:onDestroy()
    local Registry = require("Core.ECS.Registry")
    for _, stacks in pairs(self.items) do
        for _, itemEntity in pairs(stacks) do
            if Registry:hasEntity(itemEntity) then Registry:destroyEntity(itemEntity) end
        end
    end
    self.items = {}
end

return InventoryComponent
