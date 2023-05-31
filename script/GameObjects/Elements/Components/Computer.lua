local Entity = require('GameObjects.Entity')
local BasicShapes  = require('Systems.Gen.ShapeLib.BasicShapes')
local SocketType = require('GameObjects.Entities.Ship.SocketType')

local shared
local rng = RNG.FromTime()

local Computer
Computer = subclass(Entity, function (self)
  -- All of this crap is completely worthless, but updateComputer() will not be called without it
  if not shared then
    shared = {}
    shared.mesh = BasicShapes.Prism(2, 3):finalize()
    shared.mesh:computeNormals()
    shared.mesh:computeAO(0.1)
  end
  self:addRigidBody(true, shared.mesh)
  self:addVisibleMesh(shared.mesh, Material.Debug())

  -- OK, back now to what Computer actually requires
  self.name         = Config.gen.compComputerStats.name
  self.healthCurr   = Config.gen.compComputerStats.healthCurr
  self.healthMax    = Config.gen.compComputerStats.healthMax
  self.mappingRange = Config.gen.compComputerStats.mappingRange
  self.scanSpeed    = Config.gen.compComputerStats.scanSpeed
  self.scanDetail   = Config.gen.compComputerStats.scanDetail
  self.lockBreaking = Config.gen.compComputerStats.lockBreaking
--printf("Register: Computer name = '%s', type = %s, handler = %s", self.name, Event.Update, Computer.updateComputer)
  self:register(Event.Update, Computer.updateComputer)
end)

function Computer:getSocketType ()
  return SocketType.Computer
end

function Computer:damageHealth (amount)
  if self.healthCurr - amount < 1e-6 then
    self.healthCurr = 0.0
  else
    self.healthCurr = self.healthCurr - amount
  end
--printf("Vessel %s computer takes %s damage, %s remaining", self:getName(), amount, self.healthCurr)
end

function Computer:getHealth ()
  return self.healthCurr or 0.0
end

function Computer:getHealthMax ()
  return self.healthMax or 0.0
end

function Computer:getHealthPercent ()
  if self.healthMax < 1e-6 then return 0.0 end
  return 100.0 * self.healthCurr / self.healthMax
end

function Computer:getName ()
  return self.name
end

function Computer:setHealth (value, max)
  self.healthCurr = value
  self.healthMax = floor(max)
end

function Computer:setName (newName)
  self.name = newName
end

function Computer:updateComputer (state)
  if not self:getParent():isDestroyed() then
--printf("COMPUTER: %s", self:getName())
  end
end

return Computer
