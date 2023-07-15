--[[----------------------------------------------------------------------------
  Dispositions are normalized to the following scale in App.lua:
     -1.0 = maximal hostility
      0.0 = total neutrality
     +1.0 = maximal friendliness
----------------------------------------------------------------------------]]
--

local Entity = require('GameObjects.Entity')

function Entity:addDispositions()
    assert(not self.dispositions)
    self.dispositions = {}
end

function Entity:getDisposition(target)
    assert(self.dispositions)
    return self.dispositions[target] or 0
end

function Entity:isFriendlyTo(target)
    assert(self.dispositions)
    return self:getDisposition(target) >= Config.game.dispoFriendlyThreshold
end

function Entity:isHostileTo(target)
    assert(self.dispositions)
    return self:getDisposition(target) < Config.game.dispoHostileThreshold
end

function Entity:modDisposition(target, amount)
    assert(self.dispositions)

    local newDispVal = self:getDisposition(target) + amount
    newDispVal = math.min(math.max(newDispVal, Config.game.dispoMin), Config.game.dispoMax) -- normalize disposition within allowed ranges
    self:setDisposition(target, newDispVal)
end

function Entity:setDisposition(target, value)
    if self ~= Config.game.currentShip then
        assert(self.dispositions)

        local oldDispo = self:getDisposition(target)
        if oldDispo ~= value then
            self.dispositions[target] = value
            --printf("Disposition of %s to %s was %f, is now %f", self:getName(), target:getName(), oldDispo, self.dispositions[target])

            -- Generate an integer array index: -1.0 to -0.33332 -> 1, -0.33333 to 0.33332 -> 2, 0.33333 to 1.0 -> 3
            -- NOTE: This section is just debugging, but it's being left here as a "how to" for disposition descriptors
            --       Remove for productionizing
            --      local dispoNameIndex = 0
            --      if     oldDispo >= Config.game.dispoHostileThreshold  and value <  Config.game.dispoHostileThreshold  then
            --        dispoNameIndex = 1
            --      elseif oldDispo <  Config.game.dispoHostileThreshold  and value >= Config.game.dispoHostileThreshold  then
            --        dispoNameIndex = 2
            --      elseif oldDispo >= Config.game.dispoFriendlyThreshold and value <  Config.game.dispoFriendlyThreshold then
            --        dispoNameIndex = 2
            --      elseif oldDispo <  Config.game.dispoFriendlyThreshold and value >= Config.game.dispoFriendlyThreshold then
            --        dispoNameIndex = 3
            --      end
            --      if dispoNameIndex ~= 0 then
            --        local surprise = ""
            --        if dispoNameIndex == 1 or dispoNameIndex == 3 then
            --          surprise = "!"
            --        end
            --        printf("%s is now %s to %s%s", self:getName(), Config.game.dispoName[dispoNameIndex], target:getName(), surprise)
            --      end
        end
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
