local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local Words = require("Shared.Generation.Words")
local Economy = require("Modules.Economy.Components")
local CoreEntities = require("Modules.Core.Entities")
local ConstructComponents = require("Modules.Constructs.Components")
local Factions = require("Modules.Factions.Components")
local FactionEntity = require("Modules.Factions.Entities.FactionEntity")

-- Design: doc/script/factions.md

-- Distinct colours for the factions, in turn.
local COLORS = {
    Color(0.35, 0.65, 1.0, 1), Color(1.0, 0.75, 0.25, 1), Color(0.45, 0.9, 0.45, 1), Color(0.85, 0.45, 1.0, 1),
    Color(0.3, 0.9, 0.85, 1), Color(1.0, 0.5, 0.65, 1), Color(0.8, 0.8, 0.6, 1),
}
local MARAUDER_COLOR = Color(1.0, 0.2, 0.15, 1)
-- The kinds whose name ends with the kind ("Vexa Corporation"); the others start with it ("Kingdom of Vexa").
local SUFFIXED = { [Enums.FactionType.Corporation] = true, [Enums.FactionType.TradingGuild] = true, [Enums.FactionType.Empire] = true }

---Creates factions and their members, and answers who belongs where and who is whose enemy. A
---faction is an entity (`FactionEntity`); a player belongs to one through its
---`FactionMembershipComponent`; a ship or a station belongs to its owner's faction. Relations are
---per pair of factions, kept the same both ways: a disposition from -1 to 1 and a baseline it drifts
---back to (`FactionSystem`). Two factions are hostile at or below `Config.game.dispoHostileThreshold`.
---Marauders are everyone's enemy: their baseline with every faction is -1.
---
---An independent (a ship its own captain owns, who belongs to no faction) has no faction: it is
---neutral to the factions and hostile to marauders, and its weapons' team is `INDEPENDENT_TEAM`.
---@class FactionManager
---@overload fun(self: FactionManager): FactionManager class internal
---@overload fun(): FactionManager class external
local FactionManager = Class("FactionManager", function(self)
    -- The weapons' team of everything owned outside the factions.
    self.INDEPENDENT_TEAM = "independent"
end)

---@param entityOrId Entity|integer|nil
---@return Entity|nil
local function alive(entityOrId)
    if entityOrId == nil then return nil end
    local entity = rawtype(entityOrId) == "number" and Entity(entityOrId) or entityOrId
    return Registry:hasEntity(entity) and entity or nil
end

