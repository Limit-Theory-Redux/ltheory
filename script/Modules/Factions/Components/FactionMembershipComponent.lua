local Component = require("Core.ECS.Component")

---Makes a player a member of a faction. A ship or a station belongs to its owner's faction (its
---`OwnershipComponent`), so only players carry this.
---@class FactionMembershipComponent: Component
---@overload fun(self: FactionMembershipComponent, factionId: integer): FactionMembershipComponent subclass internal
---@overload fun(factionId: integer): FactionMembershipComponent subclass external
local FactionMembershipComponent = Subclass("FactionMembershipComponent", Component, function(self, factionId)
    self:setComponentName("FactionMembership")

    self.faction = factionId
end)

---@return integer factionId
function FactionMembershipComponent:getFaction()
    return self.faction
end

---@param factionId integer
function FactionMembershipComponent:setFaction(factionId)
    self.faction = factionId
end

return FactionMembershipComponent
