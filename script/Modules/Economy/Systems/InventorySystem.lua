local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local Economy = require("Modules.Economy.Components")

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
---@return table<EntityId>|nil
function InventorySystem:take(inventory, itemId, quantity)
    self.profiler:start()

    local itemsOfType = inventory:getItemsOfType(itemId)
    local takenItems = {}
    local remainingQuantity = quantity

    for id, itemEntityId in pairs(itemsOfType) do
        local itemEntity = Registry:getEntity(itemEntityId)
        ---@cast itemEntity ItemEntity
        local quantityComponent = itemEntity:getComponent(Economy.Quantity)
        ---@cast quantityComponent QuantityComponent
        local itemQuantity = quantityComponent:getQuantity()

        if itemQuantity <= remainingQuantity then
            -- Take entire item
            inventory:removeItem(itemId, id)
            table.insert(takenItems, itemEntityId)
            remainingQuantity = remainingQuantity - itemQuantity
        else
            -- Split the item and update quantity
            quantityComponent:setQuantity(itemQuantity - remainingQuantity)
            local cloneEntityId = Registry:cloneEntity(itemEntityId)
            local cloneQuantityCmp = Registry:get(cloneEntityId, Economy.Quantity)
            cloneQuantityCmp:setQuantity(remainingQuantity)
            table.insert(takenItems, cloneEntityId)
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
---@param items table<EntityId>
function InventorySystem:put(inventory, itemId, items)
    for _, itemEntityId in ipairs(items) do
        inventory:addItem(itemId, itemEntityId)
    end
end

---@param item ItemEntity
---@param owner PlayerEntity
---@param amount integer
---@return boolean success
function InventorySystem:lockItemQuantity(item, owner, amount)
    local quantityComponent = item:getComponent(Economy.Quantity)

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
---@param owner PlayerEntity
---@param amount integer|nil
---@return boolean success
function InventorySystem:unlockItemQuantity(item, owner, amount)
    local quantityComponent = item:getComponent(Economy.Quantity)

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
