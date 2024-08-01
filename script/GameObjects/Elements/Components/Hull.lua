local Entity = require('GameObjects.Entity')
local SocketType = require('GameObjects.Entities.Ship.SocketType')

local Hull = Subclass(Entity, function(self)
    self.name       = Config.gen.compHullStats.name
    self.healthCurr = Config.gen.compHullStats.healthCurr
    self.healthMax  = Config.gen.compHullStats.healthMax
end)

function Hull:getSocketType()
    return SocketType.Hull
end

function Hull:damageHealth(amount)
    if self.healthCurr - amount < 1e-6 then
        self.healthCurr = 0.0
    else
        self.healthCurr = self.healthCurr - amount
    end
    --Log.Debug("Vessel %s hull takes %s damage, %s remaining", self:getName(), amount, self.healthCurr)
end

function Hull:getHealth()
    return self.healthCurr
end

function Hull:getHealthMax()
    return self.healthMax
end

function Hull:getHealthPercent()
    if self.healthMax < 1e-6 then return 0.0 end
    return 100.0 * self.healthCurr / self.healthMax
end

function Hull:getName()
    return self.name
end

function Hull:setHealth(value, max)
    self.healthCurr = value
    self.healthMax = max
end

function Hull:setName(newName)
    self.name = newName
end

return Hull
