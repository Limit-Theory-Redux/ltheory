--[[----------------------------------------------------------------------------
  Dispositions are normalized to the following scale:
     +1.0 = maximal friendliness
      0.0 = total neutrality
     -1.0 = maximal hostility
----------------------------------------------------------------------------]]--

local Entity = require('GameObjects.Entity')

local kFriendlyThreshold = 0.5
local kHostileThreshold = -0.5

function Entity:addDispositions ()
  assert(not self.dispositions)
  self.dispositions = {}
end

function Entity:getDisposition (target)
  assert(self.dispositions)
  return self.dispositions[target] or 0
end

function Entity:isFriendlyTo (target)
  assert(self.dispositions)
  return self:getDisposition(target) >= kFriendlyThreshold
end

function Entity:isHostileTo (target)
  assert(self.dispositions)
  return self:getDisposition(target) <= kHostileThreshold
end

function Entity:modDisposition (target, amount)
  assert(self.dispositions)
  self:setDisposition(target, self:getDisposition(target) + amount)
end

function Entity:setDisposition (target, value)
  if self ~= Config.game.currentShip then
    assert(self.dispositions)
    self.dispositions[target] = value
printf("Disposition of %s to %s is now %f!", self:getName(), target:getName(), self:getDisposition(target))
  end
end

--function Entity:getDispositionColor (disp)
--  if disp < -0.3 then
--    return Color(1.0, 0.2, 0.2, 1.0) -- red (hostile)
--  elseif disp <= 0.3 then
--    return Color(0.1, 0.2, 1.0, 1.0) -- blue (neutral)
--  else
--    return Color(0.1, 1.0, 0.2, 1.0) -- green (friendly)
--  end
--end

return {
  GetColor = function (disp)
    local x = 0.5 * disp + 0.5
    return Color(
      Math.Bezier3(x, 1.00, 0.10, 0.30),
      Math.Bezier3(x, 0.10, 0.75, 1.00),
      Math.Bezier3(x, 0.30, 1.50, 0.20),
      Math.Bezier3(x, 1.00, 1.00, 1.00))
  end
}
