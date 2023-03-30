local Entity = require('GameObjects.Entity')

-- NOTE : The 'size' of an entity's yield serves as a rate limiter. No more
--        than 'size' (energy-normalized) units of item may be extracted per
--        unit time from the Entity.
local Yield = class(function (self, item, size)
  self.item     = item
  self.size     = size
  self.maxSize  = size
  self.cooldown = 300 -- ore yields start to respawn after 5 minutes after last mining
  self.duration =   5 -- ore yields respawn one unit every 5 seconds
  self.ctimer   = 0.0
  self.dtimer   = 0.0
end)

function Yield:update (dt)
  -- Gradually increase yield by 1 after a cooldown timer expires
  if not Config.game.gamePaused then
    Profiler.Begin("Yield.update")
    if self.size < self.maxSize then
      self.ctimer = self.ctimer + dt
      if self.ctimer >= self.cooldown then
        self.dtimer = self.dtimer + dt
        if self.dtimer >= self.duration then
          self.size = self.size + 1
          self.dtimer = 0
        end
      end
    else
      if self.ctimer > 0 then
        self.ctimer = 0
      end
    end
    Profiler.End()
  end
end

--------------------------------------------------------------------------------

function Entity:addYield (item, size)
  assert(not self.yield)
  self.yield = Yield(item, size)
  self:register(Event.Debug, Entity.debugYield)
  self:register(Event.Update, Entity.updateYield)
end

function Entity:debugYield (state)
  local ctx = state.context
  ctx:text("Yield")
  ctx:indent()
  ctx:text("Item: %s", self.yield.item:getName())
  ctx:text("Size: %d, Max: %d", self.yield.size, self.yield.maxSize)
  ctx:undent()
end

function Entity:decreaseYield ()
  assert(self.yield)
  local yieldDecreased = false
  if self.yield.size > 0 then
    self.yield.size = self.yield.size - 1
    self.yield.ctimer = 0 -- reset cooldown timer for restoring item
    yieldDecreased = true
  end

  return yieldDecreased
end

function Entity:getYield ()
  assert(self.yield)
  return self.yield
end

function Entity:getYieldMax ()
  assert(self.yield)
  return self.yield.maxSize
end

function Entity:getYieldSize ()
  assert(self.yield)
  return self.yield.size
end

function Entity:hasYield ()
  return self.yield ~= nil
end

function Entity:increaseYield ()
  assert(self.yield)
  if self.yield.size < self:getYieldMax() then
    self.yield.size = self.yield.size + 1
  end
end

function Entity:setYield (yieldSize)
  assert(self.yield)
  self.yield.size = yieldSize
end

function Entity:updateYield (state)
  self.yield:update(state.dt)
end

--------------------------------------------------------------------------------
