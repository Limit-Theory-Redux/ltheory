--[[----------------------------------------------------------------------------
  Dispositions are normalized to the following scale in App.lua:
     -1.0 = maximal hostility
      0.0 = total neutrality
     +1.0 = maximal friendliness
----------------------------------------------------------------------------]]--

local Entity = require('GameObjects.Entity')

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
  return self:getDisposition(target) >= Config.game.dispoFriendlyThreshold
end

function Entity:isHostileTo (target)
  assert(self.dispositions)
  return self:getDisposition(target) <  Config.game.dispoHostileThreshold
end

function Entity:modDisposition (target, amount)
  assert(self.dispositions)

  local newDispVal = self:getDisposition(target) + amount
  newDispVal = math.min(math.max(newDispVal, Config.game.dispoMin), Config.game.dispoMax) -- normalize disposition within allowed ranges
  self:setDisposition(target, newDispVal)
end

function Entity:setDisposition (target, value)
  if self ~= Config.game.currentShip then
    assert(self.dispositions)
    self.dispositions[target] = value
printf("Disposition of %s to %s is now %f!", self:getName(), target:getName(), self:getDisposition(target))

  -- generate an integer array index: -1.0 to -0.33332 -> 1, -0.33333 to 0.33332 -> 2, 0.33333 to 1.0 -> 3
  local dispNameIndex = 2
  if self:isHostileTo(target) then
    dispNameIndex = 1
  elseif self:isFriendlyTo(target) then
    dispNameIndex = 3
  end
printf("%s is now %s to %s.", self:getName(), Config.game.dispoName[dispNameIndex], target:getName())

  end
end

--function Entity:getDispositionColor (disp)
--  if disp < Config.game.dispoHostileThreshold then
--    return Color(1.0, 0.2, 0.2, 1.0) -- red (hostile)
--  elseif disp <= Config.game.dispoFriendlyThreshold then
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
