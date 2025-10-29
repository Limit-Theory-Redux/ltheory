local Registry = require("Core.ECS.Registry")
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
    local STATION_ID = 214523059
    local SHIP_ONE_ID = 20134130294
    local SHIP_TWO_ID = 322234324

    local trader = CoreEntities.Player("Trader Marketplace", true)
    local station = ConstructsEntities.SpaceStation(STATION_ID)
    local marketplaceComponent = station:get(EconomyComponents.Marketplace)

    local playerOne = CoreEntities.Player("Jack", true)
    local shipOne = ConstructsEntities.Spaceship(SHIP_ONE_ID)

    local playerTwo = CoreEntities.Player("Davy Jones", true)
    local shipTwo = ConstructsEntities.Spaceship(SHIP_TWO_ID)

    Registry:attachEntity(shipOne, playerOne)
    Registry:attachEntity(shipTwo, playerTwo)

    local inventoryComponent = shipOne:get(EconomyComponents.Inventory)
    local inventoryComponent2 = shipTwo:get(EconomyComponents.Inventory)

    inventoryComponent:addItem(EconomyEntities.Item(Items.Virtual.Credit, 1e6))
    inventoryComponent:addItem(EconomyEntities.Item(Items.RefinedMaterials.Gold, 1000))

    inventoryComponent2:addItem(EconomyEntities.Item(Items.Virtual.Credit, 1e7))
    inventoryComponent2:addItem(EconomyEntities.Item(Items.Data.InfoWafer, 500))

    local bidOne = EconomyEntities.Order(playerOne, Items.Data.InfoWafer, 10, 1000)
    local bidTwo = EconomyEntities.Order(playerTwo, Items.RefinedMaterials.Gold, 50, 500)
    local askOrder = EconomyEntities.Order(playerOne, Items.RefinedMaterials.Gold, 50, 500)
    local askOrder2 = EconomyEntities.Order(playerTwo, Items.Data.InfoWafer, 10, 500)

    marketplaceComponent:setTrader(trader)
    marketplaceComponent:addBid(bidOne, bidTwo)
    marketplaceComponent:addAsk(askOrder, askOrder2)
end

return MarketplaceTest
