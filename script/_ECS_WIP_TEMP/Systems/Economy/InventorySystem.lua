-- Systems
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.Storage.GlobalStorage") --!temp path

-- Utilities
local QuickProfiler = require("_ECS_WIP_TEMP.Shared.Tools.QuickProfiler")

---@class InventorySystem
---@overload fun(self: InventorySystem): InventorySystem class internal
---@overload fun(): InventorySystem class external
local InventorySystem = Class(function(self)
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
        local itemEntity = GlobalStorage:getEntity(itemEntityInfo)
        ---@cast itemEntity ItemEntity
        local quantityComponent = itemEntity:findComponentByArchetype(Enums.ComponentArchetype.QuantityComponent)
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
            local cloneQuantityCmp = clone:findComponentByArchetype((Enums.ComponentArchetype.QuantityComponent))
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

return InventorySystem()
