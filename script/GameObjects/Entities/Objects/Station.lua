local Entity = require('GameObjects.Entity')
local Material = require('GameObjects.Material')

local Station = subclass(Entity, function (self, seed)
  local mesh = Gen.StationOld(seed):managed()
  self:addActions()
  self:addCapacitor(10000, 10000, 100)
  self:addChildren()
  self:addDispositions()
  self:addDockable()
  self:addExplodable()
  self:addFlows()
  self:addHealth(10000, 20)
  self:addInventory(1e10)
  self:addRigidBody(true, mesh)
  self:addVisibleMesh(mesh, Material.Metal())

  self:setDrag(0, 0)
  self:setScale(100)
  self:setMass(1e10)

  self.explosionSize = 512 -- stations make bigger explosions than ships
end)

function Station:attackedBy (target)
  -- This station has been attacked
  -- TODO: Allow a number of "grace" hits that decay over time
  -- TODO: Improve smarts so that this station can decide which of multiple attackers to target
  if not self:isDestroyed() then
printf("Station %s (health at %3.2f%%) attacked by %s!", self:getName(), self:getHealthPercent(), target:getName())
    -- Stations currently have no turrets and so pushing an Attack() action generates an error
    -- If an when stations are armed, modify this method to let the station know whodunnit
    self:setDisposition(target, -1.0)
  end
end

return Station
