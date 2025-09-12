local Registry = require("Core.ECS.Registry")
local NameComponent = require("Modules.Core.Components.NameComponent")
local QuantityComponent = require("Modules.Economy.Components").Quantity

---@param bids table<Entity>
---@param asks table<Entity>
---@return table<Entity> bids, table<Entity> asks
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

---@param parentEntity Entity
---@param component InventoryComponent
local function printInventory(parentEntity, component)
    Log.Debug("%s - Inventory", parentEntity)
    for itemTypes in Iterator(component:getInventory()) do
        for itemEntity in Iterator(itemTypes) do
            if Registry:hasEntity(itemEntity) then
                Log.Debug(" ├─ %s(%d)",
                    itemEntity:get(NameComponent):getName(),
                    itemEntity:get(QuantityComponent):getQuantity()
                )
            end
        end
    end
end

return {
    getOrderEntities = getOrderEntities,
    printInventory = printInventory
}
