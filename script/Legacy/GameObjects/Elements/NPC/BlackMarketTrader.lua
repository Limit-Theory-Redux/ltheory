local Entity = require('Legacy.GameObjects.Entity')
local Credit = require('Legacy.Systems.Economy.Item').Credit
local Item = require('Legacy.Systems.Economy.Item')

--------------------------------------------------------------------------------

-- NOTE: All the evaluations below are made from the perspective of what is most
-- advantageous to _this trader_ (self).

local BlackMarketTrader = Class("BlackMarketTrader", function(self, parent)
    self.parent = parent
    self.elems = {}
end)

function BlackMarketTrader:getData(item)
    if not self.elems[item] then
        self.elems[item] = {
            asks = {},
            bids = {},
            asksQueue = {},
            bidsQueue = {},
            totalAsk = 0,
            totalAskPrice = 0,
            totalBid = 0,
            totalBidPrice = 0,
            escrow = 0,
            askOffers = {},
            bidOffers = {},
        }
    end
    return self.elems[item]
end

function BlackMarketTrader:addAsk(item, price)
    local askAdded = false

    if self.parent:hasItem(item, 1) then
        -- Offer an ask only if trader has at least 1 unit of the item in stock
        local data = self:getData(item)

        -- Go ahead and remove the item now (at Ask creation time) to keep asks only at the number of items in stock
        self.parent:removeItem(item, 1)
        data.escrow = data.escrow + 1

        data.totalAsk = data.totalAsk + 1
        data.totalAskPrice = data.totalAskPrice + price

        insert(data.asksQueue, price)
        --printf("Added ask to sell item %s at station %s for price %d", item:getName(), self.parent:getName(), price)
        askAdded = true
    end

    return askAdded
end

