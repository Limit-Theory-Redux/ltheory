local Entity = require('GameObjects.Entity')

function Entity:setType (type)
  self.type = type
end

function Entity:setSubType (subtype)
  self.subtype = subtype
end

function Entity:getType ()
  return self.type or format('Entity @ %p', self)
end

function Entity:getSubType ()
  return self.subtype or format('Entity @ %p', self)
end

function Entity:__tostring ()
  return self:getType()
end
