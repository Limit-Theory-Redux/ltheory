local Action = require('GameObjects.Action')
--local Bindings = require('States.ApplicationBindings')

local rng = RNG.FromTime()

local MoveToPos = subclass(Action, function (self, targetPos, radius)
  self.targetPos = targetPos
  self.radius = radius
end)

function MoveToPos:clone ()
  return MoveToPos(self.target, self.range)
end

function MoveToPos:getName ()
  local typename = Config:getObjectInfo("object_types", self.target:getType())
  return format("MoveTo %s '%s'", typename, self.target:getName())
end


function MoveToPos:onUpdateActive (e, dt)
    if e:getPos():distance(self.targetPosition) < self.radius then
        e:popAction()
    else
        self:flyToward(e, self.targetPos, e:getForward(), e:getUp())
    end
end

return MoveToPos
