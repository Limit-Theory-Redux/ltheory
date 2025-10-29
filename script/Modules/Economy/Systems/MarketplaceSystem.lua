local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")

local QuickProfiler = require("Shared.Tools.QuickProfiler")
local Helper = require("Shared.Helpers.MarketplaceSystemHelper")
local Items = require("Shared.Registries.Items")

local Economy = require("Modules.Economy.Components")
local Core = require("Modules.Core.Components")

local InventoryManager = require("Modules.Economy.Managers.InventoryManager")

---@class MarketplaceSystem
---@overload fun(self: MarketplaceSystem): MarketplaceSystem class internal
---@overload fun(): MarketplaceSystem class external
local MarketplaceSystem = Class("MarketplaceSystem", function(self)
    ---@diagnostic disable-next-line: invisible
    self:registerVars()
    ---@diagnostic disable-next-line: invisible
    self:registerEvents()
end)

---@private
function MarketplaceSystem:registerVars()
    ---@private
    self.profiler = QuickProfiler("MarketplaceSystem", false, false)

    self.rng = RNG.FromTime()
    self.maxUpdateRateDeviation = 0.5
    self.updateRate = 2
end

---@private
function MarketplaceSystem:registerEvents()
    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
end

function MarketplaceSystem:onPreRender()
    self.profiler:start()

    local now = TimeStamp.Now()

    for _, marketplace in Registry:iterEntities(Economy.Marketplace) do
        ---@cast marketplace MarketplaceComponent

        local traderEntity = marketplace:getTrader()

        if not traderEntity then
            goto skipMarketplace
        end

        local nextUpdate = marketplace:getNextUpdate()

        if not nextUpdate then
            nextUpdate = TimeStamp.GetFuture(self.updateRate + self.rng:getUniformRange(0, self.maxUpdateRateDeviation))
            marketplace:setNextUpdate(nextUpdate)
            goto skipMarketplace
        end

        -- update
        if now:getDifference(nextUpdate) <= 0 then
            nextUpdate = TimeStamp.GetFuture(self.updateRate + self.rng:getUniformRange(0, self.maxUpdateRateDeviation))
            marketplace:setNextUpdate(nextUpdate)
            --[[ Todo
                - Update orders
                - Update item flow
            ]]

            if Registry:hasEntity(traderEntity) then
                local bids = marketplace:getBids()
                local asks = marketplace:getAsks()

                --[[
                local bidEntities, askEntities = {}, {}

                for entity in Iterator(bids) do
                    if Registry:hasEntity(entity) then
                        insert(bidEntities, entity)
                    end
                end

                for entity in Iterator(asks) do
                    if Registry:hasEntity(entity) then
                        insert(askEntities, entity)
                    end
                end
                ]]

                self:processTrades(marketplace, bids, asks)
            end
        end

        :: skipMarketplace ::
    end
    self.profiler:stop()
end

---@param marketplace MarketplaceComponent
---@param bids table<Entity>
---@param asks table<Entity>
function MarketplaceSystem:processTrades(marketplace, bids, asks)
    for bid in Iterator(bids) do
        local bidTagCmp = bid:get(Core.Tag)
        local bidItemTypeCmp = bid:get(Economy.ItemType)
        local bidPriceCmp = bid:get(Economy.Price)
        local bidQuantityCmp = bid:get(Economy.Quantity)
        local bidIssuer = bid:get(Economy.Ownership):getOwner()
        local bidItemType = bidItemTypeCmp:getItemType()
        local bidPrice = bidPriceCmp:getPrice()
        local bidQuantity = bidQuantityCmp:getQuantity()

        -- Skip illegal bids
        if bidTagCmp:hasTag("Contraband") then
            Log.Warn("[Marketplace] Skipping contraband bid by", bidIssuer:getName())
            goto continueBid
        end

        -- Optional: Adjust bid based on HighDemand, Volatile, etc.
        if bidTagCmp:hasTag("HighDemand") then
            bidPrice = bidPrice * 1.2
        elseif bidTagCmp:hasTag("LowSupply") then
            bidPrice = bidPrice * 1.1
        end

        for ask in Iterator(asks) do
            local askTagCmp = ask:get(Core.Tag)
            local askItemTypeCmp = ask:get(Economy.ItemType)
            local askPriceCmp = ask:get(Economy.Price)
            local askQuantityCmp = ask:get(Economy.Quantity)
            local askIssuer = ask:get(Economy.Ownership):getOwner()
            local askItemType = askItemTypeCmp:getItemType()
            local askPrice = askPriceCmp:getPrice()
            local askQuantity = askQuantityCmp:getQuantity()

            -- Skip illegal asks
            if askTagCmp:hasTag("Contraband") then
                Log.Warn("[Marketplace] Skipping contraband ask by", askIssuer:getName())
                goto continueAsk
            end

            -- Filter by item type and trade legality
            if bidItemType == askItemType and bidPrice >= askPrice then
                local bidParentInv = bidIssuer:get(Core.Parent):getParent():get(Economy.Inventory)
                local askParentInv = askIssuer:get(Core.Parent):getParent():get(Economy.Inventory)

                -- Determine trade quantity
                local tradeQuantity = math.min(bidQuantity, askQuantity)

                -- Attempt to take items from seller inventory
                local items = InventoryManager:take(askParentInv, askItemType, tradeQuantity)
                if items then
                    -- Transfer to buyer
                    InventoryManager:put(bidParentInv, items)

                    Log.Debug("[Transaction] %s (%d) -> %s for price %d",
                        Items:getDefinition(bidItemType).name, tradeQuantity,
                        bidIssuer, bidPrice)

                    -- Update quantities
                    bidQuantity = bidQuantity - tradeQuantity
                    askQuantity = askQuantity - tradeQuantity

                    -- Handle bid/ask removal or cloning
                    if bidQuantity == 0 then
                        local clone = Registry:cloneEntity(bid)
                        clone:get(Economy.Ownership):setOwner(askIssuer)
                        marketplace:addBid(clone)
                        marketplace:removeBid(bid)
                        Registry:destroyEntity(bid)
                        break
                    else
                        bidQuantityCmp:setQuantity(bidQuantity)
                    end

                    if askQuantity == 0 then
                        local clone = Registry:cloneEntity(ask)
                        clone:get(Economy.Ownership):setOwner(bidIssuer)
                        marketplace:addAsk(clone)
                        marketplace:removeAsk(ask)
                        Registry:destroyEntity(ask)
                    else
                        askQuantityCmp:setQuantity(askQuantity)
                    end
                end
            end
            ::continueAsk::
        end
        ::continueBid::
    end
end

return MarketplaceSystem()
