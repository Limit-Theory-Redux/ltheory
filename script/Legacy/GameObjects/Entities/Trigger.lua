local Entity = require('Legacy.GameObjects.Entity')

local Trigger = Subclass("Trigger", Entity, function(self, halfExtents)
    self:addTrigger(halfExtents)
end)

return Trigger
