local Entity = require('GameObjects.Entity')

function Entity:addMinable (minable)
  assert(not self.minable)
  self.minable = minable
end

function Entity:getMinable ()
  assert(self.minable)
  return self.minable
end

function Entity:hasMinable ()
  return self.minable ~= nil
end

function Entity:isMinable ()
  return self.minable
end

function Entity:setMinable (minable)
  assert(self.minable)
  self.minable = minable
end
