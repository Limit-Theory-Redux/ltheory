local Action = require('GameObjects.Action')

local MineAt = subclass(Action, function (self, source, target)
  assert(source:hasYield())
  self.source = source
  self.target = target
--printf("MineAt %s from %s to %s", self.source:getYield().item:getName(), self.source:getName(), self.target:getName())
end)

function MineAt:clone ()
  return MineAt(self.source, self.target)
end

function MineAt:getName ()
  return format('MineAt @ %s at %s for %s', self.source:getYield().item, self.source:getName(), self.target:getName())
end

function MineAt:onUpdateActive (e, dt)
  local item = self.source:getYield().item
  local maxBids = self.target:getTrader():getBidVolume(item)
  local addedCount = 0

  if maxBids > 0 then
--printf("MineAt: [%s] is mining %s from %s, delivering to %s (wants %d)",
--e:getName(), item:getName(), self.source:getName(), self.target:getName(), maxBids)

    if Config.debug.instantJobs then
      for i = 1, maxBids do
--printf("Mining (instant) 1 unit of %s, count = %d of %d", item:getName(), i, maxBids)
        if not e:addItem(item, 1) then
          break
        else
          addedCount = addedCount + 1
        end
      end
      e:popAction()
    else
      -- TODO : dt-invariant extraction rate
      for i = 1, maxBids do
        -- Mine only as many of item as the buyer has bids for
--printf("Mining (regular) 1 unit of %s, count = %d of %d", item:getName(), i, maxBids)
        if not e:addItem(item, 1) then
          break
        else
          addedCount = addedCount + 1
        end
      end
      e:popAction()
    end
  else
printf("MineAt STOP: [%s] was mining %s from %s, but %s no longer has any bids for this!",
e:getName(), item:getName(), self.source:getName(), self.target:getName())
    e:popAction()
  end

  if addedCount == 0 then
    -- Asset was unable to mine (either no room in inventory, or the trader suddenly has 0 bids for this item)
    e:popAction() -- pop the Mine() action off the Action stack
    e.jobState = nil -- set the Asset's job status back to the starting point
  end
end

return MineAt
