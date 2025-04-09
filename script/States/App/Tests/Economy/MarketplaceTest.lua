local Registry = require("Systems.Storage.Registry")
local Components = require("Components")

local PlayerEntity = require("Entities.Player")
local SpaceStationEntity = require("Entities.Constructs.SpaceStationEntity")
local ItemEntity = require("Entities.Economy.ItemEntity")
local OrderEntity = require("Entities.Economy.OrderEntity")

require("Shared.Definitions.ItemDefs")
local Items = require("Shared.Registries.Items")

local MarketplaceTest = require('States.Application')

---@diagnostic disable-next-line: duplicate-set-field
function MarketplaceTest:onInit()
    local trader = PlayerEntity("Trader Marketplace", true)
    local trader2 = PlayerEntity("Trader Marketplace 2", true)
    local traderEntityInfo = Registry:storeEntity(trader)
    local traderEntityInfo2 = Registry:storeEntity(trader2)

    local station = SpaceStationEntity(0)
    local station2 = SpaceStationEntity(1)

    local inventoryComponent = station:findComponentByArchetype(Components.InventoryComponent)
    local inventoryComponent2 = station2:findComponentByArchetype(Components.InventoryComponent)

    local creditItem = ItemEntity(Items.Virtual.Credit, 1e6)
    local creditItemEntityInfo = Registry:storeEntity(creditItem)
    local goldItem = ItemEntity(Items.RefinedMaterials.Gold, 1000)
    local goldItemEntityInfo = Registry:storeEntity(goldItem)
    inventoryComponent:addItem(Items.Virtual.Credit.id, creditItemEntityInfo)
    inventoryComponent:addItem(Items.RefinedMaterials.Gold.id, goldItemEntityInfo)

    local creditItem2 = ItemEntity(Items.Virtual.Credit, 1e6)
    local creditItemEntityInfo2 = Registry:storeEntity(creditItem2)
    local goldItem2 = ItemEntity(Items.RefinedMaterials.Gold, 1000)
    local goldItemEntityInfo2 = Registry:storeEntity(goldItem2)
    inventoryComponent2:addItem(Items.Virtual.Credit.id, creditItemEntityInfo2)
    inventoryComponent2:addItem(Items.RefinedMaterials.Gold.id, goldItemEntityInfo2)

    local marketplaceComponent = station:findComponentByArchetype(Components.MarketplaceComponent)
    local marketplaceComponent2 = station2:findComponentByArchetype(Components.MarketplaceComponent)

    local bidOrder = OrderEntity(0, Items.RefinedMaterials.Gold.id, 50, 500)
    local bidOrder2 = OrderEntity(1, Items.RefinedMaterials.Gold.id, 50, 500)
    local askOrder = OrderEntity(0, Items.RefinedMaterials.Gold.id, 50, 500)
    local askOrder2 = OrderEntity(1, Items.RefinedMaterials.Gold.id, 50, 500)

    local bidOrderEntityInfo = Registry:storeEntity(bidOrder)
    local bidOrderEntityInfo2 = Registry:storeEntity(bidOrder2)
    local askOrderEntityInfo = Registry:storeEntity(askOrder)
    local askOrderEntityInfo2 = Registry:storeEntity(askOrder2)

    marketplaceComponent:setTrader(traderEntityInfo)
    marketplaceComponent2:setTrader(traderEntityInfo2)

    marketplaceComponent:addBid(bidOrderEntityInfo)
    marketplaceComponent2:addBid(bidOrderEntityInfo2)
    marketplaceComponent:addAsk(askOrderEntityInfo)
    marketplaceComponent2:addAsk(askOrderEntityInfo2)

    Registry:storeEntity(station)
    Registry:storeEntity(station2)
end

return MarketplaceTest
