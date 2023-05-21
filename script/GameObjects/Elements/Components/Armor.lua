local Entity = require('GameObjects.Entity')
local Bindings = require('States.ApplicationBindings')

function Entity:addArmor (max)
  assert(not self.armor)
  assert(max)
  self.armor = floor(max)
  self.armorMax = floor(max)
end

function Entity:damageArmor (value)
  if not self.armor then return false end
  if self.armor - value < 1e-6 then
    self.armor = 0.0
    return false
  end
  self.armor = self.armor - value
--printf("Entity %s armor takes %s damage, %s remaining", self:getName(), value, self.armor)
  return true
end

function Entity:getArmor ()
  assert(self.armor)
  return self.armor or 0.0
end

function Entity:getArmorMax ()
  assert(self.armor)
  return self.armorMax or 0.0
end

function Entity:getArmorNormalized ()
  assert(self.armor)
  if not self.armor then return 0.0 end
  if self.armorMax < 1e-6 then return 0.0 end
  return self.armor / self.armorMax
end

function Entity:getArmorPercent ()
  assert(self.armor)
  if self.armorMax < 1e-6 then return 0.0 end
  return 100.0 * self.armor / self.armorMax
end

function Entity:hasArmor ()
  return self.armor ~= nil
end

function Entity:setArmor (value, max)
  assert(self.armor)
  self.armor = value
  self.armorMax = floor(max)
end
