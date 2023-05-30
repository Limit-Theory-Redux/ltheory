local Entity = require('GameObjects.Entity')
local Material = require('GameObjects.Material')

local Ship = subclass(Entity, function (self, proto)

  self:addActions()
  self:addCapacitor(100, 10)
  self:addChildren()
  self:addDispositions()
  self:addExplodable()
  self:addHealth(50, 0.05)
  self:addInventory(100)
  self:addTrackable(true)
  self:addAttackable(true)
  self:addMinable(false)

  self.explosionSize = 64 -- ships get the default explosion size

  self.usesBoost = false -- default ships fly at only the normal speed
  self.travelDriveActive = false

  -- TODO : This will create a duplicate BSP because proto & RigidBody do not
  --        share the same BSP cache. Need unified cache.
  self:addRigidBody(true, proto.mesh) -- required

  self:addSockets()
  self:addVisibleMesh(proto.mesh, Material.Metal())
  self:addThrustController()
  self:addCredits(1000)

  -- TODO : Suggestive that JS-style prototype objects + 'clone' would work
  --        better for ShipType etc.
  for type, elems in pairs(proto.sockets) do
    for i, pos in ipairs(elems) do
      self:addSocket(type, pos, true)
    end
  end

  self:setDrag(0.75, 4.0)
  self:setScale(proto.scale)

  -- TODO: Use mass values from the ship hull class
  local mass = 1000.0 + (self:getRadius() * 2000) -- (fully loaded F-15 = 20,000 kg, but Josh's mass calc gets sluggish x 10)
  self:setMass(mass) -- lower mass is related to the ship "wobble" problem

  local shipDockedAt = nil -- create a variable to store where the ship is docked, if it's docked
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
--printf("%s (health at %3.2f%%) attacked by %s!", self:getName(), self:getHealthPercent(), target:getName())
    self:modDisposition(target, -0.2)
    if self ~= GameState.player.currentShip and self:isHostileTo(target) then
      -- If this non-player-controlled ship is not currently attacking its attacker,
      --    add an action to Attack its attacker
      if self:hasActions() then
        local actionName = format("Attack %s", target:getName()) -- must match namegen in Attack.lua
        local attackAction = self:findAction(actionName)
        if attackAction then
          if attackAction ~= self:getCurrentAction() then
            -- If the action to attack the attacker exists in this entity's Actions queue but isn't the current
            --     action, delete the old Attack action and push a new instance to the top of the Actions queue
            self:deleteAction(actionName)
            self:pushAction(Actions.Attack(target))
          end
        else
          self:pushAction(Actions.Attack(target))
        end
        self:distressCall(target, 12500)
      else
        self:pushAction(Actions.Attack(target))
      end
    end
  end
end

function Ship:distressCall (target, range)
  local owner = self:getOwner()
  for asset in owner:iterAssets() do
    if asset:getType() == Config:getObjectTypeByName("object_types", "Ship") and self:isHostileTo(target) and self:getDistance(asset) < range then
      local currentAction = asset:getCurrentAction()

      if (currentAction and not string.find(currentAction:getName(),"Attack")) or not currentAction then
        asset:pushAction(Actions.Attack(target))
        --print(asset:getName() .. " answering distress call of " .. self:getName())
      end
    end
  end
end

function Ship:setShipDocked (entity)
  self.shipDockedAt = entity -- mark 'entity' (just ships for now) as docked

  -- If the player was targeting a ship that just docked, remove the target lock
  -- TODO: This check needs to be applied to ALL ships, not just the player's ship
  if GameState.player.currentShip and self == GameState.player.currentShip:getTarget() then
    GameState.player.currentShip:setTarget(nil)
  end

--if self.shipDockedAt then
--  printf("%s docked at Station %s", self:getName(), self.shipDockedAt:getName())
--else
--  printf("%s undocked from Station %s", self:getName(), self.shipDockedAt:getName())
--end
end

function Ship:isShipDocked ()
  return self.shipDockedAt
end

return Ship
