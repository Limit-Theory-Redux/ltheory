local Entity = require('GameObjects.Entity')
local Material = require('GameObjects.Material')
local Components = requireAll('GameObjects.Elements.Components')

local Ship = subclass(Entity, function (self, proto, hull)
printf("@@@ Entities:Ship - proto.scale = %s, hull = %s", proto.scale, hull)
  -- TODO : This will create a duplicate BSP because proto & RigidBody do not
  --        share the same BSP cache. Need unified cache.
  self:addRigidBody(true, proto.mesh) -- required
  self:addVisibleMesh(proto.mesh, Material.Metal())

  self:addActions()
  self:addAttackable(true)
  self:addChildren()
  self:addDispositions()
  self:addExplodable()
  self:addMinable(false)
  self:addTrackable(true)

  -- TEMP: give each ship the maximum number of every component
  -- TODO: Load each ship's component sockets with:
  --       a) default loadout for ships never encountered
  --       b) defined loadout from ships in a save file (including the player's ship)
  --       c) nothing loaded for a ship newly built in a factory or in a trader's inventory
  self.countHull        = proto.countHull
  self.countComputer    = proto.countComputer
  self.countSensor      = proto.countSensor
  self.countLifeSupport = proto.countLifeSupport
  self.countCapacitor   = proto.countCapacitor
  self.countThruster    = proto.countThruster
  self.countTurret      = proto.countTurret
  self.countBay         = proto.countBay
  self.countInventory   = proto.countInventory
  self.countDrone       = proto.countDrone
  self.countShield      = proto.countShield
  self.countArmor       = proto.countArmor

  self:addComponents()

  -- Add all sockets to parent
  -- TODO : Suggestive that JS-style prototype objects + 'clone' would work
  --        better for ShipType etc.
  self:addSockets()

  for type, elems in pairs(proto.sockets) do
    for i, pos in ipairs(elems) do
      self:addSocket(type, pos, true)
    end
  end

  self:addThrustController()

  self:setDrag(0.75, 4.0)
--  self:setScale(Config.gen.shipHullScale[hull])
--  if hull ~= Enums.ShipHulls.VeryLarge then
    self:setScale(proto.scale)
--  end

  -- TODO: Use mass values from the ship hull class _and_ installed components
  -- NOTE: a fully loaded F-15 ~= 20,000 kg
  self:setMass(Config.gen.shipHullMass[hull]) -- lower mass is related to the ship "wobble" problem
printf("@@@ Entities:Ship - final radius = %s, mass = %s", self:getRadius(), self:getMass())

  self.explosionSize = 64 -- ships get the default explosion size
  self.usesBoost = false -- default ships fly at only the normal speed
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
--printf("%s (health at %3.2f%%) attacked by %s!", self:getName(), self:mgrHullGetHullPercent(), target:getName())
    self:modDisposition(target, -0.2)
    if self ~= GameState.player.currentShip and self:isHostileTo(target) then
      -- If this non-player-controlled ship is not currently attacking its attacker,
      --    add an action to Attack its attacker
      if self:hasActions() then
        local actionName = format("Attack %s", target:getName()) -- must match namegen in Attack.lua
        local attackAction = self:findAction(actionName)
        if attackAction then
          if attackAction ~= self:getCurrentAction(actionName) then
            -- If the action to attack the attacker exists in this entity's Actions queue but isn't the current
            --     action, delete the old Attack action and push a new instance to the top of the Actions queue
            self:deleteAction(actionName)
            self:pushAction(Actions.Attack(target))
          end
        else
          self:pushAction(Actions.Attack(target))
        end
      else
        self:pushAction(Actions.Attack(target))
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
