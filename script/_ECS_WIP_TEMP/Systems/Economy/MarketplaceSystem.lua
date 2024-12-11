-- Systems
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.Storage.GlobalStorage") --!temp path
local InventorySystem = require("_ECS_WIP_TEMP.Systems.Economy.InventorySystem")

-- Utilities
local QuickProfiler = require("_ECS_WIP_TEMP.Shared.Tools.QuickProfiler")
local Helper = require("_ECS_WIP_TEMP.Shared.Helpers.MarketplaceSystemHelper")

---@class MarketplaceSystem
---@overload fun(self: MarketplaceSystem): MarketplaceSystem class internal
---@overload fun(): MarketplaceSystem class external
local MarketplaceSystem = Class(function(self)
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

    local marketplaces = GlobalStorage:getComponentsFromArchetype(Enums.ComponentArchetype.MarketplaceComponent)
    ---@cast marketplaces table<MarketplaceComponent>

    local now = TimeStamp.Now()

    if marketplaces and #marketplaces > 0 then
        ---@param marketplace MarketplaceComponent
        for index, marketplace in IteratorIndexed(marketplaces) do
            local traderEntityInfo = marketplace:getTrader()

            if not traderEntityInfo then
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
                local trader = GlobalStorage:getEntity(traderEntityInfo)

                if trader then
                    local bids = marketplace:getBids()
                    local asks = marketplace:getAsks()

                    local bidsEntities, asksEntities = Helper.getOrderEntities(bids, asks)
                    self:processTrades(marketplace, bidsEntities, asksEntities)
                end
            end

            :: skipMarketplace ::
        end
    end
    self.profiler:stop()
end

---@param marketplace MarketplaceComponent
---@param bids table<OrderEntity>
---@param asks table<OrderEntity>
function MarketplaceSystem:processTrades(marketplace, bids, asks)
    for bid in Iterator(bids) do
        for ask in Iterator(asks) do
            local bidItemTypeCmp = bid:findComponentByArchetype(Enums.ComponentArchetype.ItemTypeComponent)
            ---@cast bidItemTypeCmp ItemTypeComponent
            local bidPriceCmp = bid:findComponentByArchetype(Enums.ComponentArchetype.PriceComponent)
            ---@cast bidPriceCmp PriceComponent
            local bidQuantityCmp = bid:findComponentByArchetype(Enums.ComponentArchetype.QuantityComponent)
            ---@cast bidQuantityCmp QuantityComponent

            local askItemTypeCmp = ask:findComponentByArchetype(Enums.ComponentArchetype.ItemTypeComponent)
            ---@cast askItemTypeCmp ItemTypeComponent
            local askPriceCmp = ask:findComponentByArchetype(Enums.ComponentArchetype.PriceComponent)
            ---@cast askPriceCmp PriceComponent
            local askQuantityCmp = ask:findComponentByArchetype(Enums.ComponentArchetype.QuantityComponent)
            ---@cast askQuantityCmp QuantityComponent

            local bidItemType = bidItemTypeCmp:getItemType()
            local bidPrice = bidPriceCmp:getPrice()
            local bidQuantity = bidQuantityCmp:getQuantity()
            local askItemType = askItemTypeCmp:getItemType()
            local askPrice = askPriceCmp:getPrice()
            local askQuantity = askQuantityCmp:getQuantity()

            -- Verify Inventory
            self.marketplaceParentInfo = marketplace:getEntity()
            self.marketplaceParentEntity = GlobalStorage:getEntity(self.marketplaceParentInfo)
            ---@type InventoryComponent
            self.marketplaceInventoryCmp = self.marketplaceParentEntity:findComponentByArchetype(Enums.ComponentArchetype
                .InventoryComponent)

            Helper.printInventory(self.marketplaceParentEntity, self.marketplaceInventoryCmp)

            -- Verify Bank Account

            if bidItemType == askItemType and bidPrice >= askPrice then
                -- Calculate trade quantity
                local tradeQuantity = math.min(bidQuantity, askQuantity)

                -- Attempt to take the required items from the inventory
                local items = InventorySystem:take(self.marketplaceInventoryCmp, askItemType, tradeQuantity)

                if items then
                    -- Put traded items into the marketplace inventory (to simulate transfer)
                    for _, item in ipairs(items) do
                        GlobalStorage:getEntity(item):destroy() --! temp destroy
                    end

                    Log.Debug("[Transaction] Trader 1 %s (%d) -> Trader 2 for price %d credits", bidItemType, tradeQuantity, bidPrice)

                    -- Update the inventory quantities
                    bidQuantity = bidQuantity - tradeQuantity
                    askQuantity = askQuantity - tradeQuantity

                    -- Update or remove the bid and ask orders
                    if bidQuantity == 0 then
                        marketplace:removeBid(bid:getEntityInfo())
                    else
                        bid:setQuantity(bidQuantity)
                    end

                    if askQuantity == 0 then
                        marketplace:removeAsk(ask:getEntityInfo())
                    else
                        ask:setQuantity(askQuantity)
                    end

                    Helper.printInventory(self.marketplaceParentEntity, self.marketplaceInventoryCmp)
                end

                break
            end
        end
    end
end

return MarketplaceSystem()
