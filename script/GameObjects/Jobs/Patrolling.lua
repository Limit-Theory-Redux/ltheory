local Job = require('GameObjects.Job')
local Flow = require('Systems.Economy.Flow')
local Actions = requireAll('GameObjects.Actions')

local Patrolling = Subclass(Job, function(self, base, system, patrolNodes)
    self.src = base
    self.jcount = 0
    self.system = base:getRoot()
    self.patrolNodes = patrolNodes
    self.attackTarget = nil
    self.workers = {}
    self.maxWorkers = 3
end)

function Patrolling:clone()
    return Patrolling(self.src, self.system)
end

function Patrolling:isWorker(asset)
    assert(asset)
    for _, worker in ipairs(self.workers) do
        if worker == asset then
            return true
        end
    end
    return false
end

function Patrolling:addWorker(asset)
    assert(asset)
    table.insert(self.workers, asset)
    return false
end

function Patrolling:removeWorker(asset)
    assert(asset)
    for i, worker in ipairs(self.workers) do
        if worker == asset then
            table.remove(self.workers, i)
        end
    end
    return false
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
    e:popAction()
    self.jobState = nil
    self.src.stationPatrolJobs = self.src.stationPatrolJobs + 1
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

function Patrolling:findClosestTarget(e, radius)
    local closestDistance = math.huge
    local closestShip = nil

    if not self.system then
        self.system = GameState.world.currentSystem
    end

    for index, ship in ipairs(self.system.ships) do
        if e:getOwner() ~= ship:getOwner() and e:isHostileTo(ship) and not ship:isShipDocked() then
            local distance = e:getDistance(ship)
            if distance < closestDistance and distance < radius then
                closestShip = ship
            end
        end
    end
    return closestShip
end

function Patrolling:checkForViableTarget(e, radius)
    local attackTarget = self:findClosestTarget(e, radius)
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
            printf("[PATROL 1] *** %s has started patrolling for station %s",
                e:getName(), self.src:getName())
            for i = 1, #self.patrolNodes do
                local useTravelDrive = false
                if i == i then useTravelDrive = true end
                if self.patrolNodes[i] then
                    self.attackTarget = self:checkForViableTarget(e, 10000)
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
            self:cancelJob(e)
            printf("[PATROL 2] *** %s has finished its patrol job for station %s",
                e:getName(), self.src:getName())
        end

        --end of update
        Profiler.End()
    end
end

return Patrolling
