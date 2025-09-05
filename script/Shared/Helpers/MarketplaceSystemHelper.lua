local Registry = require("Core.ECS.Registry")
local NameComponent = require("Core.ECS.NameComponent")
local QuantityComponent = require("Modules.Economy.Components").Quantity

---@param bids table<EntityId>
---@param asks table<EntityId>
---@return table<EntityId> bids, table<EntityId> asks
local function getOrderEntities(bids, asks)
    local bidEntities, askEntities = {}, {}

    for entityId in Iterator(bids) do
        if Registry:hasEntity(entityId) then
            insert(bidEntities, entityId)
        end
    end

    for entityId in Iterator(asks) do
        if Registry:hasEntity(entityId) then
            insert(askEntities, entityId)
        end
    end

    return bidEntities, askEntities
end

---@param parentEntity EntityId
---@param component InventoryComponent
local function printInventory(parentEntity, component)
    Log.Debug("%s(%s) - Inventory", Registry:get(parentEntity, NameComponent):getName(), parentEntity)
    for itemTypes in Iterator(component:getInventory()) do
        for itemEntityId in Iterator(itemTypes) do
            if Registry:hasEntity(itemEntityId) then
                local nameComponent = Registry:get(itemEntityId, NameComponent)
                local quantityComponent = Registry:get(itemEntityId, QuantityComponent)
                Log.Debug(" ├─ %s(%d)", nameComponent:getName(), quantityComponent:getQuantity())
            end
        end
    end
end

return {
    getOrderEntities = getOrderEntities,
    printInventory = printInventory
}
