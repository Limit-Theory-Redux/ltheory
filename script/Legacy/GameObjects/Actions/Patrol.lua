local Action = require('Legacy.GameObjects.Action')

local Patrol = Subclass("Patrol", Action, function(self, patrolNodes)
    self.patrolNodes = patrolNodes
    self.patrolCurrentNodeIndex = 1
    self.system = nil
end)

function Patrol:clone()
    return Patrol()
end

function Patrol:getName()
    return 'Patrol'
end

function Patrol:onUpdateActive(e, dt)

end

return Patrol
