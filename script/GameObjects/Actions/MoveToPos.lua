local Action = require('GameObjects.Action')
--local Bindings = require('States.ApplicationBindings')

local rng = RNG.FromTime()
local timeUntilTravelDrive = 15 -- temporary local setting

local MoveToPos = subclass(Action, function (self, targetPos, range, useTravelDrive)
  self.targetPos = targetPos
  self.range = range
  self.useTravelDrive = useTravelDrive
end)

function MoveToPos:clone ()
  return MoveToPos(self.target, self.range)
end

function MoveToPos:getName ()
  return format("MoveTo '%s'", tostring(self.targetPos))
end

function MoveToPos:onUpdateActive (e, dt)
  if e:getPos():distance(self.targetPos) < self.range then
    e:popAction()
    e.travelDriveActive = false
    e.travelDriveTimer = 0
  else
    if self.useTravelDrive then
      if not e.travelDriveActive and e.travelDriveTimer >= timeUntilTravelDrive then
        e.travelDriveActive = true
      else
        e.travelDriveTimer = e.travelDriveTimer + dt
      end
    end

    self:flyToward(e, self.targetPos, e:getForward(), e:getUp())
  end
end

return MoveToPos
