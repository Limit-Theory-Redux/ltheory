local Entity = require('Legacy.GameObjects.Entity')
local SocketType = require('Legacy.GameObjects.Entities.Ship.SocketType')

local Armor = Subclass("Armor", Entity, function(self)
    self.name       = Config.gen.compArmorStats.name
    self.healthCurr = Config.gen.compArmorStats.healthCurr
    self.healthMax  = Config.gen.compArmorStats.healthMax
end)

function Armor:getSocketType()
    return SocketType.Armor
end

function Armor:damageHealth(amount)
    if self.healthCurr - amount < 1e-6 then
        self.healthCurr = 0.0
    else
        self.healthCurr = self.healthCurr - amount
    end
    --Log.Debug("Vessel %s armor takes %s damage, %s remaining", self:getName(), amount, self.healthCurr)
end

function Armor:getHealth()
    return self.healthCurr or 0.0
end

function Armor:getHealthMax()
    return self.healthMax or 0.0
end

function Armor:getHealthPercent()
    if self.healthMax < 1e-6 then return 0.0 end
    return 100.0 * self.healthCurr / self.healthMax
end

function Armor:getName()
    return self.name
end

function Armor:setHealth(value, max)
    self.healthCurr = value
    self.healthMax = floor(max)
end

function Armor:setName(newName)
    self.name = newName
end

return Armor
