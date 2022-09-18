local Entity = require('Entity.Entity')

local Trigger = subclass(Entity, function (self, halfExtents)
  self:addTrigger(halfExtents)
end)

return Trigger
