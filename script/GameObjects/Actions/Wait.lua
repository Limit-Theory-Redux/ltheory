local Action = require('GameObjects.Action')

local Wait = Subclass("Wait", Action, function(self, duration)
    self.duration = duration
    self.t = 0
end)

function Wait:clone()
    return Wait(self.duration)
end

function Wait:getName()
    return 'Wait'
end

function Wait:onUpdateActive(e, dt)
    self.t = self.t + dt
    if self.t >= self.duration then
        e:popAction()
        return
    end
end

return Wait
