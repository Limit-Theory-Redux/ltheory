-- TODO : Seems like there's a unification opportunity here as well. Certainly
--        integrity and capacitor should be unified: all the 'energies' of an
--        entity should be stored in a kind of 'energy inventory' -- to be used
--        by shields, armor, hull, capacitor, and anything else that uses a
--        similar 'virtual currency'
-- NOTE: The above "TODO" came from Josh -- we may want to rethink it. It makes sense
--           that things using energy -- energy weapons, shield generators -- should
--           take that energy from a capacitor. But physical objects, such as hull and
--           armor, don't need to take energy and should have their own "health" values.

local Entity = require('GameObjects.Entity')
local Bindings = require('States.ApplicationBindings')

function Entity:addShield (max, rate)
  assert(not self.shield)
  assert(max)
  assert(rate)
  self.shield = floor(max)
  self.shieldMax = floor(max)
  self.shieldRate = rate
  if self.shieldMax > 1e-6 then
    self:register(Event.Update, Entity.updateShield)
  end
end

function Entity:damageShield (value)
  if not self.shield then return false end
  if self.shield - value < 1e-6 then
    self.shield = 0.0
    return false
  end
  self.shield = self.shield - value
  UI.DrawEx.Ring(200, 200, 50, Config.game.shieldColor, true)
--printf("Entity %s shields take %s damage, %s remaining", self:getName(), value, self.shield)
  return true
end

function Entity:getShield ()
  assert(self.shield)
  return self.shield or 0.0
end

function Entity:getShieldMax ()
  assert(self.shield)
  return self.shieldMax or 0.0
end

function Entity:getShieldNormalized ()
  assert(self.shield)
  if not self.shield then return 0.0 end
  if self.shieldMax < 1e-6 then return 0.0 end
  return self.shield / self.shieldMax
end

function Entity:getShieldPercent ()
  assert(self.shield)
  if self.shieldMax < 1e-6 then return 0.0 end
  return 100.0 * self.shield / self.shieldMax
end

function Entity:hasShield ()
  return self.shield ~= nil
end

function Entity:setShield (value, max, rate)
  assert(self.shield)
  self.shield = value
  self.shieldMax = floor(max)
  self.shieldRate = rate
end

function Entity:updateShield (state)
  if not self:isDestroyed() then
    if self.shield < self.shieldMax then
      local timeScale = 1.0
      if GameState.paused then
        timeScale = 0.0
      end

      if Input.GetDown(Bindings.TimeAccel) then
        timeScale = GameState.debug.timeAccelFactor
      end

      self.shield = min(self.shieldMax, self.shield + (timeScale * state.dt) * self.shieldRate)
--printf("SHIELD: %s - rate = %s, curr = %s, max = %s", self:getName(), self.shieldRate, self.shield, self.shieldMax)
    end
  end
end
