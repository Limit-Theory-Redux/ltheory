-- Systems
local Registry = require("Systems.Storage.Registry")

---@param bids table<EntityInfo>
---@param asks table<EntityInfo>
---@return table<OrderEntity> bids, table<OrderEntity> asks
local function getOrderEntities(bids, asks)
    local bidEntities, askEntities = {}, {}

    for entityInfo in Iterator(bids) do
        local entity = Registry:getEntity(entityInfo)
        if entity then
            insert(bidEntities, entity)
        end
    end

    for entityInfo in Iterator(asks) do
        local entity = Registry:getEntity(entityInfo)
        if entity then
            insert(askEntities, entity)
        end
    end

    return bidEntities, askEntities
end

---@param parentEntity Entity
---@param component InventoryComponent
local function printInventory(parentEntity, component)
    Log.Debug("%s - Inventory", parentEntity)
    for itemTypes in Iterator(component:getInventory()) do
        for itemEntityInfo in Iterator(itemTypes) do
            local itemEntity = Registry:getEntity(itemEntityInfo)

            if itemEntity then
                ---@type NameComponent
                local nameComponent = itemEntity:findComponentByArchetype(Enums.ComponentArchetype.NameComponent)
                ---@type QuantityComponent
                local quantityComponent = itemEntity:findComponentByArchetype(Enums.ComponentArchetype.QuantityComponent)

                Log.Debug(" ├─ %s(%d)", nameComponent:getName(), quantityComponent:getQuantity())
            end
        end
    end
end

return {
    getOrderEntities = getOrderEntities,
    printInventory = printInventory
}
