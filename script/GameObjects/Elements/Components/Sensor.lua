local Entity      = require('GameObjects.Entity')
local BasicShapes = require('Systems.Gen.ShapeLib.BasicShapes')
local SocketType  = require('GameObjects.Entities.Ship.SocketType')

local shared
local rng         = RNG.FromTime()

local Sensor      = Subclass("", Entity, function(self)
    -- All of this crap is completely worthless, but updateSensor() will not be called without it
    if not shared then
        shared = {}
        shared.mesh = BasicShapes.Prism(2, 3):finalize()
        shared.mesh:computeNormals()
        shared.mesh:computeAO(0.1)
    end
    self:addRigidBody(true, shared.mesh)
    self:addVisibleMesh(shared.mesh, Material.Debug())

    -- OK, back now to what Sensor actually requires
    self.name         = Config.gen.compSensorStats.name
    self.healthCurr   = Config.gen.compSensorStats.healthCurr
    self.healthMax    = Config.gen.compSensorStats.healthMax
    self.rating       = Config.gen.compSensorStats.rating
    self.lockBreaking = Config.gen.compSensorStats.lockBreaking
    self.mappingRange = Config.gen.compSensorStats.mappingRange
    self.scanDetail   = Config.gen.compSensorStats.scanDetail
    self.scanSpeed    = Config.gen.compSensorStats.scanSpeed
    --Log.Debug("Register: Sensor name = '%s', type = %s, handler = %s", self.name, OldEvent.Update, self.updateSensor)
    self:register(OldEvent.Update, self.updateSensor)
end)

function Sensor:getSocketType()
    return SocketType.Sensor
end

function Sensor:damageHealth(amount)
    if self.healthCurr - amount < 1e-6 then
        self.healthCurr = 0.0
    else
        self.healthCurr = self.healthCurr - amount
    end
    --Log.Debug("Vessel %s sensor takes %s damage, %s remaining", self:getName(), amount, self.healthCurr)
end

function Sensor:getHealth()
    return self.healthCurr or 0.0
end

function Sensor:getHealthMax()
    return self.healthMax or 0.0
end

function Sensor:getHealthPercent()
    if self.healthMax < 1e-6 then return 0.0 end
    return 100.0 * self.healthCurr / self.healthMax
end

function Sensor:getLockBreaking()
    return self.lockBreaking
end

function Sensor:getMappingRange()
    return self.mappingRange
end

function Sensor:getName()
    return self.name
end

function Sensor:getRating()
    return self.rating
end

function Sensor:getScanDetail()
    return self.scanDetail
end

function Sensor:getScanSpeed()
    return self.scanSpeed
end

function Sensor:setHealth(value, max)
    self.healthCurr = value
    self.healthMax = floor(max)
end

function Sensor:setName(newName)
    self.name = newName
end

function Sensor:updateSensor(state)
    if not self:getParent():isDestroyed() then
        --Log.Debug("SENSOR: %s", self:getName())
    end
end

return Sensor
