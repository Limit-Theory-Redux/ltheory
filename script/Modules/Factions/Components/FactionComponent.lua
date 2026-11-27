local Component = require("Core.ECS.Component")

---A faction: an organisation of players with a name, a kind (`Enums.FactionType`), a colour and a
---team, the name the weapons know its members' ships by. Its treasury is the faction entity's own
---`PlayerBankAccount`.
---@class FactionComponent: Component
---@overload fun(self: FactionComponent, name: string, factionType: integer, color: Color|nil): FactionComponent subclass internal
---@overload fun(name: string, factionType: integer, color: Color|nil): FactionComponent subclass external
local FactionComponent = Subclass("FactionComponent", Component, function(self, name, factionType, color)
    self:setComponentName("Faction")

    self.name = name
    self.factionType = factionType
    self.color = color or Color(1, 1, 1, 1)
    self.team = nil
end)

---@return string
function FactionComponent:getName()
    return self.name
end

---@return integer factionType An `Enums.FactionType`
function FactionComponent:getType()
    return self.factionType
end

---@return Color
function FactionComponent:getColor()
    return self.color
end

---@return string team
function FactionComponent:getTeam()
    return self.team
end

---@param team string
function FactionComponent:setTeam(team)
    self.team = team
end

return FactionComponent
