local Entity = require('GameObjects.Entity')

function Entity:getHull()
    return self.hull or format('Entity @ %p', self)
end

function Entity:getRole()
    return self.role or format('Entity @ %p', self)
end

function Entity:getSubType()
    return self.subtype or format('Entity @ %p', self)
end

function Entity:getType()
    return self.type or format('Entity @ %p', self)
end

function Entity:setHull(hullsize)
    self.hull = hullsize
end

function Entity:setRole(role)
    self.role = role
end

function Entity:setSubType(subtype)
    self.subtype = subtype
end

function Entity:setType(type)
    self.type = type
end

function Entity:__tostring()
    return self:getType()
end
