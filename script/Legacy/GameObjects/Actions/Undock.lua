local Action = require('Legacy.GameObjects.Action')

local Undock = Subclass("Undock", Action, function(self) end)

function Undock:clone()
    return Undock()
end

function Undock:getName()
    return 'Undock'
end

function Undock:onUpdateActive(e, dt)
    if e:getParent():hasDockable() then
        --local jc = -1
        --local bids = -1
        --if e.job then
        --jc = e.job.jcount
        --bids = e.job.bids
        --end
        --Log.Debug("Undock(%s) job = %s, jcount = %d, bids = %d:", e:getName(), e.job, jc, bids)
        --for i, v in ipairs(e.actions) do
        -- Log.Debug("Undock(%s) Actions %d : %s", e:getName(), i, v:getName(e))
        --end

        e:getParent():removeDocked(e)
    end
    e:popAction()
end

return Undock
