local Entity = require('GameObjects.Entity')

function Entity:setFactionType(type)
  self.factionType = type
end

function Entity:getFactionType()
  return self.factionType or format('Entity @ %p', self)
end

function Entity:getFactionTypeName()
    return Enums.FactionTypeNames[self.factionType] or format('Entity @ %p', self)
end

