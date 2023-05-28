local Action = require('GameObjects.Action')

local Patrol = subclass(Action, function(self, target, radius)
  self.target = target
  self.radius = radius
  self.targetPosition = nil
  self.system = nil
  self.patrolZone = nil
  self.attackTarget = nil
end)

function Patrol:clone()
  return Patrol()
end

function Patrol:getName()
  return 'Patrol'
end

function Patrol:FindClosestTarget (e, radius)
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

function Patrol:onUpdateActive (e, dt)

  if not self.system then
    self.system = GameState.world.currentSystem
  end

  if not self.patrolZone then
    self.patrolZone = self.system:sampleZones(self.system.rng)
  end

  if not self.targetPosition then
    self.targetPosition = self.patrolZone:getRandomPos(self.system.rng)
  end

  self:flyToward(e, self.targetPosition, e:getForward(), e:getUp())
  if e:getPos():distance(self.targetPosition) < 2000 then
    self.attackTarget = self:FindClosestTarget(e, 15000)
    if self.attackTarget and self.attackTarget:isAlive() and not self.attackTarget:isDestroyed() then
      e:pushAction(Actions.Attack(self.attackTarget))
      print(e:getName() .. " is attacking: " .. self.attackTarget:getName())
    end
    self.targetPosition = nil
  end

end

return Patrol
