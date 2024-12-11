-- Systems
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.Storage.GlobalStorage") --!temp path

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

function InventorySystem:take(inventory, type, quantity)
    local itemsOfType = inventory:getItemsOfType(type)
    local takenItems = {}
    local remainingQuantity = quantity

    for id, itemEntityInfo in pairs(itemsOfType) do
        local itemEntity = GlobalStorage:getEntity(itemEntityInfo)
        local quantityComponent = itemEntity:findComponentByArchetype(Enums.ComponentArchetype.QuantityComponent)
        local itemQuantity = quantityComponent:getQuantity()

        if itemQuantity <= remainingQuantity then
            -- Take entire item
            inventory:removeItem(type, id)
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

    return remainingQuantity == 0 and takenItems or nil
end

function InventorySystem:put(inventory, type, items)
    for _, itemEntityInfo in ipairs(items) do
        inventory:addItem(type, itemEntityInfo)
    end
end

return InventorySystem
