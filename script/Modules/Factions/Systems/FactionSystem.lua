local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local DamageEvents = require("Shared.Helpers.DamageEvents")
local WeaponCombatSystem = require("Modules.Constructs.Systems.WeaponCombatSystem")
local Factions = require("Modules.Factions.Components")
local FactionManager = require("Modules.Factions.Managers.FactionManager")

-- Design: doc/script/factions.md

---Keeps the factions' relations: they drift back to their baselines over time, and a faction whose
---member is hit by another faction's weapons turns against it. It gives the weapons the factions'
---hostility rule (`FactionManager.teamsHostile`). Runs at PreSim.
---@class FactionSystem
local FactionSystem = Class("FactionSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function FactionSystem:registerVars()
    self.profiler = QuickProfiler("FactionSystem", false, false)
    -- Disposition a second toward the baseline.
    self.driftPerSecond = 0.005
    -- What a hit costs the relation, at most once every `hitCooldownS` per pair of factions.
    self.hitPenalty = 0.05
    self.hitCooldownS = 1
    self.time = 0
    ---@type table<string, number> "victim:attacker" faction ids -> time of the last penalty
    self.lastHit = {}
end

function FactionSystem:registerEvents()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
    DamageEvents.subscribe(function(victim, attacker) self:onHit(victim, attacker) end)
    WeaponCombatSystem.setHostility(FactionManager.teamsHostile)
end

---@param e EventData
function FactionSystem:onPreSim(e)
    self:update(e:deltaTime())
end

---@param dt number
function FactionSystem:update(dt)
    if dt <= 0 then return end
    self.time = self.time + dt
    self.profiler:start()
    local step = self.driftPerSecond * dt
    for _, relations in Registry:view(Factions.FactionRelations) do
        for factionId, disposition in pairs(relations:getDispositions()) do
            local baseline = relations:getBaseline(factionId)
            if disposition < baseline then
                relations:setDisposition(factionId, math.min(baseline, disposition + step))
            elseif disposition > baseline then
                relations:setDisposition(factionId, math.max(baseline, disposition - step))
            end
        end
    end
    self.profiler:stop()
end

---@param victim Entity
---@param attacker Entity|nil
function FactionSystem:onHit(victim, attacker)
    local victimFaction = FactionManager.factionOf(victim)
    local attackerFaction = attacker and FactionManager.factionOf(attacker)
    if not victimFaction or not attackerFaction or victimFaction.id == attackerFaction.id then return end
    local key = victimFaction.id .. ":" .. attackerFaction.id
    if self.lastHit[key] and self.time - self.lastHit[key] < self.hitCooldownS then return end
    self.lastHit[key] = self.time
    FactionManager.adjust(victimFaction, attackerFaction, -self.hitPenalty)
end

return FactionSystem()
