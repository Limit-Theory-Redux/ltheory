local Entity = require('Legacy.GameObjects.Entity')

function Entity:getFaction()
    return self.faction or "none"
end

function Entity:setFaction(faction)
    self.faction = faction
end
