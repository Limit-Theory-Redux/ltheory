local Entity = require('Legacy.GameObjects.Entity')
local Pulse = require('Legacy.GameObjects.Entities.Effects.Pulse')

local Projectile = Subclass("Projectile", Entity, function(self, pR, pG, pB)
    self.pColorR = pR
    self.pColorG = pG
    self.pColorB = pB
    self.effect = nil
end)

function Projectile:getEffect()
    return self.effect
end

function Projectile:getPos()
    assert(self.effect)
    return self.effect.pos
end

function Entity:addProjectiles()
    assert(not self.projectiles)
    self.projectiles = {}
    self.pcount = 0

    self:register(OldEvent.Update, Entity.updateProjectiles)
    self:register(OldEvent.Update, Entity.updateProjectilesPost)
end

function Entity:addProjectile(source)
    assert(self.projectiles)

    -- TODO: Extend projectile types to non-pulse effects

    -- *** TEMP: Audio FX test START ***
    if Config.audio.sounds.pulseFire then
        Config.audio.sounds.pulseFire:play(1.0, source:getPos())
    end
    -- *** TEMP: Audio FX test END ***

    local pulse = Pulse:new()
    pulse.source = IncRef(source)

    local newProjectile = nil

    newProjectile = Projectile(source.projColorR, source.projColorG, source.projColorB)

    newProjectile.effect = pulse

    if GameState.render.pulseLights then
        newProjectile:addLight(newProjectile.pColorR * 3,
            newProjectile.pColorG * 3,
            newProjectile.pColorB * 3)
    end

    self.pcount = self.pcount + 1
    newProjectile:setName(format("Projectile: Pulse %d", self.pcount))
    --Log.Debug("Added projectile '%s' at %s", newProjectile:getName(), newProjectile.effect.pos)
    insert(self.projectiles, newProjectile)

    return newProjectile
end

function Entity:renderProjectiles(state)
    Pulse.Render(self.projectiles, state)
end

function Entity:updateProjectiles(state)
    Pulse.UpdatePrePhysics(self, self.projectiles, state.dt)
end

function Entity:updateProjectilesPost(state)
    Pulse.UpdatePostPhysics(self, self.projectiles, state.dt)
end

return Projectile
