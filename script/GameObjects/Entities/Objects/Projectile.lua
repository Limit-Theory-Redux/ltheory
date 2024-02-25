local Entity = require('GameObjects.Entity')
local Pulse = require('GameObjects.Entities.Effects.Pulse')

local Projectile = subclass(Entity, function(self, pR, pG, pB)
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

    self:register(Event.Update, Entity.updateProjectiles)
    self:register(Event.Update, Entity.updateProjectilesPost)
end

function Entity:addProjectile(source)
    assert(self.projectiles)

    -- TODO: Extend projectile types to non-pulse effects

    -- *** TEMP: Audio FX test START ***
    if Config.audio.pulseFire then
        local function calculateDistance(pointA, pointB)
            local dx = pointB.x - pointA.x
            local dy = pointB.y - pointA.y
            local dz = pointB.z - pointA.z
            return math.sqrt(dx * dx + dy * dy + dz * dz)
        end

        local function calculateVolume(distance)
            local maxDistance = 5000
            local clampedDistance = math.min(distance, maxDistance)
            local normalizedDistance = clampedDistance / maxDistance
            local volume = (1 - normalizedDistance) ^ 2 * 0.25
            if distance >= maxDistance then
                volume = 0
            end
            return volume
        end

        local distance = calculateDistance(GameState.render.gameView.eyeLast, source:getPos())
        local volDist = calculateVolume(distance)

        if volDist > 0 then
            Config.audio.pulseFire.sound:setVolume(volDist)
            GameState.audio.fxManager:play(Config.audio.pulseFire.sound)
        end

        print("Loaded: " .. tostring(GameState.audio.fxManager:getLoadedCount()),
            "Playing: " .. tostring(GameState.audio.fxManager:getPlayingCount()),
            "Total: " .. tostring(GameState.audio.fxManager:getTotalCount()))
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
