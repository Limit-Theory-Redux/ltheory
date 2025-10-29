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

                local bidsEntities, asksEntities = Helper.getOrderEntities(bids, asks)
                self:processTrades(marketplace, bidsEntities, asksEntities)
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
        for ask in Iterator(asks) do
            local bidItemTypeCmp = bid:get(Economy.ItemType)
            local bidPriceCmp = bid:get(Economy.Price)
            local bidQuantityCmp = bid:get(Economy.Quantity)

            local askIssuer = ask:get(Economy.Ownership):getOwner()
            local askItemTypeCmp = ask:get(Economy.ItemType)
            local askPriceCmp = ask:get(Economy.Price)
            local askQuantityCmp = ask:get(Economy.Quantity)

            local bidIssuer = bid:get(Economy.Ownership):getOwner()
            local bidItemType = bidItemTypeCmp:getItemType()
            local bidPrice = bidPriceCmp:getPrice()
            local bidQuantity = bidQuantityCmp:getQuantity()
            local askItemType = askItemTypeCmp:getItemType()
            local askPrice = askPriceCmp:getPrice()
            local askQuantity = askQuantityCmp:getQuantity()

            local traderInventory = Entity(marketplace:getEntityId()):get(Economy.Inventory)

            local askEntityParent = askIssuer:get(Core.Parent):getParent()
            local askEntityParentInventory = askEntityParent:get(Economy.Inventory)

            local bidEntityParent = bidIssuer:get(Core.Parent):getParent()
            local bidEntityParentInventory = bidEntityParent:get(Economy.Inventory)

            if bidItemType == askItemType and bidPrice >= askPrice then
                -- Log.Debug("item type matches. BidPrice >= askPrice")

                -- todo: reserve items here, put trade into trade queue for performance control
                -- todo: put into space station inventory
                -- todo: verify bank account in trade

                -- Calculate trade quantity
                local tradeQuantity = math.min(bidQuantity, askQuantity)

                -- Attempt to take the required items from the inventory

                local items = InventoryManager:take(askEntityParentInventory, askItemType, tradeQuantity)
                local itemName = Items:getDefinition(askItemType).name
                Helper.printInventoryDiff(askEntityParentInventory, itemName, tradeQuantity, true)

                if items then
                    -- Put traded items into the RECEIVING marketplace inventory (to simulate transfer)
                    -- (Give stuff to buyer)

                    InventoryManager:put(bidEntityParentInventory, items)
                    Helper.printInventoryDiff(bidEntityParentInventory, itemName, tradeQuantity, false)

                    Log.Debug("[Transaction] Trader 1 %s (%d) -> Trader 2 for price %d credits", Items:getDefinition(bidItemType).name,
                        tradeQuantity,
                        bidPrice)

                    -- Update the inventory quantities
                    bidQuantity = bidQuantity - tradeQuantity
                    askQuantity = askQuantity - tradeQuantity

                    -- Update or remove the bid and ask orders
                    if bidQuantity == 0 then
                        marketplace:removeBid(bid)
                        Registry:destroyEntity(bid)
                    else
                        bidQuantityCmp:setQuantity(bidQuantity)
                    end

                    if askQuantity == 0 then
                        marketplace:removeAsk(ask)
                        Registry:destroyEntity(ask)
                    else
                        askQuantityCmp:setQuantity(askQuantity)
                    end
                end

                break
            end
        end
    end
end

return MarketplaceSystem()
