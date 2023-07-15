local Entity = require('GameObjects.Entity')

<<<<<<< HEAD
local Zone = subclass(Entity, function(self, name)
    self.name = name
    self.threatLevel = 0
    self.lastThreatTime = 0
    self.threatReductionPerUpdate = 5
    self.threatReductionUpdateTime = 10
    self.dt = 0
    self:addChildren()
    self:register(Event.Update, self.updateZone)
end)

function Zone:add(e)
    insert(self.children, e)
end

function Zone:getChildren()
    return self.children
end

function Zone:getName()
    return self.name
=======
local Zone = subclass(Entity, function (self, name)
  self.name = name
  self.threatLevel = 0
  self.lastThreatTime = 0
  self.threatReductionPerUpdate = 5
  self.threatReductionUpdateTime = 10
  self.dt = 0
  self:addChildren()
  self:register(Event.Update, self.updateZone)
end)

function Zone:getName ()
  return self.name
>>>>>>> 1b58bb0278295d31845972084d1313877cd21e29
end

function Zone:getPos()
    return self.pos
end

function Zone:getExtent()
    return self.extent
end

<<<<<<< HEAD
function Zone:setExtent(extent)
    -- "extent" is a scalar radius for a spherical volume
    self.extent = extent
    self.trigger = Entities.Trigger(Vec3f(extent/2, extent/2, extent/2))
    self.trigger:triggerSetPos(self:getPos())
    GameState.world.currentSystem:addChild(self.trigger)
end

function Zone:sample(rng)
    return rng:choose(self.children)
=======
function Zone:setExtent (extent)
  -- "extent" is a scalar radius for a spherical volume
  self.extent = extent
  self.trigger = Entities.Trigger(Vec3f(extent/2, extent/2, extent/2))
  self.trigger:triggerSetPos(self:getPos())
  GameState.world.currentSystem:addChild(self.trigger)
end

function Zone:adjustThreat(amount)
  self:setThreat(self.threatLevel + amount)
  self.lastThreatTime = self.dt
end

function Zone:setThreat(amount)
  self.threatLevel = max(0, floor(amount * 100) / 100)
  self.lastThreatTime = self.dt
end

function Zone:getThreat()
  return self.threatLevel
end


function Zone:sample (rng)
  return rng:choose(self.children)
>>>>>>> 1b58bb0278295d31845972084d1313877cd21e29
end

function Zone:getRandomPos(rng)
    return self.pos + rng:getDir3():scale((0.1 * self.extent) * rng:getExp() ^ rng:getExp())
end

function Zone:updateZone(state)
    if self.dt > self.lastThreatTime + self.threatReductionUpdateTime then
        if self.threatLevel > 0 then
            self:adjustThreat(-self.threatReductionPerUpdate)
        end
        self.lastThreatTime = self.dt
    end

    self.dt = self.dt + state.dt
end

function Zone:updateZone(state)
  if self.dt > self.lastThreatTime + self.threatReductionUpdateTime then
    if self.threatLevel > 0 then
      self:adjustThreat(-self.threatReductionPerUpdate)
    end
    self.lastThreatTime = self.dt
  end

  self.dt = self.dt + state.dt
end

return Zone
