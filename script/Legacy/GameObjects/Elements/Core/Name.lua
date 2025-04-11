local Entity = require('Legacy.GameObjects.Entity')

function Entity:getName()
    return self.name or format('Entity @ %p', self)
end

function Entity:setName(name)
    self.name = name
end

function Entity:__tostring()
    return self:getName()
end
