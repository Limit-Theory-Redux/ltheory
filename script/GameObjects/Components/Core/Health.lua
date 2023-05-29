local Entity = require('GameObjects.Entity')
local Bindings = require('States.ApplicationBindings')
local rng = RNG.FromTime()

function Entity:addHealth(max, rate)
  assert(not self.health)
  assert(max)
  assert(rate)
  self.health = max
  self.healthMax = max
  self.healthRate = rate
  self:register(Event.Update, Entity.updateHealth)
end

function Entity:damage(amount, source)
  assert(self.health)

  if self.health <= 0 then return end

  self.health = max(0, self.health - amount)
  self:send(Event.Damaged(amount, source))

  -- Treat health < 0.01 (from 0.0 - 100.0) as zero when testing for destruction
  if math.floor(self.health * 100) < 1 then
    -- Entity has been damaged to the point of destruction (0 health)
    self.health = 0
    self:clearActions()

    -- Also need to process destroyed entity's assets, including credits and cargo

    -- Also ALSO need to notify nearby ships
    --    resulting Actions may include Evade, Attack, and/or alert faction members

    -- If this object was attackable, make it unattackable
    if self:hasAttackable() then
      self:setAttackable(false)
    end

    if self:hasLight() then
      self:deleteLight(self)
    end

    local thisShipName      = self:getName()
    local attackingShipName = source:getName()
    if self.usesBoost then
      thisShipName = thisShipName .. " [Ace]"
    end
    if source.usesBoost then
      attackingShipName = attackingShipName .. " [Ace]"
    end

    for item, count in pairs(self:getInventory()) do
      if rng:getInt(0, 100) < 100 then
        source:addItem(item, count)
        self:removeItem(item, count)
        print("Transported item!")
      end
    end
    source:addCredits(self:getCredits() * 0.25)

    printf("%s destroyed by %s!", thisShipName, attackingShipName)

    -- Remove destroyed ship from system's list of active ships
    for i, ship in ipairs(GameState.world.currentSystem.ships) do
      if ship == self then
        remove(GameState.world.currentSystem.ships, i)
      end
    end

    -- Any active ship still targeting this destroyed ship should lose it as a current target
    for _, ship in ipairs(GameState.world.currentSystem.ships) do
      if ship:getTarget() == self then
        ship:setTarget(nil)
      end
    end

    self:send(Event.Destroyed(source))

    -- Remove economic capabilities
    -- TODO: What happens to the inventory items and credits held by the factory and trader?
    if self:hasMarket() then
      self:removeMarket()
    end
    if self:hasFactory() then
      self:removeFactory()
    end
    if self:hasTrader() then
      self:removeTrader()
    end

    -- If this object was dockable, make it undockable
    -- NOTE: This must come last, as removing docked ships includes a self.dockable assertion
    if self:hasDockable() and self:isDockable() then
      self:setUndockable()
    end

    if self == GameState.player.currentShip then
      -- TODO: Do any unloading/savegame/etc actions required upon player ship destruction
      -- NOTE: The "Game Over" message is displayed in Application.lua
      printf("Player ship %s has been destroyed, game over!", self:getName())
    end
  end
end

function Entity:getHealth()
  assert(self.health)
  return self.health
end

function Entity:getHealthNormalized()
  assert(self.health)
  return self.health / self.healthMax
end

function Entity:getHealthPercent()
  assert(self.health)
  return 100.0 * self.health / self.healthMax
end

function Entity:hasHealth()
  return self.health ~= nil
end

-- WARNING : Note the subtlety that isAlive and isDestroyed are NOT
--           complementary! An asteroid is not alive, but neither has it been
--           destroyed. Both 'alive' and 'destroyed' require health to be true.

function Entity:isAlive()
  return self.health and self.health > 0
end

function Entity:isDestroyed()
  return self.health and self.health <= 0
end

function Entity:setHealth(value, max, rate)
  assert(self.health)
  self.health = value
  self.healthMax = max
  self.healthRate = rate
end

function Entity:updateHealth(state)
  if not self:isDestroyed() then
    local timeScale = 1.0
    if GameState.paused then
      timeScale = 0.0
    end

    if Input.GetDown(Bindings.TimeAccel) then
      timeScale = GameState.debug.timeAccelFactor
    end

    self.health = min(self.healthMax, self.health + (timeScale * state.dt) * self.healthRate)
  end
end
