local Entity = require('GameObjects.Entity')

function Entity:addHealth (max)
  assert(not self.health)
  assert(max)
  self.health = max
  self.healthMax = max
end

function Entity:damageHealth (amount)
  assert(self.health)
  if self.health <= 0 then return end
  self.health = max(0, self.health - amount)
end

function Entity:getHealth ()
  assert(self.health)
  return self.health
end

function Entity:getHealthMax ()
  assert(self.healthMax)
  return self.healthMax
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

function Entity:setHealth (value, max)
  assert(self.health)
  self.health = value
  self.healthMax = max
end
