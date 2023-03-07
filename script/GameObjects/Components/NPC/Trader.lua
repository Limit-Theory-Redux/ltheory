local Entity = require('GameObjects.Entity')
local Credit = require('Systems.Economy.Item').Credit

--------------------------------------------------------------------------------

local Trader = class(function (self, parent)
  self.parent = parent
  self.credits = 0
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
  local data = self:getData(item)
  if not self.parent:removeItem(item, 1) then return false end
  data.escrow = data.escrow + 1
  insert(data.asksQueue, price)
  return true
end

function Trader:addAsk (item, price)
  local data = self:getData(item)
  if not self.parent:removeItem(item, 1) then return false end
  data.escrow = data.escrow + 1
  insert(data.asks, price)
--  insert(data.asksQueue, price) -- TODO: test if this is a good change
  return true
end

function Trader:addBid (item, price)
  local data = self:getData(item)
  -- TODO : Remove credits
  insert(data.bids, price)
--  insert(data.bidsQueue, price) -- TODO: test if this is a good change
  return true
end

-- Return the maximum profitable volume and corresponding total profit from
-- buying item here and selling at dst
function Trader:computeTrade (item, maxCount, dst)
  local src = self

  local asks = src:getData(item).asks
  local bids = dst:getData(item).bids
  local count = 0
  local profit = 0

  local i = 1
  while count < maxCount do
    local ask = asks[i]
    local bid = bids[i]
    if not ask or not bid or ask >= bid then break end
    count = count + 1
    profit = profit + (bid - ask)
    i = i + 1
  end

  return count, profit
end

function Trader:getBidVolume (item)
  local data = self:getData(item)
  return #data.bids + #data.bidsQueue
end

function Trader:getAskVolume (item)
  local data = self:getData(item)
  return #data.asks + #data.asksQueue
end

function Trader:getBuyFromPrice (item, count)
  local price = 0
  local asks = self:getData(item).asks
  for i = 1, count do
    price = price + (asks[i] or math.huge)
  end
  return price
end

function Trader:getSellToPrice (item, count)
  local price = 0
  local bids = self:getData(item).bids
  for i = 1, count do
    price = price + (bids[i] or 0)
  end
  return price
end

function Trader:buy (asset, item)
  local player = asset:getOwner()
  local data = self:getData(item)
  if #data.asks == 0 then return false end
  if not player:hasCredits(0) then return false end -- make sure asset's owning player isn't broke

  local price = data.asks[1]
  assert(data.escrow > 0)
  if not player:hasItem(Credit, price) then return false end

--printf("BUY: Asset %s (owner %s) buys 1 unit of item %s from trader %s at price %d",
--asset:getName(), player:getName(), item:getName(), self.parent:getName(), price)

  asset:addItem(item, 1)
  player:removeItem(Credit, price)
  self.parent:addCredits(price)
--  self.credits = self.credits + price
  data.totalAsk = data.totalAsk + 1
  data.totalAskPrice = data.totalAskPrice + price
  data.escrow = data.escrow - 1
  data.asks[1] = (data.asks[1] + 1) or math.huge -- update bid price
--  remove(data.asks, 1)
  return true
end

function Trader:sell (asset, item)
  local rng = self.parent.parent.rng
  local madeSale = false

  local player = asset:getOwner()
  local data = self:getData(item)
  if #data.bids > 0 then
    if not asset:hasItem(item, 1) then return false end

    local price = data.bids[1]
    if not self.parent:hasCredits(price) then return false end -- no deal if trader is broke
    if price > 0 then

--printf("SELL: Asset %s (owner %s) sells 1 unit of item %s to trader %s at price %d",
--asset:getName(), player:getName(), item:getName(), self.parent:getName(), price)

      asset:removeItem(item, 1)
      player:addItem(Credit, price)
      self.parent:removeCredits(price)
--      self.credits = self.credits - price
--printf("removed %d credits from %s (now has %d credits)", price, self.parent:getName(), self.parent:getCredits())
      data.totalBid = data.totalBid + 1
      data.totalBidPrice = data.totalBidPrice + price
      self.parent:addItem(item, 1)
      if rng:getInt(0, 100) < 50 then
        data.bids[1] = math.max(0, data.bids[1] - 1) -- possibly update bid price
--      remove(data.bids, 1)
      end

      madeSale = true
    end
  end

  return madeSale
end

local function sortAsks (a, b)
  return a < b
end

local function sortBids (a, b)
  return a > b
end

function Trader:update ()
  local rng = self.parent.parent.rng

  for item, data in pairs(self.elems) do
    for i = 1, #data.bids do
      if rng:getInt(0, 100) < 5 then
        -- Possibly increase bid (so that there's always some interest in buying something)
        -- TODO: Connect this to any nearby Factory so that bid prices depend on the factory's desired inputs
        local raisedPrice = 0
        if rng:getInt(0, 100) < 10 then
          raisedPrice = rng:getInt(5, 30) -- rare windfall
        else
          raisedPrice = 1 -- restore bid price
        end
        if raisedPrice > 0 then
          if self.parent:hasCredits(raisedPrice) then
            data.bids[1] = data.bids[1] + math.max(0, raisedPrice or math.huge) -- change bid price (within limits)
--            printf("%s has %d credits (sale = %s) and raised bid for %s by %d to %d",
--                self.parent:getName(), self.parent:getCredits(), madeSale, item:getName(), raisedPrice, data.bids[1])
--          else
--            printf("%s has %d credits (sale = %s) and can't raise bid for %s by %d",
--                self.parent:getName(), self.parent:getCredits(), madeSale, item:getName(), raisedPrice)
          end
        end
      end
    end

    if #data.asksQueue > 0 then
      for i, v in ipairs(data.asksQueue) do insert(data.asks, v) end
      table.clear(data.asksQueue)
      table.sort(data.asks, sortAsks)
    end

    if #data.bidsQueue > 0 then
      for i, v in ipairs(data.bidsQueue) do insert(data.bids, v) end
      table.clear(data.bidsQueue)
      table.sort(data.bids, sortBids)
    end
  end
end

--------------------------------------------------------------------------------

function Entity:addTrader ()
  assert(not self.trader)
  self.trader = Trader(self)
  self:register(Event.Debug, Entity.debugTrader)
  self:register(Event.Update, Entity.updateTrader)

printf("Added Trader to %s", self:getName())

  return self.trader
end

function Entity:debugTrader (state)
  local ctx = state.context
  ctx:text('Trader')
  ctx:indent()
  ctx:text('Credits: %d', self:getCredits())
--  ctx:text('Credits: %d', self.trader.credits)
  for item, data in pairs(self.trader.elems) do
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
