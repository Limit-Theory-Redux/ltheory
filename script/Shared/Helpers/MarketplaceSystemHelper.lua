local Registry = require("Core.ECS.Registry")

---@param bids table<EntityId>
---@param asks table<EntityId>
---@return table<OrderEntity> bids, table<OrderEntity> asks
local function getOrderEntities(bids, asks)
    local bidEntities, askEntities = {}, {}

    for entityId in Iterator(bids) do
        local entity = Registry:getEntity(entityId)
        if entity then
            insert(bidEntities, entity)
        end
    end

    for entityId in Iterator(asks) do
        local entity = Registry:getEntity(entityId)
        if entity then
            insert(askEntities, entity)
        end
    end

    return bidEntities, askEntities
end

---@param parentEntity Entity
---@param component InventoryComponent
local function printInventory(parentEntity, component)
    local QuantityComponent = require("Modules.Economy").Components.Quantity

    Log.Debug("%s - Inventory", parentEntity)
    for itemTypes in Iterator(component:getInventory()) do
        for itemEntityId in Iterator(itemTypes) do
            local itemEntity = Registry:getEntity(itemEntityId)

            if itemEntity then
                local quantityComponent = itemEntity:getComponent(QuantityComponent)

                Log.Debug(" ├─ %s(%d)", itemEntity:getName(), quantityComponent:getQuantity())
            end
        end
    end
end

return {
    getOrderEntities = getOrderEntities,
    printInventory = printInventory
}
