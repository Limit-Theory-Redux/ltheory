-- Systems
local Registry = require("Systems.Storage.Registry")

-- Components
local QuantityComponent = require("Components.Economy.QuantityComponent")

-- Utilities
local QuickProfiler = require("Shared.Tools.QuickProfiler")

---@class InventorySystem
---@overload fun(self: InventorySystem): InventorySystem class internal
---@overload fun(): InventorySystem class external
local InventorySystem = Class("InventorySystem", function(self)
    ---@diagnostic disable-next-line: invisible
    self:registerVars()
end)

---@private
function InventorySystem:registerVars()
    ---@private
    self.profiler = QuickProfiler("InventorySystem", false, false)
end

---@param inventory InventoryComponent
---@param itemId integer
---@param quantity number
---@return table<EntityInfo>|nil
function InventorySystem:take(inventory, itemId, quantity)
    self.profiler:start()

    local itemsOfType = inventory:getItemsOfType(itemId)
    local takenItems = {}
    local remainingQuantity = quantity

    for id, itemEntityInfo in pairs(itemsOfType) do
        local itemEntity = Registry:getEntity(itemEntityInfo)
        ---@cast itemEntity ItemEntity
        local quantityComponent = itemEntity:findComponentByArchetype(QuantityComponent)
        ---@cast quantityComponent QuantityComponent
        local itemQuantity = quantityComponent:getQuantity()

        if itemQuantity <= remainingQuantity then
            -- Take entire item
            inventory:removeItem(itemId, id)
            table.insert(takenItems, itemEntityInfo)
            remainingQuantity = remainingQuantity - itemQuantity
        else
            -- Split the item and update quantity
            quantityComponent:setQuantity(itemQuantity - remainingQuantity)
            local clone, cloneEntityInfo = itemEntity:clone()
            local cloneQuantityCmp = clone:findComponentByArchetype((QuantityComponent))
            cloneQuantityCmp:setQuantity(remainingQuantity)
            table.insert(takenItems, cloneEntityInfo)
            remainingQuantity = 0
        end

        if remainingQuantity <= 0 then
            break
        end
    end
    self.profiler:stop()
    return remainingQuantity == 0 and takenItems or nil
end

---@param inventory InventoryComponent
---@param itemId integer
---@param items table<EntityInfo>
function InventorySystem:put(inventory, itemId, items)
    for _, itemEntityInfo in ipairs(items) do
        inventory:addItem(itemId, itemEntityInfo)
    end
end

---@param item ItemEntity
---@param owner Player
---@param amount integer
---@return boolean success
function InventorySystem:lockItemQuantity(item, owner, amount)
    local quantityComponent = item:findComponentByArchetype(QuantityComponent)
    ---@cast quantityComponent QuantityComponent

    if amount > quantityComponent:getQuantity() then
        Log.Warn("Trying to reserve more than available quantity")
        return false
    end

    self.lockedQuantity = self.lockedQuantity or {}
    self.lockedQuantity[owner] = (self.lockedQuantity[owner] or 0) + amount
    quantityComponent:setLockedQuantity(owner, amount)
    return true
end

---@param item ItemEntity
---@param owner Player
---@param amount integer|nil
---@return boolean success
function InventorySystem:unlockItemQuantity(item, owner, amount)
    local quantityComponent = item:findComponentByArchetype(QuantityComponent)
    ---@cast quantityComponent QuantityComponent

    if not quantityComponent:getLockedQuantity() then
        Log.Warn("Trying to unlock quantity without locking it first")
        return false
    end

    if not quantityComponent:getLockedQuantity(owner) then
        Log.Warn("No locked quantity for this owner")
        return false
    end

    quantityComponent:unlockQuantity(owner, amount)
    return true
end

return InventorySystem()
