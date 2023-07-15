local Job = require('GameObjects.Job')
local Flow = require('Systems.Economy.Flow')
local Actions = requireAll('GameObjects.Actions')

local Patrolling = subclass(Job, function(self, base, system, patrolNodes)
  self.base = base
  self.jcount = 0
  self.system = system
  self.patrolNodes = patrolNodes
end)

function Patrolling:clone()
  return Patrolling(self.base, self.system)
end

function Patrolling:getType()
  return Enums.Jobs.Marauding
end

function Patrolling:getName()
  return format('Marauding %d for base %s',
    self.jcount,
    self.base:getName())
end

function Patrolling:reset()
  self.patrolNodes = nil
end

function Patrolling:cancelJob(e)
  e:popAction()
  e.jobState = nil
end

function Patrolling:getPayout(e)
  -- TODO: black market demand and threat based potential payout
  self.jcount = self.jcount + 1 -- temp until proper jcount setting
  local payout = 1000
  return payout
end

function Patrolling:getThreatLevel()
  local zone = self.base:getZone()
  if zone then
    return zone.threatLevel
  else
    return 0
  end
end

function Patrolling:onUpdateActive(e, dt)
  if not GameState.paused then
    Profiler.Begin('Actions.Patrolling.onUpdateActive')
    if not e.jobState then e.jobState = Enums.JobStatePatrolling.None end
    e.jobState = e.jobState + 1

    if e.jobState == Enums.JobStatePatrolling.Patrolling then
      for i = 1, #self.patrolNodes, 1 do
        if self.patrolNodes[i] then
          e:pushAction(Actions.MoveTo(self.patrolNodes[i]))
        end
      end
    elseif e.jobState == Enums.JobStatePatrolling.DockingAtStation then
      e:pushAction(Actions.MoveTo(self.base, 150))
      e:pushAction(Actions.DockAt(self.base))
    elseif e.jobState == Enums.JobStatePatrolling.Undocking then
      if e:isShipDocked() then
        e:pushAction(Actions.Undock())
      end
    elseif e.jobState == Enums.JobStatePatrolling.JobFinished then
      e:getPayout()
      if self.jcount <= 0 then
        self:cancelJob(e)
      else
        -- repeat until job is done
        e.jobState = Enums.JobStatePatrolling.None
      end
    end

    --end of update
    Profiler.End()
  end
end

return Patrolling
