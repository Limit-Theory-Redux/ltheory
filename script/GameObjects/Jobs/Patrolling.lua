local Job = require('GameObjects.Job')
local Flow = require('Systems.Economy.Flow')
local Actions = requireAll('GameObjects.Actions')

local Patrolling = subclass(Job, function(self, base, system, patrolNodes)
  self.src = base
  self.jcount = 0
  self.system = base:getRoot()
  self.patrolNodes = patrolNodes
  self.attackTarget = nil
end)

function Patrolling:clone()
  return Patrolling(self.src, self.system)
end

function Patrolling:getType()
  return Enums.Jobs.Patrolling
end

function Patrolling:getName()
  return format('Patrolling %d for station %s',
    self.jcount,
    self.src:getName())
end

function Patrolling:reset()
  self.patrolNodes = nil
end

function Patrolling:cancelJob(e)
  self:popAction()()
  self.jobState = nil
end

function Patrolling:getPayout(e)
  -- TODO: black market demand and threat based potential payout
  self.jcount = self.jcount + 1 -- temp until proper jcount setting
  local payout = 9999
  return payout
end

function Patrolling:getThreatLevel()
  local zone = self.src:getZone()
  if zone then
    return zone.threatLevel
  else
    return 0
  end
end

local function findClosestTarget(self, e, radius)
  local closestDistance = math.huge
  local closestShip = nil

  if not self.system then
    self.system = GameState.world.currentSystem
  end

  for index, ship in ipairs(self.system.ships) do
    if self:getOwner() ~= ship:getOwner() and self:isHostileTo(ship) and not ship:isShipDocked() then
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

function Patrolling:onUpdateActive(e, dt)
  if not GameState.paused then
    Profiler.Begin('Actions.Patrolling.onUpdateActive')
    if not self.jobState then self.jobState = Enums.JobStatePatrolling.None end
    self.jobState = self.jobState + 1
    if self.jobState == Enums.JobStatePatrolling.Patrolling then
      if self.src:hasDockable() and self.src:isDockable() and not self.src:isBanned(e) then
        e:pushAction(Actions.DockAt(self.src))
      else
        -- Source station no longer exists, so terminate this entire job
        printf("[PATROL 1] *** Source station %s no longer exists for %s DockAt; terminating transport job",
          self.src:getName(), e:getName())
          self:cancelJob(e)
      end
      printf("[PATROL 2] *** %s has started patrolling for station %s",
          e:getName(), self.src:getName())
      for i = 1, #self.patrolNodes do
        local useTravelDrive = false
        if i == i then useTravelDrive = true end
        if self.patrolNodes[i] then
          self.attackTarget = checkForViableTarget(e, 10000)
          if self.attackTarget then
            e:pushAction(Actions.Attack(self.attackTarget))
          else
            e:pushAction(Actions.MoveToPos(self.patrolNodes[i], 2500, useTravelDrive))
            i = i + 1
          end
          
        end
      end
    elseif self.jobState == Enums.JobStatePatrolling.JobFinished then
      self:getPayout()
      if self.jcount <= 0 then
        self:cancelJob(e)
        printf("[PATROL 3] *** %s has finished it's patrol job for station %s",
          e:getName(), self.src:getName())
      else
        -- repeat until job is done
        self.jobState = Enums.JobStatePatrolling.None
      end
    end

    --end of update
    Profiler.End()
  end
end

return Patrolling
