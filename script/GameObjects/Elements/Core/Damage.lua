local Entity = require('GameObjects.Entity')

function Entity:applyDamage (amount, source)
  local damageRemaining = amount
  local shieldRemaining = self:getShield()
  local armorRemaining  = self:getArmor()

  self:send(Event.Damaged(amount, source))

  if shieldRemaining > 0 then
    -- Reduce this ship's shield protection (doesn't actually damage the shield generator)
    self:damageShield(amount)
    damageRemaining = amount - shieldRemaining
  end

  if damageRemaining > 0 then
    if armorRemaining > 0 then
      -- Some damage made it through the shields, so damage any armor plating installed
      self:damageArmor(damageRemaining)
      damageRemaining = damageRemaining - armorRemaining
    end
  end

  if damageRemaining > 0 then
    -- Some damage made it through the armor, so damage the hull
    self:damageHealth(damageRemaining)

    if self:getHealth() > 0 then
      -- Randomly damage some internal components, too
    end
  end

  if self:isDestroyed() and self:hasAttackable() and self:isAttackable() then
    -- Entity has been damaged to the point of destruction (0 hull integrity)
    self:clearActions()

local thisShipName      = self:getName()
local attackingShipName = source:getName()
if self.usesBoost then
  thisShipName = thisShipName .. " [Ace]"
end
if source.usesBoost then
  attackingShipName = attackingShipName .. " [Ace]"
end
printf("%s destroyed by %s!", thisShipName, attackingShipName)

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
