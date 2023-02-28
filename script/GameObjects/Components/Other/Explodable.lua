local Entity = require('GameObjects.Entity')

local rng = RNG.Create(1231)

local function explode (self, source)
  if self:getOwner() then self:getOwner():removeAsset(self) end

  Config.game.explosionSize = self.explosionSize
  local root = self:getRoot()
  for i = 1, 8 do
    local p = self:getPos() + rng:getSphere():scale(8.0 * self:getScale() * rng:getExp() ^ (1.0 / 3.0))
    local v = self:getVelocity()
    local e = Entities.Effects.Explosion(p, v, min(0.0, 0.5 - rng:getExp()))
    root:addChild(e)
  end

  self:clearActions()
  self.explodable = false
--printf("%s exploded! self.explodable = %s, hasExplo = %s, isExplo = %s",
--self:getName(), self.explodable, self:hasExplodable(), self:isExplodable())
end

function Entity:addExplodable ()
  assert(not self.explodable)
  self.explodable = true
--printf("Explodable: %s: self.explodable = %s, self:hasExplodable() = %s",
--self:getName(), self.explodable, self:hasExplodable())

  self.explosionSize = 64

  self:register(Event.Destroyed, explode)
end

function Entity:hasExplodable ()
  if self == nil or self.explodable == nil then
    return false
  else
    return true
  end
end

function Entity:isExplodable ()
  if self == nil or self.explodable == nil then
    return false
  else
    return self.explodable
  end
end
