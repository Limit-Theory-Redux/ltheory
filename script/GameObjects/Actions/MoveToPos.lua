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
  return format("MoveTo '%s'", tostring(self.targetPos))
end


function MoveToPos:onUpdateActive (e, dt)
    if e:getPos():distance(self.targetPos) < self.radius then
        e:popAction()
    else
        self:flyToward(e, self.targetPos, e:getForward(), e:getUp())
    end
end

return MoveToPos
