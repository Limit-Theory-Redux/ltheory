local Action = require('GameObjects.Action')

local Undock = subclass(Action, function (self) end)

function Undock:clone ()
  return Undock()
end

function Undock:getName ()
  return 'Undock'
end

function Undock:onUpdateActive (e, dt)
  if e:getParent():hasDockable() then
    local jc = -1
    local bids = -1

    if e.job then
      jc = e.job.jcount or 0
      bids = e.job.bids or 0
    end
    --printf("Undock(%s) job = %s, jcount = %d, bids = %d", e:getName(), e.job:getName(), jc, bids)

    --for i, v in ipairs(e.actions) do
    --  printf("Undock(%s) Actions %d : %s", e:getName(), i, v:getName(e))
    --end
    e:getParent():removeDocked(e)
  end
  e:popAction()
end

return Undock
