local Action = require('GameObjects.Action')
--local Bindings = require('States.ApplicationBindings')

local rng = RNG.FromTime()
local timeUntilTravelDrive = 15 -- temporary local setting

local MoveTo = subclass(Action, function (self, target, range, useTravelDrive)
<<<<<<< HEAD
    self.target = target
    self.range = range
    self.useTravelDrive = useTravelDrive
=======
  self.target = target
  self.range = range
  self.useTravelDrive = useTravelDrive
>>>>>>> 1b58bb0278295d31845972084d1313877cd21e29
end)

function MoveTo:clone()
    return MoveTo(self.target, self.range)
end

function MoveTo:getName()
    local typename = Config:getObjectInfo("object_types", self.target:getType())
    return format("MoveTo %s '%s'", typename, self.target:getName())
end

local function getTargetPos(e, target)
    local tp = target:getPos()
    local tr = target:getRadius()
    local tu = target:getUp()
    local er = e:getRadius()
    return tp - tu:muls(1.25 * tr + er)
end

function MoveTo:onUpdateActive(e, dt)
    -- Move to within the supplied range of the target object
    local tp = getTargetPos(e, self.target)

<<<<<<< HEAD
    -- Within range of the target object?
    if (e:getPos() - tp):length() <= self.range or (e == GameState.player.currentShip and not GameState.player.playerMoving) then
        -- MoveTo is complete, remove movement action from entity's Action queue
        e:popAction()
        e.travelDriveActive = false
        e.travelDriveTimer = 0
=======
  -- Within range of the target object?
  if (e:getPos() - tp):length() <= self.range or (e == GameState.player.currentShip and not GameState.player.playerMoving) then
    -- MoveTo is complete, remove movement action from entity's Action queue
--printf("-> %s ended", e:getCurrentAction():getName())
    e:popAction()
    e.travelDriveActive = false
    e.travelDriveTimer = 0
>>>>>>> 1b58bb0278295d31845972084d1313877cd21e29

        if e == GameState.player.currentShip and GameState.player.playerMoving then
            GameState.player.playerMoving = false
        end

        return -- within range, so end flight
    end

    -- Use the "target" metaphor to store where this ship is moving to
    e:setTarget(self.target)

    if GameState.debug.instantJobs then
        local p = e:getPos()
        local dp = tp - p
        e:setPos(p + dp:normalize():scale(rng:getUniform() * min(dp:length(), dt * GameState.debug.jobSpeed)))
    else
        if self.useTravelDrive then
            if not e.travelDriveActive and e.travelDriveTimer >= timeUntilTravelDrive then
                e.travelDriveActive = true
            else
                e.travelDriveTimer = e.travelDriveTimer + dt
            end
        end

<<<<<<< HEAD
        local tf = self.target:getForward()
        local tu = self.target:getUp()
        self:flyToward(e, tp, -tf, tu)
    end
=======
  if GameState.debug.instantJobs then
    local p = e:getPos()
    local dp = tp - p
    e:setPos(p + dp:normalize():scale(rng:getUniform() * min(dp:length(), dt * GameState.debug.jobSpeed)))
  else
    if self.useTravelDrive then
      if not e.travelDriveActive and e.travelDriveTimer >= timeUntilTravelDrive then
        e.travelDriveActive = true
      else
        e.travelDriveTimer = e.travelDriveTimer + dt
      end
    end

    local tf = self.target:getForward()
    local tu = self.target:getUp()
    self:flyToward(e, tp, -tf, tu)
  end
>>>>>>> 1b58bb0278295d31845972084d1313877cd21e29
end

return MoveTo
