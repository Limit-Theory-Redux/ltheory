local Action = require('GameObjects.Action')

local Patrol = subclass(Action, function(self, patrolNodes)
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
  if not self.system then
    self.system = e:getRoot() or GameState.world.currentSystem
  end
  if self.patrolNodes[self.patrolCurrentNodeIndex] then
    if e:getPos():distance(self.targetPosition) < 2000 or self.wasAttacking then
      self.patrolCurrentNodeIndex = self.patrolCurrentNodeIndex + 1
      e:pushAction(Actions.MoveTo(self.patrolNodes[self.patrolCurrentNodeIndex]))
    end
  else
    e:popAction()
  end
end

return Patrol
