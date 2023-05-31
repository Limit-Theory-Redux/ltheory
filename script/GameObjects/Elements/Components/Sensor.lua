local Entity = require('GameObjects.Entity')
local BasicShapes  = require('Systems.Gen.ShapeLib.BasicShapes')
local SocketType = require('GameObjects.Entities.Ship.SocketType')

local shared
local rng = RNG.FromTime()

local Sensor
Sensor = subclass(Entity, function (self)
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
  self.mappingRange = Config.gen.compSensorStats.mappingRange
  self.scanSpeed    = Config.gen.compSensorStats.scanSpeed
  self.scanDetail   = Config.gen.compSensorStats.scanDetail
  self.lockBreaking = Config.gen.compSensorStats.lockBreaking
--printf("Register: Sensor name = '%s', type = %s, handler = %s", self.name, Event.Update, Sensor.updateSensor)
  self:register(Event.Update, Sensor.updateSensor)
end)

function Sensor:getSocketType ()
  return SocketType.Sensor
end

function Sensor:damageHealth (amount)
  if self.healthCurr - amount < 1e-6 then
    self.healthCurr = 0.0
  else
    self.healthCurr = self.healthCurr - amount
  end
--printf("Vessel %s sensor takes %s damage, %s remaining", self:getName(), amount, self.healthCurr)
end

function Sensor:getHealth ()
  return self.healthCurr or 0.0
end

function Sensor:getHealthMax ()
  return self.healthMax or 0.0
end

function Sensor:getHealthPercent ()
  if self.healthMax < 1e-6 then return 0.0 end
  return 100.0 * self.healthCurr / self.healthMax
end

function Sensor:getName ()
  return self.name
end

function Sensor:setHealth (value, max)
  self.healthCurr = value
  self.healthMax = floor(max)
end

function Sensor:setName (newName)
  self.name = newName
end

function Sensor:updateSensor (state)
  if not self:getParent():isDestroyed() then
--printf("SENSOR: %s", self:getName())
  end
end

return Sensor
