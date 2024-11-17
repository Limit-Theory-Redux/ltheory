local Action = require('GameObjects.Action')

local MaraudAt = Subclass(Action, function(self, zone, radius)
    self.radius = radius
    self.targetPosition = nil
    self.system = nil
    self.patrolZone = zone
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
        local subType = Config:getObjectInfo("ship_subtypes", ship:getSubType())
        if e:getOwner() ~= ship:getOwner() and not ship:isShipDocked() and (subType == "Trader" or subType == "Miner") then
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
    if attackTarget and attackTarget:isAlive() and not attackTarget:isDestroyed()
        and attackTarget:mgrInventoryGetFreeTotal() and attackTarget:getOwner() ~= e:getOwner() then
        return attackTarget
    end
    return nil
end

function MaraudAt:onUpdateActive(e, dt)
    if e:mgrInventoryGetFreeTotal() < 1 or self.searchAttempts >= 24 then
        e:popAction()
    end

    if not self.system then
        self.system = e:getRoot() or GameState.world.currentSystem
    end

    if self.targetPosition then
        if e:getPos():distance(self.targetPosition) < 2500 or self.wasAttacking then
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
            e:pushAction(Actions.MoveToPos(self.targetPosition, 2000, true))
        end
    elseif not self.targetPosition and self.patrolZone then
        self.targetPosition = self.patrolZone:getRandomPos(self.system.rng)
    end
    self.searchAttempts = self.searchAttempts + 1
end

return MaraudAt
