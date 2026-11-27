local Component = require("Core.ECS.Component")

---What a faction thinks of the others: a disposition from -1 (hostile) to 1 (friendly) per faction,
---and the baseline it drifts back to. A faction it has no entry for is at the neutral baseline.
---@class FactionRelationsComponent: Component
---@overload fun(self: FactionRelationsComponent): FactionRelationsComponent subclass internal
---@overload fun(): FactionRelationsComponent subclass external
local FactionRelationsComponent = Subclass("FactionRelationsComponent", Component, function(self)
    self:setComponentName("FactionRelations")

    ---@type table<integer, number>
    self.dispositions = {}
    ---@type table<integer, number>
    self.baselines = {}
end)

---@param factionId integer
---@return number disposition
function FactionRelationsComponent:getDisposition(factionId)
    return self.dispositions[factionId] or self:getBaseline(factionId)
end

---@param factionId integer
---@param disposition number
function FactionRelationsComponent:setDisposition(factionId, disposition)
    self.dispositions[factionId] = math.max(-1, math.min(1, disposition))
end

---@param factionId integer
---@return number baseline
function FactionRelationsComponent:getBaseline(factionId)
    return self.baselines[factionId] or 0
end

---@param factionId integer
---@param baseline number
function FactionRelationsComponent:setBaseline(factionId, baseline)
    self.baselines[factionId] = math.max(-1, math.min(1, baseline))
end

---@return table<integer, number> dispositions Faction id -> disposition, for the factions it has one for
function FactionRelationsComponent:getDispositions()
    return self.dispositions
end

return FactionRelationsComponent
