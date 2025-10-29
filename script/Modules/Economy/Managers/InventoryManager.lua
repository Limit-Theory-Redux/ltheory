local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local Economy = require("Modules.Economy.Components")
local Items = require("Shared.Registries.Items")
local EconomyEntities = require("Modules.Economy.Entities")
local EconomyComponents = require("Modules.Economy.Components")
local Physics = require("Modules.Physics.Components")
local Entity = require("Core.ECS.Entity")

---@class InventoryManager
---@overload fun(self: InventoryManager): InventoryManager class internal
---@overload fun(): InventoryManager class external
local InventoryManager = Class("InventoryManager", function(self)
    ---@diagnostic disable-next-line: invisible
    self:registerVars()
end)

---@private
function InventoryManager:registerVars()
    ---@private
    self.profiler = QuickProfiler("InventoryManager", false, false)
end

---@param inventory InventoryComponent
---@param itemId integer
---@param quantity number
---@return table<Entity>|nil
function InventoryManager:take(inventory, itemId, quantity)
    self.profiler:start()

    local itemsOfType = inventory:getItemsOfType(itemId)
    local takenItems = {}
    local remainingQuantity = quantity

    for id, itemEntity in pairs(itemsOfType) do
        local quantityComponent = itemEntity:get(Economy.Quantity)

        if not quantityComponent then
            Log.Warn(string.format("Item entity %s does not have a Quantity component", itemEntity))
            return nil
        end

        local itemQuantity = quantityComponent:getQuantity()

        -- Not enough stuff?
        if itemQuantity <= remainingQuantity then
            -- Log.Debug("Not enough stuff. Taking all items.")
            -- Take entire item
            inventory:removeItem(itemEntity)
            table.insert(takenItems, itemEntity)
            remainingQuantity = remainingQuantity - itemQuantity
        else
            -- Split the item and update quantity
            -- Log.Debug("Updating seller item quantity to: ", itemQuantity - remainingQuantity)
            -- Log.Debug(string.format("Seller item entity: %s", itemEntity))
            -- Log.Debug(string.format("Seller item quantity component: %s", quantityComponent:getGuid()))
            quantityComponent:setQuantity(itemQuantity - remainingQuantity)
            local physicsComponentMass = itemEntity:get(Physics.Mass):getMass()

            local clonedEntity = Registry:cloneEntity(itemEntity)
            local clonedQuantityCmp = clonedEntity:get(Economy.Quantity)

            if not clonedQuantityCmp then
                Log.Warn(string.format("Cloned item entity %s does not have a Quantity component", clonedEntity))
                return nil
            end

            Log.Debug("Updating cloned item quantity to: ", remainingQuantity);
            Log.Debug(string.format("Clone item entity: %s", clonedEntity))
            Log.Debug(string.format("Clone item quantity component: %s", clonedQuantityCmp:getGuid()))
            clonedQuantityCmp:setQuantity(remainingQuantity)
            table.insert(takenItems, clonedEntity)
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
---@param items table<Entity>
function InventoryManager:put(inventory, items)
    for _, itemEntity in ipairs(items) do
        inventory:addItem(itemEntity)
    end
end

---@param item Entity
---@param owner Entity
---@param amount integer
---@return boolean success
function InventoryManager:lockItemQuantity(item, owner, amount)
    local quantityComponent = item:get(Economy.Quantity)

    if amount > quantityComponent:getQuantity() then
        Log.Warn("Trying to reserve more than available quantity")
        return false
    end

    self.lockedQuantity = self.lockedQuantity or {}
    self.lockedQuantity[owner] = (self.lockedQuantity[owner] or 0) + amount
    quantityComponent:setLockedQuantity(owner, amount)
    return true
end

---@param item Entity
---@param owner Entity
---@param amount integer|nil
---@return boolean success
function InventoryManager:unlockItemQuantity(item, owner, amount)
    local quantityComponent = item:get(Economy.Quantity)

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

return InventoryManager()
