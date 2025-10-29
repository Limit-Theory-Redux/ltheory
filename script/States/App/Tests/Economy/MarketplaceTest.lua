local Registry = require("Core.ECS.Registry")
local EconomyComponents = require("Modules.Economy.Components")
local EconomyEntities = require("Modules.Economy.Entities")
local ConstructsEntities = require("Modules.Constructs.Entities")
local CoreEntities = require("Modules.Core.Entities")
local CoreComponents = require("Modules.Core.Components")

require("Shared.Definitions.ItemDefs")
local Items = require("Shared.Registries.Items")
local Tags = require("Shared.Registries.Tags")

local MarketplaceTest = require('States.Application')
require("Modules.Economy.Systems")

---@diagnostic disable-next-line: duplicate-set-field
function MarketplaceTest:onInit()
    Log.Info("[TagTest] Setting up MarketplaceTest environment")

    -- Entity setup
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

    -- Inventory setup
    local inv1 = shipOne:get(EconomyComponents.Inventory)
    local inv2 = shipTwo:get(EconomyComponents.Inventory)

    inv1:addItem(EconomyEntities.Item(Items.Virtual.Credit, 1e6))
    inv1:addItem(EconomyEntities.Item(Items.RefinedMaterials.Gold, 1000))

    inv2:addItem(EconomyEntities.Item(Items.Virtual.Credit, 1e7))
    inv2:addItem(EconomyEntities.Item(Items.Data.InfoWafer, 500))

    --! Tag setup: groups and tags, do this here temporarily until we decide where these should be defined
    Tags:new("TradeType", { "BidOrder", "AskOrder", "Contract", "Auction", "Brokered" })
    Tags:new("Goods", { "HighDemand", "LowSupply", "Volatile", "Luxury", "Consumable", "RawMaterial" })
    Tags:new("Legality", { "Legal", "Restricted", "Contraband" })
    Tags:new("Context", { "OrgTrade", "MissionTrade", "EventTrade" })

    -- Orders
    local bid1 = EconomyEntities.Order(playerOne, Items.Data.InfoWafer, 10, 1000)
    local bid2 = EconomyEntities.Order(playerTwo, Items.RefinedMaterials.Gold, 50, 500)
    local ask1 = EconomyEntities.Order(playerOne, Items.RefinedMaterials.Gold, 50, 500)
    local ask2 = EconomyEntities.Order(playerTwo, Items.Data.InfoWafer, 10, 500)
    local orders = { bid1, bid2, ask1, ask2 }

    -- Assign meaningful tags
    for _, o in ipairs(orders) do
        local tagComp = o:get(CoreComponents.Tag)
        -- Type
        if o == bid1 or o == bid2 then
            tagComp:addTag("BidOrder")
        else
            tagComp:addTag("AskOrder")
        end

        -- Goods classification
        local itemType = o:get(EconomyComponents.ItemType):getItemType()
        if itemType == Items.Data.InfoWafer.id then
            tagComp:addTag("HighDemand")
        else
            tagComp:addTag("RawMaterial")
        end

        -- Legality / context
        tagComp:addTag("Legal")
        tagComp:addTag("OrgTrade")
    end

    -- Marketplace setup
    marketplaceComponent:setTrader(trader)
    marketplaceComponent:addBid(bid1, bid2)
    marketplaceComponent:addAsk(ask1, ask2)

    -- Assertions
    for _, o in ipairs(orders) do
        local tagComp = o:get(CoreComponents.Tag)
        assert(tagComp:hasAnyTagInGroup("TradeType"), "Order missing TradeType tag")
        assert(tagComp:hasAnyTagInGroup("Goods"), "Order missing Goods tag")
        assert(tagComp:hasAnyTagInGroup("Legality"), "Order missing Legality tag")
        assert(tagComp:hasAnyTagInGroup("Context"), "Order missing Context tag")
    end

    Log.Info("[TagTest] MarketplaceTest setup complete with all orders tagged correctly")
end

return MarketplaceTest
