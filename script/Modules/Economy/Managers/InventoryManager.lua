local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local Economy = require("Modules.Economy.Components")
local Items = require("Shared.Registries.Items")
local EconomyEntities = require("Modules.Economy.Entities")

---Changes inventories: counts, adds and removes quantities of an item, keeping one stack entity per
---item and the inventory's mass capacity.
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

---@param itemId integer
---@return number mass The mass of one unit
local function unitMass(itemId)
    local definition = Items:getDefinition(itemId)
    return definition and definition.mass or 0
end

---@param inventory InventoryComponent
---@param itemId integer
---@return integer quantity
function InventoryManager:count(inventory, itemId)
    local total = 0
    for _, itemEntity in pairs(inventory:getItems(itemId)) do
        total = total + itemEntity:get(Economy.Quantity):getQuantity()
    end
    return total
end

---@param inventory InventoryComponent
---@return number mass Of every stack together
function InventoryManager:getMass(inventory)
    local mass = 0
    for itemId in pairs(inventory:getInventory()) do
        mass = mass + self:count(inventory, itemId) * unitMass(itemId)
    end
    return mass
end

---@param inventory InventoryComponent
---@return number freeMass `math.huge` for an inventory without a capacity
function InventoryManager:getFreeMass(inventory)
    local capacity = inventory:getCapacity()
    if not capacity then return math.huge end
    return math.max(0, capacity - self:getMass(inventory))
end

---How many units of an item fit in the inventory's free capacity.
---@param inventory InventoryComponent
---@param itemId integer
---@return integer|number quantity `math.huge` when the inventory has no capacity or the item no mass
function InventoryManager:getRoomFor(inventory, itemId)
    local mass = unitMass(itemId)
    local free = self:getFreeMass(inventory)
    if mass <= 0 or free == math.huge then return math.huge end
    return math.floor(free / mass)
end

---Adds units of an item to its stack, making the stack when there is none. Fails, adding nothing,
---when they don't fit.
---@param inventory InventoryComponent
---@param itemId integer
---@param quantity integer
---@return boolean added
function InventoryManager:add(inventory, itemId, quantity)
    if quantity <= 0 then return quantity == 0 end
    if self:getRoomFor(inventory, itemId) < quantity then return false end

    local _, stack = next(inventory:getItems(itemId))
    if stack then
        local quantityComponent = stack:get(Economy.Quantity)
        quantityComponent:setQuantity(quantityComponent:getQuantity() + quantity)
    else
        inventory:addItem(EconomyEntities.Item(Items:getDefinition(itemId), quantity))
    end
    return true
end

---Removes units of an item. Fails, removing nothing, when the inventory holds fewer.
---@param inventory InventoryComponent
---@param itemId integer
---@param quantity integer
---@return boolean removed
function InventoryManager:remove(inventory, itemId, quantity)
    if quantity <= 0 then return quantity == 0 end
    if self:count(inventory, itemId) < quantity then return false end

    local remaining = quantity
    for _, itemEntity in pairs(inventory:getItems(itemId)) do
        local quantityComponent = itemEntity:get(Economy.Quantity)
        local taken = math.min(remaining, quantityComponent:getQuantity())
        quantityComponent:setQuantity(quantityComponent:getQuantity() - taken)
        remaining = remaining - taken
        if remaining <= 0 then break end
    end
    -- Emptied stacks are dropped after the walk, so the walk never sees its table change.
    for _, itemEntity in pairs(inventory:getItems(itemId)) do
        if itemEntity:get(Economy.Quantity):getQuantity() <= 0 then
            inventory:removeItem(itemEntity)
            Registry:destroyEntity(itemEntity)
        end
    end
    return true
end

---Takes units of an item out of the inventory as item entities, splitting a stack when needed.
---Fails, taking nothing, when the inventory holds fewer.
---@param inventory InventoryComponent
---@param itemId integer
---@param quantity number
---@return table<Entity>|nil
function InventoryManager:take(inventory, itemId, quantity)
    if self:count(inventory, itemId) < quantity then return nil end
    self.profiler:start()

    local takenItems = {}
    local remainingQuantity = quantity
    local stacks = {}
    for _, itemEntity in pairs(inventory:getItems(itemId)) do stacks[#stacks + 1] = itemEntity end

    for _, itemEntity in ipairs(stacks) do
        local quantityComponent = itemEntity:get(Economy.Quantity)
        local itemQuantity = quantityComponent:getQuantity()

        if itemQuantity <= remainingQuantity then
            inventory:removeItem(itemEntity)
            table.insert(takenItems, itemEntity)
            remainingQuantity = remainingQuantity - itemQuantity
        else
            quantityComponent:setQuantity(itemQuantity - remainingQuantity)
            local clonedEntity = Registry:cloneEntity(itemEntity)
            clonedEntity:get(Economy.Quantity):setQuantity(remainingQuantity)
            table.insert(takenItems, clonedEntity)
            remainingQuantity = 0
        end

        if remainingQuantity <= 0 then
            break
        end
    end
    self.profiler:stop()
    return takenItems
end

---Puts item entities into the inventory, merging each into the stack of its item.
---@param inventory InventoryComponent
---@param items table<Entity>
function InventoryManager:put(inventory, items)
    for _, itemEntity in ipairs(items) do
        local itemId = itemEntity:get(Economy.Item):getItem()
        local _, stack = next(inventory:getItems(itemId))
        if stack and stack ~= itemEntity then
            local quantityComponent = stack:get(Economy.Quantity)
            quantityComponent:setQuantity(quantityComponent:getQuantity() + itemEntity:get(Economy.Quantity):getQuantity())
            Registry:destroyEntity(itemEntity)
        else
            inventory:addItem(itemEntity)
        end
    end
end

---@param item Entity
---@param owner Entity
---@param amount integer
---@return boolean success
function InventoryManager:lockItemQuantity(item, owner, amount)
    local quantityComponent = item:get(Economy.Quantity)

    if amount > quantityComponent:getAvailableQuantity() then
        Log.Warn("Trying to reserve more than available quantity")
        return false
    end

    quantityComponent:setLockedQuantity(owner, amount)
    return true
end

---@param item Entity
---@param owner Entity
---@param amount integer|nil
---@return boolean success
function InventoryManager:unlockItemQuantity(item, owner, amount)
    local quantityComponent = item:get(Economy.Quantity)

    if quantityComponent:getLockedQuantity(owner) <= 0 then
        Log.Warn("No locked quantity for this owner")
        return false
    end

    quantityComponent:unlockQuantity(owner, amount)
    return true
end

return InventoryManager()
