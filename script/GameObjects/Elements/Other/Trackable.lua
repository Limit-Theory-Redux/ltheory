local Entity = require('GameObjects.Entity')

function Entity:addTrackable (trackable)
  assert(not self.trackable)
  self.trackable = trackable
end

function Entity:getTrackable ()
  assert(self.trackable)
  return self.trackable
end

function Entity:hasTrackable ()
  return self.trackable ~= nil
end

function Entity:isTrackable ()
  return self.trackable
end

function Entity:setTrackable (trackable)
  assert(self.trackable)
  self.trackable = trackable
end
