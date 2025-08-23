local Registry = require("Core.ECS.Registry")
local EconomyComponents = require("Modules.Economy.Components")
local EconomyEntities = require("Modules.Economy.Entities")
local ConstructsEntities = require("Modules.Constructs.Entities")
local CoreEntities = require("Modules.Core.Entities")

require("Shared.Definitions.ItemDefs")
local Items = require("Shared.Registries.Items")

local MarketplaceTest = require('States.Application')

---@diagnostic disable-next-line: duplicate-set-field
function MarketplaceTest:onInit()
    local trader = CoreEntities.Player("Trader Marketplace", true)
    local trader2 = CoreEntities.Player("Trader Marketplace 2", true)
    local traderEntityId = Registry:storeEntity(trader)
    local traderEntityId2 = Registry:storeEntity(trader2)

    local station = ConstructsEntities.SpaceStation(0)
    local station2 = ConstructsEntities.SpaceStation(1)

    local inventoryComponent = station:getComponent(EconomyComponents.Inventory)
    local inventoryComponent2 = station2:getComponent(EconomyComponents.Inventory)

    local creditItem = EconomyEntities.Item(Items.Virtual.Credit, 1e6)
    local creditItemEntityId = Registry:storeEntity(creditItem)
    local goldItem = EconomyEntities.Item(Items.RefinedMaterials.Gold, 1000)
    local goldItemEntityId = Registry:storeEntity(goldItem)
    inventoryComponent:addItem(Items.Virtual.Credit.id, creditItemEntityId)
    inventoryComponent:addItem(Items.RefinedMaterials.Gold.id, goldItemEntityId)

    local creditItem2 = EconomyEntities.Item(Items.Virtual.Credit, 1e6)
    local creditItemEntityId2 = Registry:storeEntity(creditItem2)
    local goldItem2 = EconomyEntities.Item(Items.RefinedMaterials.Gold, 1000)
    local goldItemEntityId2 = Registry:storeEntity(goldItem2)
    inventoryComponent2:addItem(Items.Virtual.Credit.id, creditItemEntityId2)
    inventoryComponent2:addItem(Items.RefinedMaterials.Gold.id, goldItemEntityId2)

    local marketplaceComponent = station:getComponent(EconomyComponents.Marketplace)
    local marketplaceComponent2 = station2:getComponent(EconomyComponents.Marketplace)

    local bidOrder = EconomyEntities.Order(0, Items.RefinedMaterials.Gold.id, 50, 500)
    local bidOrder2 = EconomyEntities.Order(1, Items.RefinedMaterials.Gold.id, 50, 500)
    local askOrder = EconomyEntities.Order(0, Items.RefinedMaterials.Gold.id, 50, 500)
    local askOrder2 = EconomyEntities.Order(1, Items.RefinedMaterials.Gold.id, 50, 500)

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
