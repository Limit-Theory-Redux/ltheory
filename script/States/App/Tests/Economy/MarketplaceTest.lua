local EconomyComponents = require("Modules.Economy.Components")
local EconomyEntities = require("Modules.Economy.Entities")
local ConstructsEntities = require("Modules.Constructs.Entities")
local CoreEntities = require("Modules.Core.Entities")

require("Shared.Definitions.ItemDefs")
local Items = require("Shared.Registries.Items")

local MarketplaceTest = require('States.Application')

require("Modules.Economy.Systems")

---@diagnostic disable-next-line: duplicate-set-field
function MarketplaceTest:onInit()
    local trader = CoreEntities.Player("Trader Marketplace", true)
    local trader2 = CoreEntities.Player("Trader Marketplace 2", true)

    local station = ConstructsEntities.SpaceStation(0)
    local station2 = ConstructsEntities.SpaceStation(1)

    local inventoryComponent = station:get(EconomyComponents.Inventory)
    local inventoryComponent2 = station2:get(EconomyComponents.Inventory)

    local creditItem = EconomyEntities.Item(Items.Virtual.Credit, 1e6)
    local goldItem = EconomyEntities.Item(Items.RefinedMaterials.Gold, 1000)
    inventoryComponent:addItem(Items.Virtual.Credit.id, creditItem)
    inventoryComponent:addItem(Items.RefinedMaterials.Gold.id, goldItem)

    local creditItem2 = EconomyEntities.Item(Items.Virtual.Credit, 1e6)
    local goldItem2 = EconomyEntities.Item(Items.RefinedMaterials.Gold, 1000)
    inventoryComponent2:addItem(Items.Virtual.Credit.id, creditItem2)
    inventoryComponent2:addItem(Items.RefinedMaterials.Gold.id, goldItem2)

    local marketplaceComponent = station:get(EconomyComponents.Marketplace)
    local marketplaceComponent2 = station2:get(EconomyComponents.Marketplace)

    local bidOrder = EconomyEntities.Order(0, Items.RefinedMaterials.Gold.id, 50, 500)
    local bidOrder2 = EconomyEntities.Order(1, Items.RefinedMaterials.Gold.id, 50, 500)
    local askOrder = EconomyEntities.Order(0, Items.RefinedMaterials.Gold.id, 50, 500)
    local askOrder2 = EconomyEntities.Order(1, Items.RefinedMaterials.Gold.id, 50, 500)

    marketplaceComponent:setTrader(trader)
    marketplaceComponent2:setTrader(trader2)

    marketplaceComponent:addBid(bidOrder)
    marketplaceComponent2:addBid(bidOrder2)
    marketplaceComponent:addAsk(askOrder)
    marketplaceComponent2:addAsk(askOrder2)
end

return MarketplaceTest
