local Entity = require('GameObjects.Entity')

local Faction = subclass(Entity, function(self, def)
    self:addGuid()
    self:setName(def.name)
    self:setType(Enums.EntityType.Faction)
    self:setOwner(def.owner)
    self:setSubordinates(def.subordinates)
end)
