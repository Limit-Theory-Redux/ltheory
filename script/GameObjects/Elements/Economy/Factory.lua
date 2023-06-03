local Entity = require('GameObjects.Entity')
local Item = require('Systems.Economy.Item')

--------------------------------------------------------------------------------

local Factory = class(function (self, parent)
  self.parent = parent
  self.prods = {}
  self.time = 0
  self.timeOnline = 0
end)

function Factory:addProduction (type)
  local prod = {
    type = type,
    t = 0,
    active = false,
    blocked = false,
    askTimers = {},
    bidTimers = {},
  }

  insert(self.prods, prod)
  local duration = type:getDuration()

  do -- Update flows based on inputs / outputs of this production
    for _, input in type:iterInputs() do
      local rate = -input.count / duration
      self.parent:modFlow(input.item, rate)
      insert(prod.bidTimers, { item = input.item, value = 0, max = duration / input.count })
    end

    for _, output in type:iterOutputs () do
      local rate = output.count / duration
      self.parent:modFlow(output.item, rate)
      insert(prod.askTimers, { item = output.item, value = 0, max = duration / output.count })
    end
  end
end

-- Is the factory stalled due to a lack of inputs or excess of outputs at the
-- moment?
function Factory:isBlocked ()
  for _, prod in ipairs(self.prods) do
    if prod.blocked then return true end
  end
  return false
end

-- Get the fraction of time that this factory has been active (i.e., not
-- blocked)
function Factory:getUptime ()
  return self.timeOnline / self.time
end

function Factory:hasProductionType (productiontype)
  -- Scan all production lines installed in this factory for one that matches the given production type
  local hasProductionType = false

  for _, prod in ipairs(self.prods) do
    if prod.type == productiontype then
      hasProductionType = true
      break
    end
  end

  return hasProductionType
end

function Factory:updateProduction (prod, dt)
  local rng = RNG.FromTime()

  if not prod.active then
    -- Check inventory for presence of required inputs
    for _, input in prod.type:iterInputs() do
      if not self.parent:mgrInventoryHasItem(input.item, input.count) then
        prod.blocked = true
        return
      end
    end

    prod.blocked = false

    -- Entity has all the necessary inputs, let's start a round of production
    for _, input in prod.type:iterInputs() do
      self.parent:mgrInventoryRemoveItem(input.item, input.count)
    end

    prod.active = true
    prod.t = prod.type.duration
  end

  if prod.active then
    prod.t = prod.t - dt
    if prod.t <= 0 then
      prod.active = false
      for i, output in prod.type:iterOutputs() do
        -- TODO : How to handle failure when a factory finishes a production
        --        for which the output inventory has insufficient capacity?
        if self.parent:mgrInventoryAddItem(output.item, output.count) then
          if output.item ~= Item.Energy then
            printf("FACTORY %s produced %d units of %s", self.parent:getName(), output.count, output.item:getName())
          end
        else
          printf("FACTORY %s produced %d units of %s but could not store them all",
              self.parent:getName(), output.count, output.item:getName())
        end
      end

      -- TODO: Make factory trade order timers work
      --       Until then, after a production run, respawn bids and asks per factory inputs and outputs
      for _, input in prod.type:iterInputs() do
        for i = 1, input.count * Config.econ.inputBacklog do
          if input.item == Item.Energy then
            -- Make sure Energy-requiring factories bid well (to spur production)
            self.parent.trader:addBid(input.item, 10 + rng:getInt(25, 100))
          elseif input.item == Item.Waste then
            -- Make sure Waste-requiring factories bid fairly well (to increase Energy supply)
            self.parent.trader:addBid(input.item, 5 + rng:getInt(5, 45))
          else
            self.parent.trader:addBid(input.item, input.item.energy * 10 + rng:getInt(1, 10)) -- TODO: do smarter bids
          end
        end
      end
      for _, output in prod.type:iterOutputs() do
        for i = 1, output.count do
          if output.item == Item.Energy or output.item == Item.WasteRad or output.item == Item.Waste then
            -- Make Energy and Waste / Radioactive Waste cheap to buy
            self.parent.trader:addAsk(output.item, output.item.energy)
          else
            -- Asking price should be at least the cost of all the inputs plus a little profit
            local askPrice = 1
            if output.item ~= Item.AnodeSludge then -- TODO: eliminate this temporary special case
              for _, input in prod.type:iterInputs() do
                askPrice = askPrice + input.item.energy * input.count
              end
            end
            askPrice = math.floor(math.max(askPrice, output.item.energy) * Config.econ.markup) * output.count
            self.parent.trader:addAsk(output.item, askPrice)
          end
        end
      end

    end
  end
