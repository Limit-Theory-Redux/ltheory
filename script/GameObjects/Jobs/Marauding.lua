local Job = require('GameObjects.Job')
local Flow = require('Systems.Economy.Flow')
local Actions = requireAll('GameObjects.Actions')

local Marauding = subclass(Job, function(self, base, system)
  self.base = base
  self.jcount = 0
  self.maraudingArea = nil
  self.system = system
  self.attackTarget = nil
  self.targetPosition = nil
  self.blackMarketTarget = nil
end)

function Marauding:clone()
  return Marauding(self.src, self.dst, self.item)
end

function Marauding:getName()
  return format('Marauding %d for base %s',
    self.jcount,
    self.base:getName())
end

function Marauding:onUpdateActive(e, dt)
  if not GameState.paused then
    Profiler.Begin('Actions.Marauding.onUpdateActive')
    if not e.jobState then e.jobState = Enums.JobStateMarauding.None end
    e.jobState = e.jobState + 1

    if e.jobState == Enums.JobStateMarauding.SelectArea then
    
      if self.system then
        self.maraudingArea = self.system:sampleZones(self.system.rng)
      else
        e:popAction()
      end

    elseif e.jobState == Enums.JobStateMarauding.MovingToArea then

      self.targetPosition = self.patrolZone:getRandomPos(self.system.rng)
      e:pushAction(Actions.MoveToPos(self.targetPosition, 2000))

    elseif e.jobState == Enums.JobStateMarauding.Marauding then
      e:pushAction(Actions.MaraudAt(self.system, 10000))
    elseif e.jobState == Enums.JobStateMarauding.FindBlackMarket then
      if not self.blackMarketTarget then
        self.blackMarketTarget = self.base --Needs to find a marketplace
      end
      e:pushAction(Actions.MoveTo(self.blackMarketTarget, 150))
    elseif e.jobState == Enums.JobStateMarauding.DockingAtStation then
      if self.blackMarketTarget:hasDockable() and self.blackMarketTarget:isDockable() and not self.blackMarketTarget:isBanned(e) then
        e:pushAction(Actions.DockAt(self.blackMarketTarget))
      else
        e:popAction()
        e.jobState = nil
      end
    elseif e.jobState == Enums.JobStateMarauding.SellingItems then
      if self.dst:hasDockable() and self.dst:isDockable() and not self.dst:isBanned(e) then
        local sold = 0
        for index, item in ipairs(e.inventory) do
          while e:getItemCount(item) > 0 and self.blackMarketTarget:getTrader():buy(e, item) do
            sold = sold + 1
          end
          printf("[MARAUDER] %s sold %d units of %s to Black Market %s", e:getName(), sold, item:getName(), self.blackMarketTarget:getName())
        end
      else
        e:popAction()
        e.jobState = nil
      end
    elseif e.jobState == Enums.JobStateMarauding.Undocking then
      if e:isShipDocked() then
        e:pushAction(Actions.Undock())
      end
    elseif e.jobState == Enums.JobStateMarauding.JobFinished then
      e:popAction()
      e.jobState = nil
    end

    --end of update
    Profiler.End()
  end
end

return Marauding
