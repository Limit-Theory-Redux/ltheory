local Entity = require('GameObjects.Entity')
local Credit = require('Systems.Economy.Item').Credit

--------------------------------------------------------------------------------

-- NOTE: All the evaluations below are made from the perspective of what is most
--       advantageous to _this trader_ (self).

local Trader = class(function (self, parent)
  self.parent = parent
  self.elems = {}
end)

function Trader:getData (item)
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
    }
  end
  return self.elems[item]
end

function Trader:addAsk (item, price)
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

function Trader:addBid (item, price)
  local data = self:getData(item)

  data.totalBid = data.totalBid + 1
  data.totalBidPrice = data.totalBidPrice + price

  insert(data.bidsQueue, price)
--printf("Added bid to buy item %s at station %s for price %d", item:getName(), self.parent:getName(), price)
  return true
end

-- Return the maximum profitable volume and corresponding total profit from
--     buying item here and selling at destination
function Trader:computeTrade (item, maxCount, dst)
  local src = self

  local asks = src:getData(item).asks
  local bids = dst:getData(item).bids
  local count = 0
  local profit = 0

--printf("#asks = %d, #bids = %d", #asks, #bids)
--for i = 1, #bids do
--  printf("bids[%d] = %d", i, bids[i])
--end

  local i = 1
  while count < maxCount do
    local ask = asks[i]
    local bid = bids[i]
--if ask and bid then
--printf("item %s from station %s, ask = %d to station %s, bid = %d : profit = %d at count %d",
--item:getName(), src.parent:getName(), ask, dst.parent:getName(), bid, profit, count)
--end
    if not ask or not bid or ask >= bid then break end
    count = count + 1
    profit = profit + (bid - ask)
    i = i + 1
  end

  return count, profit
end

function Trader:getAskVolume (item)
  local data = self:getData(item)
  return #data.asks + #data.asksQueue
end

function Trader:getBidVolume (item)
  local data = self:getData(item)
  return #data.bids + #data.bidsQueue
end

function Trader:getBuyFromPrice (item, count)
  -- Price the trader is asking to receive to buy "count" units of this item from it
  local price = 0
  local asks = self:getData(item).asks
  for i = 1, count do
    price = price + (asks[i] or 0)
--    price = price + (asks[i] or math.huge)
  end
  return price
end

function Trader:getSellToPrice (item, count)
  -- Price the trader is bidding to pay to buy "count" units of this item from anyone
  local price = 0
  local bids = self:getData(item).bids
  for i = 1, count do
    price = price + (bids[i] or 0)
  end
  return price
end

function Trader:buy (asset, item)
  -- Trader buys item FROM Asset
  -- NOTE: ships carry the _item_, but their owners hold the _money_
  local rng = self.parent.parent.rng
  local madePurch = false
  local player = asset:getOwner()
  local data = self:getData(item)

  if #data.bids > 0 then
    local price = data.bids[1]

    if self.parent:hasCredits(price) then
      if self.parent:getInventoryFree() >= item:getMass() then
        if asset:removeItem(item, 1) then
          self.parent:addItem(item, 1)

          self.parent:removeCredits(price)
          player:addCredits(price)

--printf("BUY: Trader parent %s buys 1 unit of item %s from Asset %s (Owner %s) at price %d",
--self.parent:getName(), item:getName(), asset:getName(), player:getName(), price)

          data.totalBid = data.totalBid - 1
          data.totalBidPrice = data.totalBidPrice - price

          remove(data.bids, 1)

          madePurch = true
        end
      end
    end
  end

  return madePurch
end

function Trader:sell (asset, item)
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
        --     done when the ask was made and the escrow count was incremented
        asset:addItem(item, 1)
--printf("SELL: Trader parent %s sells 1 unit of item %s to Asset %s (Owner %s) at price %d",
--    self.parent:getName(), item:getName(), asset:getName(), player:getName(), price)

--printf("Trader %s now has %d units of item %s",
--    self.parent:getName(), self.parent:getItemCount(item), item:getName())

        player:removeCredits(price)
        self.parent:addCredits(price)
--printf("removed %d credits from %s (now has %d credits)", price, self.parent:getName(), self.parent:getCredits())

        data.totalAsk = data.totalAsk - 1
        data.totalAskPrice = data.totalAskPrice - price
        data.escrow = data.escrow - 1

        remove(data.asks, 1)

        madeSale = true
      end
    end
  end

  return madeSale
end

local function sortAsks (a, b)
  -- Trader wants to sell (through asks) at the highest price
  return a > b
end

local function sortBids (a, b)
  -- Trader wants to buy (through bids) at the lowest price
  return a < b
end

function Trader:update ()
  if not Config.game.gamePaused then
    local rng = self.parent.parent.rng
    for item, data in pairs(self.elems) do
      -- Move asks from asks queue to asks table
      if #data.asksQueue > 0 then
        for i, v in ipairs(data.asksQueue) do insert(data.asks, v) end
        table.clear(data.asksQueue)
        table.sort(data.asks, sortAsks)
--for i = 1, #data.asks do
--  printf("ask[%d] = %d", i, data.asks[i])
--end
      end

      -- Move bids from bids queue to bids table
      if #data.bidsQueue > 0 then
        for i, v in ipairs(data.bidsQueue) do insert(data.bids, v) end
        table.clear(data.bidsQueue)
        table.sort(data.bids, sortBids)
--for i = 1, #data.bids do
--  printf("bid[%d] = %d", i, data.bids[i])
--end
      end

      -- Possibly decrease ask to increase chance that someone will sell this item to the trader
      if rng:getInt(0, 100) < 1 then
        for i = 1, #data.asks do
          data.asks[i] = math.max(1, data.asks[i] - 1) -- lower price on all asks for this item
        end
      end

      -- Possibly increase bid to increase chance that someone will buy this item from the trader
      if rng:getInt(0, 100) < 1 then
        local raisedPrice = 1
        if rng:getInt(0, 100) < 2 then
          raisedPrice = rng:getInt(5, 30) -- rare windfall
        end
        if self.parent:hasCredits(data.totalBidPrice + raisedPrice * data.totalBid) then
          -- Trader can cover the increased price
          for i = 1, #data.bids do
            data.bids[i] = data.bids[i] + raisedPrice -- raise price on all bids for this item
          end
        end
      end
    end
  end
end

--------------------------------------------------------------------------------

function Entity:addTrader ()
  assert(not self.trader)
  self.trader = Trader(self)
  self:register(Event.Debug, Entity.debugTrader)
  self:register(Event.Update, Entity.updateTrader)

  return self.trader
end

function Entity:debugTrader (state)
  local ctx = state.context
  ctx:text('Trader')
  ctx:indent()
  ctx:text('Credits: %d', self:getCredits())
  for item, data in pairs(self.trader.elems) do
    if #data.bids > 0 or #data.asks > 0 then
      ctx:text('%s', item:getName())
      ctx:indent()
      if #data.bids > 0 then
        ctx:text('[BID] Vol: %d  Hi: %d', #data.bids, data.bids[1])
      end
      if #data.asks > 0 then
        ctx:text('[ASK] Vol: %d  Lo: %d', #data.asks, data.asks[1])
      end
      ctx:undent()
    end
  end
  ctx:undent()
end

function Entity:getTrader ()
  assert(self.trader)
  return self.trader
end

function Entity:hasTrader ()
  return self.trader ~= nil
end

function Entity:updateTrader (state)
  self.trader:update(state.dt)
end

--------------------------------------------------------------------------------