---@return Entity[] factions Every faction there is
function FactionManager:all()
    local list = {}
    for entity in Registry:view(Factions.Faction) do list[#list + 1] = entity end
    table.sort(list, function(a, b) return a.id < b.id end)
    return list
end

---A faction's name of its kind: "Vexa Corporation", "Kingdom of Vexa".
---@param factionType integer
---@param rng RNG
---@return string
function FactionManager:nameFor(factionType, rng)
    local base = Words.getCoolName(rng)
    local kind = Enums.FactionTypeNames[factionType]
    if not kind then return base end
    if SUFFIXED[factionType] then return base .. " " .. kind end
    return kind .. " " .. base
end

---Makes a faction, with its team and its relations to the factions there are (neutral, or hostile
---when either is marauders).
---@param spec { type: integer, name: string|nil, color: Color|nil, treasury: number|nil, seed: integer|nil }
---@return Entity faction
function FactionManager:create(spec)
    local rng = RNG.Create(spec.seed or RNG.FromTime():get31())
    local existing = FactionManager:all()
    local marauders = spec.type == Enums.FactionType.Marauders
    local color = spec.color or (marauders and MARAUDER_COLOR) or COLORS[#existing % #COLORS + 1]
    local faction = FactionEntity(spec.name or FactionManager:nameFor(spec.type, rng), spec.type, color, spec.treasury)
    faction:get(Factions.Faction):setTeam("faction:" .. faction.id)
    for _, other in ipairs(existing) do
        local otherMarauders = other:get(Factions.Faction):getType() == Enums.FactionType.Marauders
        local baseline = (marauders or otherMarauders) and -1 or 0
        FactionManager:setBaseline(faction, other, baseline)
        FactionManager:setRelation(faction, other, baseline)
    end
    return faction
end

---A player (an AI captain) of the faction, with an account of `credits`.
---@param faction Entity
---@param name string
---@param credits number|nil
---@return Entity player
function FactionManager:createMember(faction, name, credits)
    local player = CoreEntities.Player(name, true)
    player:get(Economy.PlayerBankAccount):setAccountBalance(credits or 0)
    player:add(Factions.FactionMembership(faction.id))
    return player
end

---The faction an entity belongs to: itself for a faction, its membership's for a player, its owner's
---for an owned ship or station.
---@param entityOrId Entity|integer|nil
---@return Entity|nil faction
function FactionManager:factionOf(entityOrId)
    local entity = alive(entityOrId)
    for _ = 1, 3 do
        if not entity then return nil end
        if entity:get(Factions.Faction) then return entity end
        local membership = entity:get(Factions.FactionMembership)
        if membership then return alive(membership:getFaction()) end
        local ownership = entity:get(Economy.Ownership)
        entity = ownership and alive(ownership:getOwner()) or nil
    end
    return nil
end

---@param a Entity
---@param b Entity
---@return number disposition Of `a` toward `b`; 1 for a faction toward itself
function FactionManager:relation(a, b)
    if a.id == b.id then return 1 end
    return a:get(Factions.FactionRelations):getDisposition(b.id)
end

---@param a Entity
---@param b Entity
---@param disposition number
function FactionManager:setRelation(a, b, disposition)
    if a.id == b.id then return end
    a:get(Factions.FactionRelations):setDisposition(b.id, disposition)
    b:get(Factions.FactionRelations):setDisposition(a.id, disposition)
end

---Moves the relation of two factions by `delta`, both ways.
---@param a Entity
---@param b Entity
---@param delta number
function FactionManager:adjust(a, b, delta)
    FactionManager:setRelation(a, b, FactionManager:relation(a, b) + delta)
end

---@param a Entity
---@param b Entity
---@param baseline number
function FactionManager:setBaseline(a, b, baseline)
    if a.id == b.id then return end
    a:get(Factions.FactionRelations):setBaseline(b.id, baseline)
    b:get(Factions.FactionRelations):setBaseline(a.id, baseline)
end

---@param a Entity|nil
---@param b Entity|nil
---@return boolean
function FactionManager:isHostile(a, b)
    -- Without a faction (an independent): only marauders are enemies.
    if not a or not b then
        local other = a or b
        return other ~= nil and FactionManager:isMarauders(other)
    end
    if a.id == b.id then return false end
    return FactionManager:relation(a, b) <= Config.game.dispoHostileThreshold
end

---@param faction Entity|nil
---@return boolean
function FactionManager:isMarauders(faction)
    local component = faction and faction:get(Factions.Faction)
    return component ~= nil and component:getType() == Enums.FactionType.Marauders
end

---@param entityOrId Entity|integer|nil
---@return string|nil team The weapons' team of the entity's faction; `INDEPENDENT_TEAM` for one owned
---outside the factions; nil for an unowned one
function FactionManager:teamOf(entityOrId)
    local faction = FactionManager:factionOf(entityOrId)
    if faction then return faction:get(Factions.Faction):getTeam() end
    local entity = alive(entityOrId)
    local ownership = entity and entity:get(Economy.Ownership)
    if ownership and alive(ownership:getOwner()) then return FactionManager.INDEPENDENT_TEAM end
    return nil
end

---Makes the ship (or station) a target the weapons know by its faction's team.
---@param entity Entity
function FactionManager:enlist(entity)
    local team = FactionManager:teamOf(entity)
    if not team then return end
    local targetable = entity:get(ConstructComponents.Targetable)
    if targetable then targetable:setTeam(team) else entity:add(ConstructComponents.Targetable(team, "small")) end
end

---The weapons' rule: a faction team engages the teams of factions it is hostile to; independents and
---marauders engage each other, and independents no one else; a team that is no faction's engages
---and is engaged as any other team (a different team is an enemy).
---@param own string|nil
---@param other string|nil
---@return boolean
function FactionManager:teamsHostile(own, other)
    local ownId = own and tonumber(own:match("^faction:(%d+)$"))
    local otherId = other and tonumber(other:match("^faction:(%d+)$"))
    if ownId and otherId then return FactionManager:isHostile(alive(ownId), alive(otherId)) end
    local independent = FactionManager.INDEPENDENT_TEAM
    if own == independent or other == independent then
        if own == other then return false end
        local factionId = ownId or otherId
        return factionId ~= nil and FactionManager:isMarauders(alive(factionId))
    end
    return own == nil or other ~= own
end

return FactionManager()
