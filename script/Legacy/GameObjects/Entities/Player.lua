local Entity = require('Legacy.GameObjects.Entity')

local Player = Subclass("Player", Entity, function(self, name)
    self:setName(name)

    self:addActions() -- needed for Think() action
    self:addAssets()
    self:addDispositions()

    self.controlling = nil
    self.docked = nil
end)

function Player:getControlling()
    return self.controlling
end

function Player:getRoot()
    if not self.controlling then return nil end
    return self.controlling:getRoot()
end

function Player:setControlling(target)
    self.controlling = target
end

return Player
