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
    return Marauding(self.base, self.system)
end

function Marauding:getType()
    return Enums.Jobs.Marauding
end

function Marauding:getName()
    return format('Marauding %d for base %s',
        self.jcount,
        self.base:getName())
end

function Marauding:reset()
    self.maraudingArea = nil
    self.attackTarget = nil
    self.targetPosition = nil
    self.blackMarketTarget = nil
end

function Marauding:getPayout(e)
    -- TODO: black market demand and threat based potential payout
    self.jcount = self.jcount + 1 -- temp until proper jcount setting
    local payout = 1000
    return payout
end

function Marauding:getThreatLevel()
    local zone = self.maraudingArea:getZone()
    if zone then
        return zone.threatLevel
    else
        return 0
    end
end

function Marauding:onUpdateActive(e, dt)
    if not GameState.paused then
        Profiler.Begin('Actions.Marauding.onUpdateActive')
        if not e.jobState then e.jobState = Enums.JobStateMarauding.None end
        e.jobState = e.jobState + 1

        if e.jobState == Enums.JobStateMarauding.SelectArea then
            if self.base and self.system then
                self.maraudingArea = self.system:sampleZones(self.system.rng)
            else
                e:popAction()
            end
        elseif e.jobState == Enums.JobStateMarauding.MovingToArea then
            if self.maraudingArea then
                self.targetPosition = self.maraudingArea:getRandomPos(self.system.rng)
                e:pushAction(Actions.MoveToPos(self.targetPosition, 2000))
            else
                e:popAction()
            end
        elseif e.jobState == Enums.JobStateMarauding.Marauding then
            e:pushAction(Actions.MaraudAt(self.maraudingArea, 10000))
        elseif e.jobState == Enums.JobStateMarauding.FindBlackMarket then
            if not self.blackMarketTarget then
                self.blackMarketTarget = self.base --TODO: Needs to find a marketplace
            end
            e:pushAction(Actions.MoveTo(self.blackMarketTarget, 150))
        elseif e.jobState == Enums.JobStateMarauding.DockingAtStation then
            if self.blackMarketTarget and self.blackMarketTarget:hasDockable() and self.blackMarketTarget:isDockable() and not self.blackMarketTarget:isBanned(e) then
                e:pushAction(Actions.DockAt(self.blackMarketTarget))
            else
                e:popAction()
                e.jobState = nil
            end
        elseif e.jobState == Enums.JobStateMarauding.SellingItems then
            if self.blackMarketTarget and self.blackMarketTarget:hasDockable() and self.blackMarketTarget:isDockable() and not self.blackMarketTarget:isBanned(e) then
                local sold = 0
                for item, count in pairs(e.inventory) do
                    for i = 1, count do
                        if self.blackMarketTarget:getBlackMarketTrader():buy(e, item) then
                            sold = sold + 1
                        end
                    end
                    printf("[MARAUDER] %s sold %d units of %s to Black Market %s", e:getName(), sold, item:getName(),
                        self.blackMarketTarget:getName())
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
