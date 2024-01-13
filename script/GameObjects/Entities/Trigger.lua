local Entity = require('GameObjects.Entity')

local Trigger = subclass(Entity, function(self, halfExtents)
    self:addTrigger(halfExtents)
end)

return Trigger
