local Action = require('GameObjects.Action')
local Bindings = require('States.ApplicationBindings')

local rng = RNG.FromTime()

local MoveTo = subclass(Action, function (self, target, range)
  self.target = target
  self.range = range
end)

function MoveTo:clone ()
  return MoveTo(self.target, self.range)
end

function MoveTo:getName ()
  local typename = Config:getObjectInfo("object_types", self.target:getType())
  return format("MoveTo %s '%s'", typename, self.target:getName())
end

function MoveTo:onUpdateActive (e, dt)
  if e:getMinDistance(self.target) <= self.range or (e == Config.game.currentShip and not Config.game.playerMoving) then
    -- MoveTo is complete, remove movement action from entity's Action queue
printf("-> %s ended", e:getCurrentAction():getName())
    e:popAction()

    if Config.game.playerMoving then
      Config.game.playerMoving = false
      Config.debug.instantJobs = true
    end

    return
  end

  if Config.game.playerMoving then
    Config.debug.instantJobs = false
  end

  if Config.debug.instantJobs then
--print("MoveTo - instantJob!")
    local p = e:getPos()
    local dp = self.target:getPos() - p
    e:setPos(p + dp:normalize():scale(rng:getUniform() * min(dp:length(), dt * Config.debug.jobSpeed)))
  else
--printf("MoveTo - flyToward %s", self.target:getName())
    self:flyToward(e,
      self.target:getPos(),
      e:getForward(),
      e:getUp())
  end
end

return MoveTo
