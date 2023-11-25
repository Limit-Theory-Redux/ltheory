local Entity = require('GameObjects.Entity')

function Entity:setRelation(faction, value)
    local guid = guidToKey(faction:getGuid())

    if self.relations then
        self.relations[guid] = value
    else
        self.relations = {}
        self.relations[guid] = value
    end
end

function Entity:getRelation(faction)
    local guid = guidToKey(faction:getGuid())
    return self.relations[guid] or format('Entity @ %p', self)
end
