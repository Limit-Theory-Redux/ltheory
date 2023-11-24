local Entity = require('GameObjects.Entity')

function Entity:getSubordinates()
    return self.subordinates or format('Entity @ %p', self)
end

function Entity:addSubordinate(aiPlayer)
    if aiPlayer and aiPlayer.guid then
        self.subordinates[aiPlayer.guid] = aiPlayer
    end
end

function Entity:removeSubordinate(aiPlayer)
    if aiPlayer and aiPlayer.guid and self.subordinates[aiPlayer.guid] then
        self.subordinates[aiPlayer.guid] = nil
    end
end
