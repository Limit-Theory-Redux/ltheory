local Entity      = require('GameObjects.Entity')
local BasicShapes = require('Systems.Gen.ShapeLib.BasicShapes')
local SocketType  = require('GameObjects.Entities.Ship.SocketType')

local shared
local rng = RNG.FromTime()

local Communicator = subclass(Entity, function(self)
    -- All of this crap is completely worthless, but updateCommunicator() will not be called without it
    if not shared then
        shared = {}
        shared.mesh = BasicShapes.Prism(2, 3):finalize()
        shared.mesh:computeNormals()
        shared.mesh:computeAO(0.1)
    end
    self:addRigidBody(true, shared.mesh)
    self:addVisibleMesh(shared.mesh, Material.Debug())

    -- OK, back now to what Communicator actually requires
    self.name         = Config.gen.compCommunicatorStats.name
    self.healthCurr   = Config.gen.compCommunicatorStats.healthCurr
    self.healthMax    = Config.gen.compCommunicatorStats.healthMax
    self.rating       = Config.gen.compCommunicatorStats.rating
    --printf("Register: Communicator name = '%s', type = %s, handler = %s", self.name, Event.Update, self.updateCommunicator)
    self:register(Event.Update, self.updateCommunicator)
end)

function Communicator:getSocketType()
    return SocketType.Communicator
end

function Communicator:damageHealth(amount)
    if self.healthCurr - amount < 1e-6 then
        self.healthCurr = 0.0
    else
        self.healthCurr = self.healthCurr - amount
    end
    --printf("Vessel %s communicator takes %s damage, %s remaining", self:getName(), amount, self.healthCurr)
end

function Communicator:getHealth()
    return self.healthCurr or 0.0
end

function Communicator:getHealthMax()
    return self.healthMax or 0.0
end

function Communicator:getHealthPercent()
    if self.healthMax < 1e-6 then return 0.0 end
    return 100.0 * self.healthCurr / self.healthMax
end

function Communicator:getName()
    return self.name
end

function Communicator:getRating()
    return self.rating
end

function Communicator:setHealth(value, max)
    self.healthCurr = value
    self.healthMax = floor(max)
end

function Communicator:setName(newName)
    self.name = newName
end

function Communicator:updateCommunicator(state)
    if not self:getParent():isDestroyed() then
        --printf("COMMUNICATOR: %s", self:getName())
    end
end

return Communicator
