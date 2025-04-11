local Action = require('Legacy.GameObjects.Action')

local Repeat = Subclass("Repeat", Action, function(self, actions)
    self.actions = actions
end)

function Repeat:getName()
    return 'Repeat'
end

function Repeat:onUpdateActive(e, dt)
    for i = #self.actions, 1, -1 do
        e:pushAction(self.actions[i]:clone())
    end
end

return Repeat
