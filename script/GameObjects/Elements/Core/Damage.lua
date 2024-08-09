local Entity = require('GameObjects.Entity')

function Entity:applyDamage(amount, source)
    local damageRemaining   = amount
    local shieldRemaining   = self:mgrShieldGetStrength()
    local armorRemaining    = self:mgrArmorGetHealth()

    -- TEMP: Modify the names of NPC ships to indicate their higher threat level
    -- TODO: Assign and display "Ace" status in a more formally managed way
    local thisShipName      = self:getName()
    local attackingShipName = source:getName()
    if self.usesBoost then
        thisShipName = thisShipName .. " [Ace]"
    end
    if source.usesBoost then
        attackingShipName = attackingShipName .. " [Ace]"
    end

    --Log.Debug("hit on '%s' from '%s' for %s damage", thisShipName, attackingShipName, amount)

    self:send(OldEvent.Damaged(amount, source))

    -- Apply damage first to shields (if any), then armor (if any), then hull
    if shieldRemaining > 0 then
        -- Reduce this ship's shield protection (doesn't actually damage the shield generator)
        self:mgrShieldReduceStrength(amount)
        damageRemaining = amount - shieldRemaining
    end
    if damageRemaining > 0 then
        if armorRemaining > 0 then
            -- Some damage made it through the shields, so damage any armor plating installed
            self:mgrArmorDamageHealth(damageRemaining)
            damageRemaining = damageRemaining - armorRemaining
        end
    end
    if damageRemaining > 0 then
        -- Some damage made it through the armor, so damage the hull
        self:mgrHullDamageHealth(damageRemaining)

        if self:mgrHullGetHealth() > 0 then
            -- Randomly damage some internal components, too
        end
    end

    -- Check whether hull health has reached 0; if so, process the vessel's destruction
    if self:isDestroyed() and self:hasAttackable() and self:isAttackable() then
        -- Vessel has been damaged to the point of destruction (0 hull integrity)
        self:clearActions()

        Log.Debug("%s destroyed by %s!", thisShipName, attackingShipName)

        -- Unregister debug events for the destroyed entity
        self:unregister(OldEvent.Debug, Entity.mgrInventoryDebug)

        -- TODO: process the formerly "isAlive()" entity's assets, including credits and cargo
        -- TODO: notify nearby ships that entity has been destroyed
        -- resulting Actions may include Evade, Attack, and/or alert faction members

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

        self:send(OldEvent.Destroyed(source))

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

        -- TODO: Replace the vessel's RigidBody with an appropriate destroyed object
        -- Also create a temporary debris field (zone + numerous small objects)

        if self == GameState.player.currentShip then
            -- TODO: Do any unloading/savegame/etc actions required upon player ship destruction
            -- NOTE: The "Game Over" message is displayed in Application.lua
            Log.Debug("Player ship %s has been destroyed, game over!", self:getName())
        end
    end
end
