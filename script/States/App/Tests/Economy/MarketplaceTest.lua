local Registry = require("Core.ECS.Registry")
local Components = loadComponents("Economy")
local Entities = loadEntities("Core", "Constructs", "Economy")

require("Shared.Definitions.ItemDefs")
local Items = require("Shared.Registries.Items")

local MarketplaceTest = require('States.Application')

---@diagnostic disable-next-line: duplicate-set-field
function MarketplaceTest:onInit()
    local trader = Entities.PlayerEntity("Trader Marketplace", true)
    local trader2 = Entities.PlayerEntity("Trader Marketplace 2", true)
    local traderEntityId = Registry:storeEntity(trader)
    local traderEntityId2 = Registry:storeEntity(trader2)

    local station = Entities.SpaceStationEntity(0)
    local station2 = Entities.SpaceStationEntity(1)

    local inventoryComponent = station:getComponent(Components.InventoryComponent)
    local inventoryComponent2 = station2:getComponent(Components.InventoryComponent)

    local creditItem = Entities.ItemEntity(Items.Virtual.Credit, 1e6)
    local creditItemEntityId = Registry:storeEntity(creditItem)
    local goldItem = Entities.ItemEntity(Items.RefinedMaterials.Gold, 1000)
    local goldItemEntityId = Registry:storeEntity(goldItem)
    inventoryComponent:addItem(Items.Virtual.Credit.id, creditItemEntityId)
    inventoryComponent:addItem(Items.RefinedMaterials.Gold.id, goldItemEntityId)

    local creditItem2 = Entities.ItemEntity(Items.Virtual.Credit, 1e6)
    local creditItemEntityId2 = Registry:storeEntity(creditItem2)
    local goldItem2 = Entities.ItemEntity(Items.RefinedMaterials.Gold, 1000)
    local goldItemEntityId2 = Registry:storeEntity(goldItem2)
    inventoryComponent2:addItem(Items.Virtual.Credit.id, creditItemEntityId2)
    inventoryComponent2:addItem(Items.RefinedMaterials.Gold.id, goldItemEntityId2)

    local marketplaceComponent = station:getComponent(Components.MarketplaceComponent)
    local marketplaceComponent2 = station2:getComponent(Components.MarketplaceComponent)

    local bidOrder = Entities.OrderEntity(0, Items.RefinedMaterials.Gold.id, 50, 500)
    local bidOrder2 = Entities.OrderEntity(1, Items.RefinedMaterials.Gold.id, 50, 500)
    local askOrder = Entities.OrderEntity(0, Items.RefinedMaterials.Gold.id, 50, 500)
    local askOrder2 = Entities.OrderEntity(1, Items.RefinedMaterials.Gold.id, 50, 500)

    local bidOrderEntityId = Registry:storeEntity(bidOrder)
    local bidOrderEntityId2 = Registry:storeEntity(bidOrder2)
    local askOrderEntityId = Registry:storeEntity(askOrder)
    local askOrderEntityId2 = Registry:storeEntity(askOrder2)

    marketplaceComponent:setTrader(traderEntityId)
    marketplaceComponent2:setTrader(traderEntityId2)

    marketplaceComponent:addBid(bidOrderEntityId)
    marketplaceComponent2:addBid(bidOrderEntityId2)
    marketplaceComponent:addAsk(askOrderEntityId)
    marketplaceComponent2:addAsk(askOrderEntityId2)

    Registry:storeEntity(station)
    Registry:storeEntity(station2)
end

return MarketplaceTest
