local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local Helper = require("Shared.Helpers.MarketplaceSystemHelper")
local Items = require("Shared.Registries.Items")
local InventorySystem = require("Modules.Economy.Systems").Inventory
local Economy = require("Modules.Economy.Components")

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
        local traderEntityId = marketplace:getTrader()

        if not traderEntityId then
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

            if Registry:hasEntity(traderEntityId) then
                local bids = marketplace:getBids()
                local asks = marketplace:getAsks()

                local bidsEntities, asksEntities = Helper.getOrderEntities(bids, asks)
                self:processTrades(marketplace, bidsEntities, asksEntities)
            end
        end

        :: skipMarketplace ::
    end
    self.profiler:stop()
end

---@param marketplace MarketplaceComponent
---@param bids table<EntityId>
---@param asks table<EntityId>
function MarketplaceSystem:processTrades(marketplace, bids, asks)
    for bidId in Iterator(bids) do
        for askId in Iterator(asks) do
            local bidItemTypeCmp = Registry:get(bidId, Economy.OrderItemType)
            local bidPriceCmp = Registry:get(bidId, Economy.Price)
            local bidQuantityCmp = Registry:get(bidId, Economy.Quantity)

            local askItemTypeCmp = Registry:get(askId, Economy.OrderItemType)
            local askPriceCmp = Registry:get(askId, Economy.Price)
            local askQuantityCmp = Registry:get(askId, Economy.Quantity)

            local bidItemType = bidItemTypeCmp:getItemType()
            local bidPrice = bidPriceCmp:getPrice()
            local bidQuantity = bidQuantityCmp:getQuantity()
            local askItemType = askItemTypeCmp:getItemType()
            local askPrice = askPriceCmp:getPrice()
            local askQuantity = askQuantityCmp:getQuantity()

            -- Verify Inventory
            self.marketplaceParentEntity = marketplace:getEntityId()
            self.marketplaceInventoryCmp = Registry:get(self.marketplaceParentEntity, Economy.Inventory)

            Helper.printInventory(self.marketplaceParentEntity, self.marketplaceInventoryCmp)

            if bidItemType == askItemType and bidPrice >= askPrice then
                -- todo: reserve items here, put trade into trade queue for performance control
                -- todo: verify bank account in trade

                -- Calculate trade quantity
                local tradeQuantity = math.min(bidQuantity, askQuantity)

                -- Attempt to take the required items from the inventory
                local items = InventorySystem:take(self.marketplaceInventoryCmp, askItemType, tradeQuantity)

                if items then
                    -- Put traded items into the marketplace inventory (to simulate transfer)
                    for _, item in ipairs(items) do
                        Registry:destroyEntity(item) --! temp destroy
                    end

                    Log.Debug("[Transaction] Trader 1 %s (%d) -> Trader 2 for price %d credits", Items:getDefinition(bidItemType).name,
                        tradeQuantity,
                        bidPrice)

                    -- Update the inventory quantities
                    bidQuantity = bidQuantity - tradeQuantity
                    askQuantity = askQuantity - tradeQuantity

                    -- Update or remove the bid and ask orders
                    if bidQuantity == 0 then
                        marketplace:removeBid(bidId)
                        Registry:destroyEntity(bidId)
                    else
                        bidQuantityCmp:setQuantity(bidQuantity)
                    end

                    if askQuantity == 0 then
                        marketplace:removeAsk(askId)
                        Registry:destroyEntity(askId)
                    else
                        askQuantityCmp:setQuantity(askQuantity)
                    end

                    Helper.printInventory(self.marketplaceParentEntity, self.marketplaceInventoryCmp)
                end

                break
            end
        end
    end
end

return MarketplaceSystem()
