local GlobalStorage = require("_ECS_WIP_TEMP.Systems.Storage.GlobalStorage")         --!temp path
local MarketplaceSystem = require("_ECS_WIP_TEMP.Systems.Economy.MarketplaceSystem") --!temp path

local PlayerEntity = require("_ECS_WIP_TEMP.Entities.Player")
local SpaceStationEntity = require("_ECS_WIP_TEMP.Entities.Constructs.SpaceStationEntity")
local ItemEntity = require("_ECS_WIP_TEMP.Entities.Economy.ItemEntity")
local OrderEntity = require("_ECS_WIP_TEMP.Entities.Economy.OrderEntity")

require("_ECS_WIP_TEMP.Shared.Definitions.ItemDefs")
local Items = require("_ECS_WIP_TEMP.Shared.Registries.Items")

local MarketplaceTest = require('States.Application')

---@diagnostic disable-next-line: duplicate-set-field
function MarketplaceTest:onInit()
    local trader = PlayerEntity("Trader", true)
    local trader2 = PlayerEntity("Trader 2", true)
    local traderEntityInfo = GlobalStorage:storeEntity(trader)
    local traderEntityInfo2 = GlobalStorage:storeEntity(trader2)

    local station = SpaceStationEntity(0)
    local station2 = SpaceStationEntity(1)

    ---@type InventoryComponent
    local inventoryComponent = station:findComponentByArchetype(Enums.ComponentArchetype.InventoryComponent)
    ---@type InventoryComponent
    local inventoryComponent2 = station2:findComponentByArchetype(Enums.ComponentArchetype.InventoryComponent)

    local creditItem = ItemEntity(Items.Virtual.Credit, 1e6)
    local creditItemEntityInfo = GlobalStorage:storeEntity(creditItem)
    local goldItem = ItemEntity(Items.RefinedMaterials.Gold, 1000)
    local goldItemEntityInfo = GlobalStorage:storeEntity(goldItem)
    inventoryComponent:put(creditItemEntityInfo)
    inventoryComponent:put(goldItemEntityInfo)

    local creditItem2 = ItemEntity(Items.Virtual.Credit, 1e6)
    local creditItemEntityInfo2 = GlobalStorage:storeEntity(creditItem2)
    local goldItem2 = ItemEntity(Items.RefinedMaterials.Gold, 1000)
    local goldItemEntityInfo2 = GlobalStorage:storeEntity(goldItem2)
    inventoryComponent2:put(creditItemEntityInfo2)
    inventoryComponent2:put(goldItemEntityInfo2)

    self:printInventory(station, inventoryComponent)
    self:printInventory(station2, inventoryComponent2)

    ---@type MarketplaceComponent
    local marketplaceComponent = station:findComponentByArchetype(Enums.ComponentArchetype.MarketplaceComponent)
    ---@type MarketplaceComponent
    local marketplaceComponent2 = station2:findComponentByArchetype(Enums.ComponentArchetype.MarketplaceComponent)

    local bidOrder = OrderEntity(0, Items.RefinedMaterials.Gold.name, 50, 500)
    local bidOrder2 = OrderEntity(1, Items.RefinedMaterials.Gold.name, 50, 500)
    local askOrder = OrderEntity(0, Items.RefinedMaterials.Gold.name, 50, 500)
    local askOrder2 = OrderEntity(1, Items.RefinedMaterials.Gold.name, 50, 500)

    local bidOrderEntityInfo = GlobalStorage:storeEntity(bidOrder)
    local bidOrderEntityInfo2 = GlobalStorage:storeEntity(bidOrder2)
    local askOrderEntityInfo = GlobalStorage:storeEntity(askOrder)
    local askOrderEntityInfo2 = GlobalStorage:storeEntity(askOrder2)

    marketplaceComponent:setTrader(traderEntityInfo)
    marketplaceComponent2:setTrader(traderEntityInfo2)

    marketplaceComponent:addBid(bidOrderEntityInfo)
    marketplaceComponent2:addBid(bidOrderEntityInfo2)
    marketplaceComponent:addAsk(askOrderEntityInfo)
    marketplaceComponent2:addAsk(askOrderEntityInfo2)

    GlobalStorage:storeEntity(station)
    GlobalStorage:storeEntity(station2)

    --TODO: Add recurring bids/asks and let the marketplaces trade
end

---@param parentEntity Entity
---@param component InventoryComponent
function MarketplaceTest:printInventory(parentEntity, component)
    Log.Debug("%s - Inventory", parentEntity)
    for itemTypes in Iterator(component:getInventory()) do
        for itemEntityInfo in Iterator(itemTypes) do
            local itemEntity = GlobalStorage:getEntity(itemEntityInfo)

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

return MarketplaceTest
