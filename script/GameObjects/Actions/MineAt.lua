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

--printf("MineAt: [%s], %s from %s to %s (wants %d)",
--e:getName(), item:getName(), self.source:getName(), self.target:getName(), maxBids)

  if Config.debug.instantJobs then
    for i = 1, maxBids do
--printf("Mining (instant) 1 unit of %s, count = %d of %d", item:getName(), i, maxBids)
      e:addItem(item, 1)
    end
    e:popAction()
    return
  else
    -- TODO : dt-invariant extraction rate
    for i = 1, maxBids do
      -- Mine only as many of item as the buyer has bids for
      local addSuccess = e:addItem(item, 1)
--printf("Mining (regular) 1 unit of %s, count = %d of %d", item:getName(), i, maxBids)
      if not addSuccess then
        e:popAction()
        return
      end
    end
    e:popAction()
  end
end

return MineAt
