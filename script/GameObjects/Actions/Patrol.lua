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
end

function Patrol:onUpdateActive(e, dt)
  if not self.system then
    self.system = GameState.world.currentSystem
  end

  if not self.patrolZone then
    self.patrolZone = self.system:sampleZones(self.system.rng)
  end

  if not self.targetPosition then
    self.targetPosition = self.patrolZone:getRandomPos(self.system.rng)
  end

  if self.attackTarget then

    if not self.attackTarget:isAlive() or self.attackTarget:isDestroyed() then --If target is destroyed, look for nearby targets
      self.attackTarget = checkForViableTarget(self, e, 5000)
      if not self.attackTarget then return end
    end

    local actionName = format("Attack %s", self.attackTarget:getName())
    local attackAction = e:findAction(actionName)
    if attackAction ~= e:getCurrentAction(actionName) then
      e:pushAction(Actions.Attack(self.attackTarget))
      print(e:getName() .. " is attacking: " .. self.attackTarget:getName())
    end
  elseif e:getPos():distance(self.targetPosition) < 2000 then
    self.targetPosition = self.patrolZone:getRandomPos(self.system.rng)
    self.attackTarget = checkForViableTarget(self, e, 10000)
  else
    self:flyToward(e, self.targetPosition, e:getForward(), e:getUp())
  end
end

return Patrol
