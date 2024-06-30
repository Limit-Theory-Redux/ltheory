local Entity = require('GameObjects.Entity')

function Entity:addGuid()
    self.guid = Guid.Create()
end

function Entity:getGuid()
    return self.guid
end
