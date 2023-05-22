local Entity = require('GameObjects.Entity')

function Entity:addGuid ()
  self.guid = Guid()
end

function Entity:getGuid ()
  return self.guid
end