end

function Factory:updateTradeOrders (prod, dt)
  -- Buy inputs and sell outputs
  -- NOTE: This is Josh's original code for re-adding bids and asks after a production run
  --       Currently it runs constantly (not just after a production run) and inconsistently
  -- TODO: Fix this so that trade orders run at the correct time and to the correct amounts
  local trader = self.parent:getTrader()
  local duration = prod.type:getDuration() -- ????: should this be used here somewhere?

  -- TODO : Intelligently compute price ranges via estimation using item
  --        intrinsic energy

  local askSlope = 0.995
  local bidSlope = 0.995
  local maxAsk = 100 / askSlope
  local maxBid = 100 / bidSlope

  -- Add new asks for the Output items this factory just produced
  for _, timer in ipairs(prod.askTimers) do
    timer.value = timer.value + dt
    if timer.value >= timer.max then
      timer.value = timer.value - timer.max
      local price = askSlope * (trader:getData(timer.item).asks[1] or maxAsk)
      trader:addAsk(timer.item, price)
    end
  end

  -- Add new bids for the Input items this factory needs for another run
  for _, timer in ipairs(prod.bidTimers) do
    timer.value = timer.value + dt
    if timer.value >= timer.max then
      timer.value = timer.value - timer.max
      local price = (trader:getData(timer.item).bids[1] or 0)
      price = maxBid - bidSlope * (maxBid - price)
      if price >= 1 then
        trader:addBid(timer.item, price)
      end
    end
  end
end

function Factory:update (dt)
  if not GameState.paused then
    Profiler.Begin('Factory.Update')
    self.time = self.time + dt
    if not self:isBlocked() then self.timeOnline = self.timeOnline + dt end

    for _, prod in ipairs(self.prods) do
      self:updateProduction(prod, dt)
    end

    -- NOTE : Disabled trade orders for the moment due to not having limits on
    --        max active orders, leading to stalling the entire game via tens
    --        of thousands of individual energy cell orders...
--    self:updateTradeOrders(prod, dt)
    Profiler.End()
  end
end

--------------------------------------------------------------------------------

function Entity:addFactory ()
  assert(not self.factory)
  self.factory = Factory(self)
  self:register(Event.Update, Entity.updateFactory)
  self:register(Event.Debug, Entity.debugFactory)
end

function Entity:addProduction (type)
  assert(self.factory)
  self.factory:addProduction(type)
end

function Entity:debugFactory (state)
  local ctx = state.context
  ctx:text('Factory')
  ctx:indent()
  ctx:text('<< %s >>', self.factory:isBlocked() and 'OFFLINE' or 'ONLINE')
  ctx:text('Uptime: %.0f%%', 100 * self.factory:getUptime())
  ctx:undent()
end

function Entity:getFactory ()
  assert(self.factory)
  return self.factory
end

function Entity:hasFactory ()
  return self.factory ~= nil
end

function Entity:removeFactory ()
  assert(self.factory)
  self:unregister(Event.Update, Entity.updateFactory)
  self:unregister(Event.Debug, Entity.debugFactory)
  self.factory = nil
end

function Entity:updateFactory (state)
  self.factory:update(state.dt)
end
