local Entity      = require('GameObjects.Entity')
local BasicShapes = require('Systems.Gen.ShapeLib.BasicShapes')
local SocketType  = require('GameObjects.Entities.Ship.SocketType')

local shared
local rng         = RNG.FromTime()

local LifeSupport = subclass(Entity, function(self)
    -- All of this crap is completely worthless, but updateLifeSupport() will not be called without it
    if not shared then
        shared = {}
        shared.mesh = BasicShapes.Prism(2, 3):finalize()
        shared.mesh:computeNormals()
        shared.mesh:computeAO(0.1)
    end
    self:addRigidBody(true, shared.mesh)
    self:addVisibleMesh(shared.mesh, Material.Debug())

    -- OK, back now to what LifeSupport actually requires
    self.name       = Config.gen.compLifeSupportStats.name
    self.healthCurr = Config.gen.compLifeSupportStats.healthCurr
    self.healthMax  = Config.gen.compLifeSupportStats.healthMax
    self.pods       = Config.gen.compLifeSupportStats.pods
    --printf("Register: Sensor name = '%s', type = %s, handler = %s", self.name, Event.Update, self.updateLifeSupport)
    self:register(Event.Update, self.updateLifeSupport)
end)

function LifeSupport:getSocketType()
    return SocketType.LifeSupport
end

function LifeSupport:damageHealth(amount)
    if self.healthCurr - amount < 1e-6 then
        self.healthCurr = 0.0
    else
        self.healthCurr = self.healthCurr - amount
    end
    --printf("Vessel %s life support takes %s damage, %s remaining", self:getName(), amount, self.healthCurr)
end

function LifeSupport:getHealth()
    return self.healthCurr or 0.0
end

function LifeSupport:getHealthMax()
    return self.healthMax or 0.0
end

function LifeSupport:getHealthPercent()
    if self.healthMax < 1e-6 then return 0.0 end
    return 100.0 * self.healthCurr / self.healthMax
end

function LifeSupport:getName()
    return self.name
end

function LifeSupport:setHealth(value, max)
    self.healthCurr = value
    self.healthMax = floor(max)
end

function LifeSupport:setName(newName)
    self.name = newName
end

function LifeSupport:updateLifeSupport(state)
    if not self:getParent():isDestroyed() then
        --printf("LIFESUPPORT: %s", self:getName())
    end
end

return LifeSupport
