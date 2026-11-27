local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local ScaleConfig = require("Config.Gen.UniverseScaleConfig")
local DamageEvents = require("Shared.Helpers.DamageEvents")
local PhysicsComponents = require("Modules.Physics.Components")
local Economy = require("Modules.Economy.Components")
local FactionManager = require("Modules.Factions.Managers.FactionManager")

-- Design: doc/script/factions.md

---How threatened each station is: a station's threat rises when a ship of its faction is hit within
---`radius` of it (by the damage over `damagePerThreat`) or raided there (by `raidThreat`), and halves
---every `halfLifeS`. The job board posts security jobs at the threatened stations, paid by them.
---@class ThreatSystem
local ThreatSystem = Class("ThreatSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function ThreatSystem:registerVars()
    self.profiler = QuickProfiler("ThreatSystem", false, false)
    self.radius = ScaleConfig:toGameUnits(150e3, "starSystem")
    self.damagePerThreat = 100
    self.raidThreat = 3
    self.halfLifeS = 180
    ---@type table<integer, number> Station id -> threat
    self.threat = {}
end

function ThreatSystem:registerEvents()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
    DamageEvents.subscribe(function(victim, _, amount) self:onIncident(victim, amount / self.damagePerThreat) end)
end

---@param e EventData
function ThreatSystem:onPreSim(e)
    local dt = e:deltaTime()
    if dt <= 0 then return end
    local keep = 0.5 ^ (dt / self.halfLifeS)
    for id, threat in pairs(self.threat) do
        threat = threat * keep
        self.threat[id] = threat > 0.01 and threat or nil
    end
end

---Raises the threat of the stations of the victim's faction near it.
---@param victim Entity
---@param amount number Threat to add
function ThreatSystem:onIncident(victim, amount)
    local faction = FactionManager.factionOf(victim)
    local rbCmp = victim:get(PhysicsComponents.RigidBody)
    local body = rbCmp and rbCmp:getRigidBody()
    if not faction or not body or amount <= 0 then return end
    local at = body:getPos()
    for station in Registry:view(Economy.Marketplace) do
        local stationBody = station:get(PhysicsComponents.RigidBody) and station:get(PhysicsComponents.RigidBody):getRigidBody()
        if stationBody and FactionManager.factionOf(station) == faction then
            local p = stationBody:getPos()
            local dx, dy, dz = p.x - at.x, p.y - at.y, p.z - at.z
            if math.sqrt(dx * dx + dy * dy + dz * dz) <= self.radius then
                self.threat[station.id] = (self.threat[station.id] or 0) + amount
            end
        end
    end
end

---A raid on the victim: its faction's stations near it feel it.
---@param victim Entity
function ThreatSystem:reportRaid(victim)
    self:onIncident(victim, self.raidThreat)
end

---@param station Entity
---@return number threat
function ThreatSystem:threatOf(station)
    return self.threat[station.id] or 0
end

---@return table<integer, number> threat Station id -> threat, for the threatened stations
function ThreatSystem:getThreats()
    return self.threat
end

return ThreatSystem()
