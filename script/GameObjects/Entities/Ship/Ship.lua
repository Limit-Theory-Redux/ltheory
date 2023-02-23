local Entity = require('GameObjects.Entity')
local Material = require('GameObjects.Material')

local Ship = subclass(Entity, function (self, proto)
  self:addActions()
  self:addCapacitor(100, 10)
  self:addChildren()
  self:addExplodable()
  self:addHealth(100, 5)
  self:addInventory(100)

  self.explosionSize = 64 -- ships get the default explosion size

  -- TODO : This will create a duplicate BSP because proto & RigidBody do not
  --        share the same BSP cache. Need unified cache.
  self:addRigidBody(true, proto.mesh) -- required

  self:addSockets()
  self:addVisibleMesh(proto.mesh, Material.Metal())
  self:addThrustController()

  -- TODO : Suggestive that JS-style prototype objects + 'clone' would work
  --        better for ShipType et al
  for type, elems in pairs(proto.sockets) do
    for i, pos in ipairs(elems) do
      self:addSocket(type, pos, true)
    end
  end

  self:setDrag(0.75, 4.0)
  self:setScale(proto.scale)

  local mass = 50.0 * (self:getRadius() ^ 3.0)
  self:setMass(mass)
end)

-- TODO : Calculate true top speed based on max thrust & drag factor
function Ship:getTopSpeed ()
  return 100
end

function Ship:attackedBy (target)
  -- This ship has been attacked (self.health reduced below self.healthMax by damage)
  -- TODO: Allow a number of "grace" hits that decay over time
  -- TODO: Improve smarts so that this ship can decide which of multiple attackers to target
  if not self:isDestroyed() then
    -- Ignore hits on ships that have already been destroyed
printf("%s (health at %3.2f%%) attacked by %s!", self:getName(), self:getHealthPercent(), target:getName())
    if self ~= Config.game.currentShip then
      -- If this non-player-controlled ship is not yet attacking its attacker, empty its Action queue and add the Attack action
      if self:hasActions() then
        local currAction = self:getCurrentAction()
        if currAction and not string.find(currAction:getName(), "Attack") then
          self:clearActions()
          self:pushAction(Actions.Attack(target))
        end
      end
    end
  end
end

return Ship
