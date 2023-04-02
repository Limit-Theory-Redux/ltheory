local Entity = require('GameObjects.Entity')
local Material = require('GameObjects.Material')

local Station = subclass(Entity, function (self, seed)
  local mesh = Gen.StationOld(seed):managed()
  self:addActions()
  self:addAttackable(true)
  self:addCapacitor(10000, 10000, 100)
  self:addChildren()
  self:addDispositions()
  self:addDockable()
  self:addExplodable()
  self:addFlows()
  self:addHealth(10000, 20)
  self:addInventory(1e8)
  self:addMinable(false)
  self:addRigidBody(true, mesh)
  self:addTrackable(true)
  self:addVisibleMesh(mesh, Material.Metal())

  self:setDrag(10, 10) -- fix station in place
  self:setScale(100)
  self:setMass(1e7)

  self.explosionSize = 512 -- destroyed stations have visually larger explosions than ships
end)

function Station:attackedBy (target)
  -- This station has been attacked
  -- TODO: Allow a number of "grace" hits that decay over time
  -- TODO: Improve smarts so that this station can decide which of multiple attackers to target
  if not self:isDestroyed() then
--printf("Station %s (health at %3.2f%%) attacked by %s!", self:getName(), self:getHealthPercent(), target:getName())
    -- Stations currently have no turrets, so pushing an Attack() action generates an error
    -- If and when stations are armed, modify this method to let the station know whodunnit
    self:modDisposition(target, -0.2)

    if self:hasDockable() then
      if self:isHostileTo(target) and self:isDockable() then
        -- If this object was dockable, make it undockable
        self:setUndockable()
      end
    end
  end
end

return Station
