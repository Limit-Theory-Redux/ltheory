local Entity = require('GameObjects.Entity')

function Entity:addHealth (max, rate)
  assert(not self.health)
  assert(max)
  assert(rate)
  self.health = max
  self.healthMax = max
  self.healthRate = rate
  self:register(Event.Update, Entity.updateHealth)
end

function Entity:damage (amount, source)
  assert(self.health)

  if self.health <= 0 then return end

  self.health = max(0, self.health - amount)
  self:send(Event.Damaged(amount, source))

  if self.health <= 0.009999999 then
    -- Entity has been damaged to the point of destruction (0 health)
    self.health = 0
    self:clearActions()
    -- Also need to process destroyed entity's assets, including credits and cargo
    -- Also ALSO need to notify nearby ships
    --    resulting Actions may include Evade, Attack, and/or alert faction members

printf("%s destroyed by %s!", self:getName(), source:getName())
    if self:hasDockable() and self:isDockable() then
      -- If this object was dockable, make it undockable
      self:setUndockable()
    end

    self:send(Event.Destroyed(source))
  end
end

function Entity:getHealth ()
  assert(self.health)
  return self.health
end

function Entity:getHealthNormalized ()
  assert(self.health)
  return self.health / self.healthMax
end

function Entity:getHealthPercent ()
  assert(self.health)
  return 100.0 * self.health / self.healthMax
end

function Entity:hasHealth ()
  return self.health ~= nil
end

-- WARNING : Note the subtlety that isAlive and isDestroyed are NOT
--           complementary! An asteroid is not alive, but neither has it been
--           destroyed. Both 'alive' and 'destroyed' require health to be true.

function Entity:isAlive ()
  return self.health and self.health > 0
end

function Entity:isDestroyed ()
  return self.health and self.health <= 0
end

function Entity:setHealth (value, max, rate)
  assert(self.health)
  self.health = value
  self.healthMax = max
  self.healthRate = rate
end

function Entity:updateHealth (state)
  if not self:isDestroyed() then
    self.health = min(self.healthMax, self.health + state.dt * self.healthRate)
  end
end
