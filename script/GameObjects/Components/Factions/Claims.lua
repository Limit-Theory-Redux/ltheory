local Entity = require('GameObjects.Entity')

function Entity:addClaim(entity, strength)
    local guid = entity:getGuid()

    if self.claims then
        self.claims[guid] = strength
    else
        self.claims = {}
        self.claims[guid] = strength
    end
end

function Entity:getClaimStrength(entity)
    local guid = entity:getGuid()
    return self.claims[guid] or format('Entity @ %p', self)
end

--! Consider if this is a duplicate of Elements\NPC\Claims or both are of use