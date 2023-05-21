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

function Entity:addCapacitor (max, rate)
  assert(not self.charge)
  assert(max)
  assert(rate)
  self.charge = max
  self.chargeMax = max
  self.chargeRate = rate
  self:register(Event.Update, Entity.updateCapacitor)
end

function Entity:discharge (value)
  if not self.charge then return false end
  if self.charge < value then return false end
  self.charge = self.charge - value
--printf("Entity %s discharges %s, %s remaining", self:getName(), value, self.charge)
  return true
end

function Entity:getCharge ()
  assert(self.charge)
  return self.charge or 0
end

function Entity:getChargeMax ()
  assert(self.charge)
  return self.chargeMax or 0
end

function Entity:getChargeNormalized ()
  assert(self.charge)
  if not self.charge then return 0 end
  return self.charge / self.chargeMax
end

function Entity:getChargePercent ()
  assert(self.charge)
  return 100.0 * self.charge / self.chargeMax
end

function Entity:hasCharge ()
  return self.charge ~= nil
end

function Entity:setCharge (value, max, rate)
  assert(self.charge)
  self.charge = value
  self.chargeMax = max
  self.chargeRate = rate
end

function Entity:updateCapacitor (state)
  if not self:isDestroyed() then
    local timeScale = 1.0
    if GameState.paused then
      timeScale = 0.0
    end

    if Input.GetDown(Bindings.TimeAccel) then
      timeScale = GameState.debug.timeAccelFactor
    end

    self.charge = min(self.chargeMax, self.charge + (timeScale * state.dt) * self.chargeRate)
  end
end
