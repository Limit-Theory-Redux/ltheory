local Entity      = require('GameObjects.Entity')
local BasicShapes = require('Systems.Gen.ShapeLib.BasicShapes')
local SocketType  = require('GameObjects.Entities.Ship.SocketType')

local shared
local rng         = RNG.FromTime()

local Cloak       = subclass(Entity, function(self)
    -- All of this crap is completely worthless, but updateCloak() will not be called without it
    if not shared then
        shared = {}
        shared.mesh = BasicShapes.Prism(2, 3):finalize()
        shared.mesh:computeNormals()
        shared.mesh:computeAO(0.1)
    end
    self:addRigidBody(true, shared.mesh)
    self:addVisibleMesh(shared.mesh, Material.Debug())

    -- OK, back now to what Cloak actually requires
    self.name       = Config.gen.compCloakStats.name
    self.healthCurr = Config.gen.compCloakStats.healthCurr
    self.healthMax  = Config.gen.compCloakStats.healthMax
    self.rating     = Config.gen.compCloakStats.rating
    self.draw       = Config.gen.compCloakStats.draw
    self.active     = false
    --printf("Register: Cloak name = '%s', type = %s, handler = %s", self.name, Event.Update, self.updateCloak)
    self:register(Event.Update, self.updateCloak)
end)

function Cloak:getSocketType()
    return SocketType.Cloak
end

function Cloak:activate()
    if self.healthCurr > 0 then
        self.active = true
    end
end

function Cloak:deactivate()
    self.active = false
end

function Cloak:damageHealth(amount)
    if self.healthCurr - amount < 1e-6 then
        self.healthCurr = 0.0
    else
        self.healthCurr = self.healthCurr - amount
    end
    --printf("Vessel %s cloak takes %s damage, %s remaining", self:getName(), amount, self.healthCurr)
end

function Cloak:getHealth()
    return self.healthCurr or 0.0
end

function Cloak:getHealthMax()
    return self.healthMax or 0.0
end

function Cloak:getHealthPercent()
    if self.healthMax < 1e-6 then return 0.0 end
    return 100.0 * self.healthCurr / self.healthMax
end

function Cloak:getName()
    return self.name
end

function Cloak:getStrength()
    local strength = 0

    if self.active then
        strength = self.rating
    end

    return strength
end

function Cloak:setHealth(value, max)
    self.healthCurr = value
    self.healthMax = floor(max)
end

function Cloak:setName(newName)
    self.name = newName
end

function Cloak:updateCloak(state)
    if not self:getParent():isDestroyed() then
        if self.active then
            local timeScale = 1.0
            if GameState.paused then
                timeScale = 0.0
            end
            if Input.GetDown(Bindings.TimeAccel) then
                timeScale = GameState.debug.timeAccelFactor
            end

            local drawAmount = (timeScale * state.dt) * self.drawRate * self.rating

            -- Cloak draws energy from capacitor
            self:getParent():mgrCapacitorDischarge(drawAmount)

            if self:getParent():mgrCapacitorGetCharge() <= 0 then
                -- Out of capacitor power, so turn off this cloaking device
                self.active = false
            end

            --printf("CLOAK: '%s' active = %s", self:getName(), self.active)
        end
    end
end

return Cloak
