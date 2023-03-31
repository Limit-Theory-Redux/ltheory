local Entity = require('GameObjects.Entity')
local Material = require('GameObjects.Material')

local Background = subclass(Entity, function (self, proto)
  self:addActions() -- required
--  self:addAssets()

  -- Flat: All these calls are still required even for an invisible ship!
  --       Mass, drag, and the ThrustController() are used to tell the universe
  --       how to rotate when the game is in Startup mode

  -- TODO : This will create a duplicate BSP because proto & RigidBody do not
  --        share the same BSP cache. Need unified cache.
  self:addRigidBody(true, proto.mesh)

  self:addSockets()
  self:addThrustController()

  self:setDrag(0.75, 4.0)
  self:setScale(proto.scale)

  self:setMass(40000) -- mass needed for the Background Mode "invisible ship" to rotate nicely
end)

-- TODO : Calculate true top speed based on max thrust & drag factor
-- Flat: this function, while strictly speaking not needed for an invisible ship that
--       can only rotate and has no thrust, is retained Just In Case
function Background:getTopSpeed ()
  return 1
end

return Background
