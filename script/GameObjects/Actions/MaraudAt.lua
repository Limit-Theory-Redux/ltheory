local Action = require('GameObjects.Action')

local MaraudAt = subclass(Action, function(self, system, radius)
  self.radius = radius
  self.targetPosition = nil
  self.system = system
  self.patrolZone = nil
  self.attackTarget = nil
  self.wasAttacking = false
  self.searchAttempts = 0
end)

function MaraudAt:clone()
  return MaraudAt()
end

function MaraudAt:getName()
  return 'MaraudAt'
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

function MaraudAt:onUpdateActive(e, dt)

  if self:getInventoryFree() < 1 or self.searchAttempts > 32 then
    e:popAction()
  end

  if self.targetPosition then
    if e:getPos():distance(self.targetPosition) < 2000 or self.wasAttacking then
      self.attackTarget = checkForViableTarget(self, e, self.radius)
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
  self.searchAttempts = self.searchAttempts + 1
end

return MaraudAt
