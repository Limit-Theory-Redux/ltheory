local Entity = require('GameObjects.Entity')

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

function Zone:add(e)
    insert(self.children, e)
end

function Zone:getChildren()
    return self.children
end

function Zone:getName()
    return self.name
end

function Zone:getPos()
    return self.pos
end

function Zone:getExtent()
    return self.extent
end

function Zone:setExtent(extent)
    -- "extent" is a scalar radius for a spherical volume
    self.extent = extent
    self.trigger = Entities.Trigger(Vec3f(extent / 2, extent / 2, extent / 2))
    self.trigger:triggerSetPos(self:getPos())
    GameState.world.currentSystem:addChild(self.trigger)
end

function Zone:sample(rng)
    return rng:choose(self.children)
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

function Zone:adjustThreat(value)
    self.threatLevel = math.max(0, self.threatLevel + value)
end

return Zone
