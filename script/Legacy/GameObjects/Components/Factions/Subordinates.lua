local Entity = require('Legacy.GameObjects.Entity')

function Entity:getSubordinates()
    return self.subordinates or format('Entity @ %p', self)
end

function Entity:addSubordinate(aiPlayer)
    if aiPlayer and aiPlayer.guid then
        self.subordinates[guidToKey(aiPlayer.guid)] = aiPlayer
    end
end

function Entity:removeSubordinate(aiPlayer)
    if aiPlayer and aiPlayer.guid and self.subordinates[guidToKey(aiPlayer.guid)] then
        self.subordinates[guidToKey(aiPlayer.guid)] = nil
    end
end
