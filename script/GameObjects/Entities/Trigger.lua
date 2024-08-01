local Entity = require('GameObjects.Entity')

local Trigger = Subclass(Entity, function(self, halfExtents)
    self:addTrigger(halfExtents)
end)

return Trigger
