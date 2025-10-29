local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
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

---@param component InventoryComponent
local function printInventory(component)
    local parentEntity = Entity(component:getEntityId())
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

local function formatWinLoss(qty)
    if qty == 0 then
        return qty
    end

    if qty < 0 then
        return string.format("\27[1;101;93m %d \27[0m", qty)
    end

    if qty > 0 then
        return string.format("\27[1;102;93m +%d \27[0m", qty)
    end
end

local function formatBuyerSeller(isSeller)
    if isSeller then
        return string.format("\27[92mSeller\27[0m")
    else
        return string.format("\27[93mBuyer\27[0m")
    end
end

---@param component InventoryComponent
---@param itemTypeName string
---@param saleQty number
---@param isSeller boolean
local function printInventoryDiff(component, itemTypeName, saleQty, isSeller)
    local parentEntity = Entity(component:getEntityId())
    Log.Debug("%s - Inventory (%s)", parentEntity, formatBuyerSeller(isSeller))

    for itemTypes in Iterator(component:getInventory()) do
        for itemEntity in Iterator(itemTypes) do
            if Registry:hasEntity(itemEntity) then
                local itemName = itemEntity:get(NameComponent):getName()
                if isSeller and saleQty > 0 then
                    saleQty = saleQty * -1
                end

                if itemName == itemTypeName then
                    Log.Debug(" ├─ %s(%d) %s",
                        itemName,
                        itemEntity:get(QuantityComponent):getQuantity(),
                        formatWinLoss(saleQty)
                    )
                else
                    Log.Debug(" ├─ %s(%d)",
                        itemName,
                        itemEntity:get(QuantityComponent):getQuantity()
                    )
                end
            end
        end
    end
end


return {
    getOrderEntities = getOrderEntities,
    printInventory = printInventory,
    printInventoryDiff = printInventoryDiff
}
