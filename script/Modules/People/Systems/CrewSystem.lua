local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local DamageEvents = require("Shared.Helpers.DamageEvents")
local FactionManager = require("Modules.Factions.Managers.FactionManager")
local People = require("Modules.People.Components")
local PeopleManager = require("Modules.People.Managers.PeopleManager")

---Keeps posts and their people in step as things happen: the people of a post that is gone (a ship
---destroyed) survive it, lose it (`ShipLost`, toward their employer) and are free for a new post; a
---post whose commander is gone gets the highest rank aboard as commander; everyone on a post that is
---hit learns who attacked them (`Attacked`, toward the attacker's faction, else the attacker). Runs
---every `intervalS` at PreSim.
---@class CrewSystem
local CrewSystem = Class("CrewSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function CrewSystem:registerVars()
    self.profiler = QuickProfiler("CrewSystem", false, false)
    self.intervalS = 0.5
    self.untilNext = 0
end

function CrewSystem:registerEvents()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
    DamageEvents.subscribe(function(victim, attacker) self:onHit(victim, attacker) end)
end

---@param e EventData
function CrewSystem:onPreSim(e)
    self.untilNext = self.untilNext - e:deltaTime()
    if self.untilNext > 0 then return end
    self.untilNext = self.intervalS
    self:update()
end

function CrewSystem:update()
    self.profiler:start()
    local adrift = {}
    for person, assignment in Registry:view(People.Assignment) do
        if not Registry:hasEntity(Entity(assignment:getPost())) then adrift[#adrift + 1] = person end
    end
    for _, person in ipairs(adrift) do
        local employer = PeopleManager.employerOf(person)
        PeopleManager.experience(person, "ShipLost", employer and employer.id or nil)
        PeopleManager.unassign(person)
    end

    local leaderless = {}
    for post, crew in Registry:view(People.Crew) do
        local commander = crew:getCommander()
        if not commander or not Registry:hasEntity(Entity(commander)) then leaderless[#leaderless + 1] = post end
    end
    for _, post in ipairs(leaderless) do
        local first = PeopleManager.membersOf(post)[1]
        if first then PeopleManager.setCommander(post, first.person) end
    end
    self.profiler:stop()
end

---@param victim Entity
---@param attacker Entity|nil
function CrewSystem:onHit(victim, attacker)
    local post = DamageEvents.rootOf(victim)
    if not post or not post:get(People.Crew) or not attacker then return end
    local faction = FactionManager.factionOf(attacker)
    PeopleManager.experienceAll(post, "Attacked", (faction or attacker).id)
end

return CrewSystem()