function BlackMarketTrader:addAskOffer(bidder)
    local count = bidder.job.jcount
    local item = bidder.job.item
    local data = self:getData(item)
    local askOffersAdded = 0

    for i = 1, count do
        if #data.askOffers < data.totalAsk then
            insert(data.askOffers, bidder)
            askOffersAdded = askOffersAdded + 1
        end
    end

    if askOffersAdded > 0 then
        printf("TRADER: Added %d ask offers from %s to obtain %d units of %s from %s | asks = %d, offers = %d",
            askOffersAdded, bidder:getName(), count, item:getName(), self.parent:getName(), data.totalAsk,
            #data.askOffers)
    else
        printf(
            "TRADER ***: Couldn't add any ask offers from %s to obtain %d units of %s from %s | asks = %d, offers = %d",
            bidder:getName(), count, item:getName(), self.parent:getName(), data.totalAsk, #data.askOffers)
    end

    return askOffersAdded
end

function BlackMarketTrader:addBid(item, price)
    local data = self:getData(item)

    data.totalBid = data.totalBid + 1
    data.totalBidPrice = data.totalBidPrice + price

    insert(data.bidsQueue, price)
    --printf("Added bid to buy item %s at station %s for price %d", item:getName(), self.parent:getName(), price)
    return true
end

function BlackMarketTrader:addBidOffer(bidder)
    local count = bidder.job.jcount
    -- first item for now .. Sketchy stuff in the following line :D
    local item = nil
    for a, v in pairs(self.elems) do
        if a ~= nil then
            item = a
            break
        end
    end
    local data = self:getData(item)
    local offersAdded = 0

    for i = 1, count do
        if #data.bidOffers < data.totalBid then
            insert(data.bidOffers, bidder)
            offersAdded = offersAdded + 1
        end
    end

    if offersAdded > 0 then
        printf("TRADER: Added %d bid offers from %s to supply %d units of %s to %s | bids = %d, offers = %d",
            offersAdded, bidder:getName(), count, item:getName(), self.parent:getName(), data.totalBid, #data.bidOffers)
    else
        printf("TRADER ***: Couldn't add any bid offers from %s to supply %d units of %s to %s | bids = %d, offers = %d",
            bidder:getName(), count, item:getName(), self.parent:getName(), data.totalBid, #data.bidOffers)
    end

    return offersAdded
end

-- Return the maximum profitable volume and corresponding total profit from
-- buying item here and selling at destination
function BlackMarketTrader:computeTrade(item, maxCount, dst, asset)
    local src = self
    local srcData = src:getData(item)
    local dstData = dst:getData(item)
    local asks = srcData.asks
    local bids = dstData.bids

    -- Only consider as many asks as remain unreserved
    local assetAsks = 0
    if asset then
        assetAsks = BlackMarketTrader:countAskOffers(srcData, asset)
    end
    local otherAsks = #srcData.askOffers - assetAsks
    local asksFree = srcData.totalAsk - otherAsks

    -- Only consider as many bids as remain unreserved
    local assetBids = 0
    if asset then
        assetBids = BlackMarketTrader:countBidOffers(dstData, asset)
    end
    local otherBids = #dstData.bidOffers - assetBids
    local bidsFree = dstData.totalBid - otherBids

    --local aname = "-"
    --if asset then aname = asset:getName() end
    --printf("computeTrade %s: item %s from station %s (asks %d) -> station %s (bids %d); " ..
    -- "maxCount %d, assetBids %d, otherBids %d, bidsFree %d",
    -- aname, item:getName(), src.parent:getName(), #asks, dst.parent:getName(), #bids,
    -- maxCount, assetBids, otherBids, bidsFree)

    local count = 0
    local profit = 0
    while count < maxCount and count < asksFree and count < bidsFree do
        local ask = asks[count + 1]
        local bid = bids[count + 1]
        if not ask or not bid or ask >= bid then break end
        profit = profit + (bid - ask)
        count = count + 1
    end

    return count, profit
end

function BlackMarketTrader:getAskVolume(item)
    local data = self:getData(item)
    return #data.asks + #data.asksQueue - #data.askOffers
end

function BlackMarketTrader:getAskVolumeForAsset(item, asset)
    local data = self:getData(item)
    return BlackMarketTrader:countAskOffers(data, asset)
end

function BlackMarketTrader:countAskOffers(data, asset)
    local askOfferCount = 0

    for i, assetBidder in ipairs(data.askOffers) do
        if assetBidder == asset then
            askOfferCount = askOfferCount + 1
        end
    end

    return askOfferCount
end

local function findAskOffer(data, asset)
    local askOfferIndex = -1

    for i, assetBidder in ipairs(data.askOffers) do
        if assetBidder == asset then
            askOfferIndex = i
            break
        end
    end

    return askOfferIndex
end

local function removeAskOffer(data, asset)
    local askOfferIndex = findAskOffer(data, asset)
    if askOfferIndex ~= -1 then
        --printf("TRADER: removing 1 ask offer from %s", asset:getName())
        remove(data.askOffers, askOfferIndex)
    end
end

function BlackMarketTrader:getBidVolume(item)
    local data = self:getData(item)
    return #data.bids + #data.bidsQueue - #data.bidOffers
end

function BlackMarketTrader:getBidVolumeForAsset(item, asset)
    local data = self:getData(item)
    return BlackMarketTrader:countBidOffers(data, asset)
end

function BlackMarketTrader:countBidOffers(data, asset)
    local bidOfferCount = 0

    for i, assetBidder in ipairs(data.bidOffers) do
        if assetBidder == asset then
            bidOfferCount = bidOfferCount + 1
        end
    end

    return bidOfferCount
end

local function findBidOffer(data, asset)
    local bidOfferIndex = -1

    for i, assetBidder in ipairs(data.bidOffers) do
        if assetBidder == asset then
            bidOfferIndex = i
            break
        end
    end

    return bidOfferIndex
end

local function removeBidOffer(data, asset)
    local bidOfferIndex = findBidOffer(data, asset)
    if bidOfferIndex ~= -1 then
        --printf("TRADER: removing 1 bid offer from %s", asset:getName())
        remove(data.bidOffers, bidOfferIndex)
    end
end

function BlackMarketTrader:getBuyFromPrice(item, count)
    -- Price the trader is asking to receive for any asset to buy "count" units of this item
    local price = 0
    local data = self:getData(item)
    local asks = data.asks
    local maxCount = math.min(count, #asks - (#data.askOffers or 0))

    for i = 1, maxCount do
        price = price + (asks[i] or 0)
    end

    -- No price for a valid unit of an item should ever be less than 1 credit
    if maxCount > 0 then
        price = math.max(1, price)
    end

    --printf("TRADER %s - BuyFromPrice (%s): #data.asks = %d, data.escrow = %d, data.askOffers = %d, " ..
    -- "count = %d, maxCount = %d, price = %d",
    -- self.parent:getName(), item:getName(), #data.asks, data.escrow, data.askOffers, count, maxCount, price)

    return price
end

function BlackMarketTrader:getBuyFromPriceForAsset(item, count, asset)
    -- Price the trader is asking to receive for a particular asset to buy "count" units of this item
    local price = 0
    local data = self:getData(item)
    local asks = data.asks

    local otherAsks = #data.askOffers - BlackMarketTrader:countAskOffers(data, asset)
    local asksFree = data.totalAsk - otherAsks
    local maxCount = math.min(count, asksFree)

    for i = 1, maxCount do
        price = price + (asks[i] or 0)
    end

    -- No price for a valid unit of an item should ever be less than 1 credit
    if maxCount > 0 then
        price = math.max(1, price)
    end

    --printf("TRADER %s - BuyFromPriceForAsset (%s): #data.asks = %d, data.escrow = %d, data.askOffers = %d, " ..
    -- "count = %d, maxCount = %d, price = %d",
    -- self.parent:getName(), item:getName(), #data.asks, data.escrow, data.askOffers, count, maxCount, price)

    return price
end

function BlackMarketTrader:getSellToPrice(item, count)
    -- Price the trader is bidding to pay to buy "count" units of this item from anyone
    local price = 0
    local data = self:getData(item)
    local bids = data.bids
    local maxCount = math.min(count, #bids - (#data.bidOffers or 0))

    for i = 1, maxCount do
        price = price + (bids[i] or 0)
    end

    -- No price for a valid unit of an item should ever be less than 1 credit
    if maxCount > 0 then
        price = math.max(1, price)
    end

    --printf("TRADER %s - SellToPrice (%s): #data.bids = %d, data.bidOffers = %d, count = %d, maxCount = %d, price = %d",
    --self.parent:getName(), item:getName(), #data.bids, #data.bidOffers, count, maxCount, price)

    return price
end

function BlackMarketTrader:getSellToPriceForAsset(item, count, asset)
    -- Price the trader is bidding to pay to buy "count" units of this item from a particular asset
    local price = 0
    local data = self:getData(item)
    local bids = data.bids

    local otherBids = #data.bidOffers - BlackMarketTrader:countBidOffers(data, asset)
    local bidsFree = data.totalBid - otherBids
    local maxCount = math.min(count, bidsFree)

    for i = 1, maxCount do
        price = price + (bids[i] or 0)
    end

    -- No price for a valid unit of an item should ever be less than 1 credit
    if maxCount > 0 then
        price = math.max(1, price)
    end

    --printf("TRADER %s - SellToPriceForAsset (%s): #data.bids = %d, data.bidOffers = %d, count = %d, maxCount = %d, price = %d",
    --self.parent:getName(), item:getName(), #data.bids, #data.bidOffers, count, maxCount, price)

    return price
end

function BlackMarketTrader:buy(asset, item)
    -- Trader buys item FROM Asset
    -- NOTE: ships carry the _item_, but their owners hold the _money_
    local rng = self.parent.parent.rng
    local madePurch = false
    local player = asset:getOwner()
    local data = self:getData(item)

    if #data.bids > 0 then
        local price = data.bids[1]

        if self.parent:hasCredits(price) then
            if self.parent:mgrInventoryGetFree() >= item:getMass() then
                if asset:removeItem(item, 1) then
                    self.parent:addItem(item, 1)

                    self.parent:removeCredits(price)
                    asset:addCredits(price)

                    --printf("BUY: Trader parent %s buys 1 unit of item %s from Asset %s (Owner %s) at price %d",
                    --self.parent:getName(), item:getName(), asset:getName(), player:getName(), price)

                    data.totalBid = data.totalBid - 1
                    if data.totalBid < 0 then data.totalBid = 0 end
                    data.totalBidPrice = data.totalBidPrice - price
                    if data.totalBidPrice < 0 then data.totalBidPrice = 0 end

                    removeBidOffer(data, asset)

                    remove(data.bids, 1)

                    madePurch = true
                end
            end
        end
    end

    return madePurch
end

function BlackMarketTrader:sell(asset, item)
    -- Trader sells item TO Asset
    -- NOTE: ships carry the _item_, but their owners hold the _money_
    local rng = self.parent.parent.rng
    local madeSale = false
    local player = asset:getOwner()
    local data = self:getData(item)

    if #data.asks > 0 then
        assert(data.escrow > 0)

        local price = data.asks[1]
        if price > 0 and player:hasCredits(price) then
            if asset:getInventoryFree() >= item:getMass() then
                -- Note that we don't have to remove the item from the trader's owner; that was
                -- done when the ask was made and the escrow count was incremented
                asset:addItem(item, 1)
                --printf("SELL: Trader parent %s sells 1 unit of item %s to Asset %s (Owner %s) at price %d",
                -- self.parent:getName(), item:getName(), asset:getName(), player:getName(), price)

                --printf("Trader %s now has %d units of item %s",
                -- self.parent:getName(), self.parent:getItemCount(item), item:getName())

                asset:removeCredits(price)
                self.parent:addCredits(price)

                data.totalAsk = data.totalAsk - 1
                if data.totalAsk < 0 then data.totalAsk = 0 end
                data.totalAskPrice = data.totalAskPrice - price
                data.escrow = data.escrow - 1
                if data.escrow < 0 then data.escrow = 0 end

                removeAskOffer(data, asset)

                remove(data.asks, 1)

                madeSale = true
            end
        end
    end

    return madeSale
end

local function sortAsks(a, b)
    -- Trader wants to sell (through asks) at the highest price
    return a > b
end

local function sortBids(a, b)
    -- Trader wants to buy (through bids) at the lowest price
    return a < b
end

local bid = false

function BlackMarketTrader:update()
    if not GameState.paused then
        Profiler.Begin("Trader.update")
        local rng = self.parent.parent.rng

        if not bid then
            self:addBid(Item.Energy, 10000000)
            bid = true
        end

        for item, data in pairs(self.elems) do
            -- Move asks from asks queue to asks table
            if #data.asksQueue > 0 then
                for i, v in ipairs(data.asksQueue) do insert(data.asks, v) end
                table.clear(data.asksQueue)
                table.sort(data.asks, sortAsks)
            end

            -- Move bids from bids queue to bids table
            if #data.bidsQueue > 0 then
                for i, v in ipairs(data.bidsQueue) do insert(data.bids, v) end
                table.clear(data.bidsQueue)
                table.sort(data.bids, sortBids)
            end
        end
        Profiler.End()
    end
end

--------------------------------------------------------------------------------

function Entity:addBlackMarketTrader()
    assert(not self.blackMarketTrader)
    self.blackMarketTrader = BlackMarketTrader(self)
    self:register(OldEvent.Debug, Entity.debugBlackMarketTrader)
    self:register(OldEvent.Update, Entity.updateBlackMarketTrader)

    return self.blackMarketTrader
end

function Entity:debugBlackMarketTrader(state)
    local ctx = state.context
    ctx:text("Trader")
    ctx:indent()
    ctx:text("Credits: %d", self:getCredits())
    for item, data in pairs(self.blackMarketTrader.elems) do
        if #data.bids > 0 or #data.asks + data.escrow > 0 then
            ctx:text("%s", item:getName())
            ctx:indent()
            if #data.bids > 0 then
                ctx:text("[BID] Vol: %d (%d)  Hi: %d", #data.bids, #data.bidOffers, data.bids[1])
            end
            if #data.asks + data.escrow > 0 then
                if data.asks then
                    ctx:text("[ASK] Vol: %s (%s)  Lo: %s", #data.asks, #data.askOffers, data.asks[1])
                else
                    ctx:text("[ASK ***] 0 asks, %d ask offers!!!", #data.askOffers)
                    printf("TRADER **** - bad alignment; trader %s has 0 asks, %d ask offers", self:getName(),
                        #data.askOffers)
                end
            end
            ctx:undent()
        end
    end
    ctx:undent()
end

function Entity:getBlackMarketTrader()
    assert(self.blackMarketTrader)
    return self.blackMarketTrader
end

function Entity:hasBlackMarketTrader()
    return self.blackMarketTrader ~= nil
end

function Entity:removeBlackMarketTrader()
    assert(self.blackMarketTrader)
    self:unregister(OldEvent.Debug, Entity.debugBlackMarketTrader)
    self:unregister(OldEvent.Update, Entity.updateBlackMarketTrader)
    self.blackMarketTrader = nil
end

function Entity:updateBlackMarketTrader(state)
    self.blackMarketTrader:update(state.dt)
end

--------------------------------------------------------------------------------
