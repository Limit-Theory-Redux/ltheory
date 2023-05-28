local Action = require('GameObjects.Action')

local Patrol = subclass(Action, function(self, target, radius)
  self.target = target
  self.radius = radius
  self.targetPosition = nil
  self.system = nil
  self.patrolZone = nil
  self.attackTarget = nil
  self.wasAttacking = false
end)

function Patrol:clone()
  return Patrol()
end

function Patrol:getName()
  return 'Patrol'
end

local function findClosestTarget(self, e, radius)
  local closestDistance = math.huge
  local closestShip = nil
  for index, ship in ipairs(self.system.ships) do
    if e:getOwner() ~= ship:getOwner() and not ship:isShipDocked() then
      local distance = ship:getDistance(e)
      if distance < closestDistance and distance < radius then
        closestShip = ship
      end
    end
  end
  return closestShip
end

local function checkForViableTarget(self, e, radius)
  local attackTarget = findClosestTarget(self, e, radius)
  if attackTarget and attackTarget:isAlive() and not attackTarget:isDestroyed() then
    return attackTarget
  end
  return nil
end

function Patrol:onUpdateActive(e, dt)
  if not self.system then
    self.system = GameState.world.currentSystem
  end

  if not self.patrolZone then
    self.patrolZone = self.system:sampleZones(self.system.rng)
  end

  if self.targetPosition then
    if e:getPos():distance(self.targetPosition) < 2000 or self.wasAttacking then
      self.attackTarget = checkForViableTarget(self, e, 10000)
      if self.attackTarget then
        e:pushAction(Actions.Attack(self.attackTarget))
        --print(e:getName() .. " is attacking: " .. self.attackTarget:getName())
        self.wasAttacking = true
      else
        -- reset
        self.targetPosition = nil
        self.wasAttacking = false
      end
    else
      self:flyToward(e, self.targetPosition, e:getForward(), e:getUp())
    end
  elseif not self.targetPosition and self.patrolZone then
    self.targetPosition = self.patrolZone:getRandomPos(self.system.rng)
  end
end

return Patrol
