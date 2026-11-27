local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local Words = require("Shared.Generation.Words")
local CrewRoles = require("Shared.Definitions.CrewRoleDefs")
local LifeEvents = require("Shared.Definitions.LifeEventDefs")
local Economy = require("Modules.Economy.Components")
local AccountManager = require("Modules.Economy.Managers.AccountManager")
local Factions = require("Modules.Factions.Components")
local People = require("Modules.People.Components")
local NpcEntity = require("Modules.People.Entities.NpcEntity")

-- Design: doc/script/people.md

---People (NPCs) and the posts they man. Creates people, assigns them to posts in roles
---(`CrewRoleDefs`), passes command, pays their shares and tells them what happened to them
---(`LifeEventDefs`). A post is any entity people serve on: a ship now, a station later.
---
---The commander of a post carries its decision-making: the component types registered with
---`registerCommandComponent` (an economic AI, say) live on the commander, not on the post, and move
---with command. So the person decides, by what they are and what they lived through, and a new
---commander (promoted, or after a mutiny) decides from then on.
---@class PeopleManager
---@overload fun(self: PeopleManager): PeopleManager class internal
---@overload fun(): PeopleManager class external
local PeopleManager = Class("PeopleManager", function(self) end)

---@type table[] Component types that ride on a post's commander
local commandComponents = {}

---@param id integer|Entity|nil
---@return Entity|nil
local function alive(id)
    if id == nil then return nil end
    local entity = rawtype(id) == "number" and Entity(id) or id
    return Registry:hasEntity(entity) and entity or nil
end

---Simulated seconds, for careers and memories.
---@return number
local function now()
    return require("Modules.Economy.Systems.MarketplaceSystem"):now()
end

---Makes a component type ride on a post's commander (it moves when command passes).
---@param componentType table
function PeopleManager:registerCommandComponent(componentType)
    for _, known in ipairs(commandComponents) do
        if known == componentType then return end
    end
    commandComponents[#commandComponents + 1] = componentType
end

---A name of a few syllables, capitalised.
---@param rng RNG
---@param maxLength integer
---@return string
local function word(rng, maxLength)
    local name = Words.genName(rng):lower()
    if #name > maxLength then name = name:sub(1, maxLength) end
    return name:sub(1, 1):upper() .. name:sub(2)
end

---@param rng RNG
---@return string givenName, string familyName
function PeopleManager:nameFor(rng)
    return word(rng, 7), word(rng, 9)
end

---Makes a person.
---@param spec { rng: RNG, faction: Entity|nil, employer: Entity|nil, credits: number|nil }
---@return Entity person
function PeopleManager:create(spec)
    local given, family = PeopleManager:nameFor(spec.rng)
    local person = NpcEntity(given, family, spec.credits, spec.employer and spec.employer.id or nil)
    if spec.faction then person:add(Factions.FactionMembership(spec.faction.id)) end
    if spec.employer then PeopleManager:experience(person, "Hired", spec.employer.id) end
    return person
end

---Tells a person what happened to them: their morale and their opinion of `subjectId` move by the
---event's definition, and an event that counts is remembered.
---@param person Entity
---@param eventId string A `LifeEventDefs` id
---@param subjectId integer|nil
function PeopleManager:experience(person, eventId, subjectId)
    local event = LifeEvents[eventId]
    if not event or not alive(person) then return end
    local persona = person:get(People.Person)
    if persona then persona:setMorale(persona:getMorale() + event.morale) end
    local memory = person:get(People.Memory)
    if not memory then return end
    if subjectId and event.opinion ~= 0 then
        memory:setOpinion(subjectId, memory:getOpinion(subjectId) + event.opinion)
    end
    if event.remember then memory:remember({ event = eventId, subjectId = subjectId, at = now() }) end
end

---Tells everyone on a post what happened to them.
---@param post Entity
---@param eventId string
---@param subjectId integer|nil
function PeopleManager:experienceAll(post, eventId, subjectId)
    for _, entry in ipairs(PeopleManager:membersOf(post)) do
        PeopleManager:experience(entry.person, eventId, subjectId)
    end
end

-- Posts -------------------------------------------------------------------------------------------

---@param post Entity
---@return { person: Entity, role: string }[] members Highest rank first
function PeopleManager:membersOf(post)
    local crew = post and alive(post) and post:get(People.Crew)
    local list = {}
    if not crew then return list end
    for id, role in pairs(crew:getMembers()) do
        local person = alive(id)
        if person then list[#list + 1] = { person = person, role = role } end
    end
    table.sort(list, function(a, b)
        local ra, rb = CrewRoles[a.role] and CrewRoles[a.role].rank or 0, CrewRoles[b.role] and CrewRoles[b.role].rank or 0
        if ra ~= rb then return ra > rb end
        return a.person.id < b.person.id
    end)
    return list
end

---@param post Entity|nil
---@return Entity|nil commander
function PeopleManager:commanderOf(post)
    local crew = post and alive(post) and post:get(People.Crew)
    return crew and alive(crew:getCommander()) or nil
end

---A component of the post's commander: the decision-making the post runs on.
---@param post Entity|nil
---@param componentType table
---@return table|nil
function PeopleManager:commandOf(post, componentType)
    local commander = PeopleManager:commanderOf(post)
    return commander and commander:get(componentType) or nil
end

---@param person Entity
---@return Entity|nil post
function PeopleManager:postOf(person)
    local assignment = alive(person) and person:get(People.Assignment)
    return assignment and alive(assignment:getPost()) or nil
end

---@param person Entity
---@return string|nil role
function PeopleManager:roleOf(person)
    local assignment = alive(person) and person:get(People.Assignment)
    return assignment and assignment:getRole() or nil
end

---Puts a person on a post in a role. A commanding role takes command when the post has no commander.
---@param person Entity
---@param post Entity
---@param role string A `CrewRoleDefs` id
function PeopleManager:assign(person, post, role)
    assert(CrewRoles[role], "PeopleManager: no crew role " .. tostring(role))
    if person:get(People.Assignment) then PeopleManager:unassign(person) end
    local crew = post:get(People.Crew) or post:add(People.Crew())
    crew:setMember(person.id, role)
    person:add(People.Assignment(post.id, role))
    local career = person:get(People.Career)
    if career then
        local history = career:getHistory()
        history[#history + 1] = { postId = post.id, role = role, employerId = career:getEmployer(), from = now() }
    end
    if CrewRoles[role].commands and not PeopleManager:commanderOf(post) then
        PeopleManager:setCommander(post, person)
    end
end

---Takes a person off their post. If they commanded it, command passes to the highest rank aboard.
---@param person Entity
function PeopleManager:unassign(person)
    local assignment = person:get(People.Assignment)
    if not assignment then return end
    local post = alive(assignment:getPost())
    local career = person:get(People.Career)
    local current = career and career:getCurrent()
    if current then current.to = now() end
    person:remove(People.Assignment)
    local crew = post and post:get(People.Crew)
    if crew then
        crew:setMember(person.id, nil)
        if crew:getCommander() == person.id then
            -- The highest rank aboard takes over, in the role the commander held.
            local successor = PeopleManager:membersOf(post)[1]
            PeopleManager:setCommander(post, successor and successor.person or nil, person)
            if successor and CrewRoles[assignment:getRole()] and CrewRoles[assignment:getRole()].commands then
                PeopleManager:promote(successor.person, assignment:getRole())
            end
        end
    end
    -- Off a post, nobody decides for it through them.
    for _, componentType in ipairs(commandComponents) do
        if person:get(componentType) then person:remove(componentType) end
    end
end

---Gives command of the post to `person` (nil: nobody), moving the decision-making components from
---the commander before (or `from`, one just taken off the post) to them. Everyone aboard learns of it.
---@param post Entity
---@param person Entity|nil
---@param from Entity|nil
function PeopleManager:setCommander(post, person, from)
    local crew = post:get(People.Crew) or post:add(People.Crew())
    local before = from or alive(crew:getCommander())
    if before and person and before.id == person.id then return end
    crew:setCommander(person and person.id or nil)
    for _, componentType in ipairs(commandComponents) do
        local component = before and before:get(componentType)
        if component then
            before:remove(componentType)
            if person and not person:get(componentType) then person:add(component) end
        end
    end
    if person then PeopleManager:experienceAll(post, "CommandChanged", person.id) end
end

---Changes a person's role on their post (a promotion, say). A commanding role takes command.
---@param person Entity
---@param role string
function PeopleManager:promote(person, role)
    local post = PeopleManager:postOf(person)
    if not post or not CrewRoles[role] then return end
    post:get(People.Crew):setMember(person.id, role)
    person:get(People.Assignment):setRole(role)
    PeopleManager:experience(person, "Promoted", PeopleManager:employerOf(person) and PeopleManager:employerOf(person).id)
    if CrewRoles[role].commands then PeopleManager:setCommander(post, person) end
end

---Mans a post: a person for each role, hired with `spec.hire` (someone available) or made new. The
---post's owner employs them, unless `spec.selfEmployed` (the commander then owns the post).
---@param post Entity
---@param spec { roles: string[], rng: RNG, faction: Entity|nil, employer: Entity|nil, credits: number|nil, hire: (fun(role: string): Entity|nil)|nil }
---@return Entity|nil commander
function PeopleManager:man(post, spec)
    for _, role in ipairs(spec.roles) do
        local person = spec.hire and spec.hire(role)
        if person then
            person:get(People.Career):setEmployer(spec.employer and spec.employer.id or nil)
            if spec.employer then PeopleManager:experience(person, "Hired", spec.employer.id) end
        else
            person = PeopleManager:create({ rng = spec.rng, faction = spec.faction, employer = spec.employer, credits = spec.credits })
        end
        PeopleManager:assign(person, post, role)
    end
    return PeopleManager:commanderOf(post)
end

---Mans a post as an independent: its commander owns it (the caller gives the post its ownership),
---starts with `credits` in their wallet and employs the rest of the crew.
---@param post Entity
---@param spec { roles: string[], rng: RNG, credits: number|nil }
---@return Entity commander
function PeopleManager:manIndependent(post, spec)
    local commander = assert(PeopleManager:man(post, { roles = spec.roles, rng = spec.rng }),
        "PeopleManager:manIndependent: no commanding role")
    AccountManager:deposit(commander:get(Economy.PlayerBankAccount), spec.credits or 0)
    for _, member in ipairs(PeopleManager:membersOf(post)) do
        if member.person.id ~= commander.id then member.person:get(People.Career):setEmployer(commander.id) end
    end
    return commander
end

-- Work and pay ------------------------------------------------------------------------------------

---@param person Entity
---@return Entity|nil employer
function PeopleManager:employerOf(person)
    local career = alive(person) and person:get(People.Career)
    return career and alive(career:getEmployer()) or nil
end

---People an employer has on its books but on no post.
---@param employer Entity
---@return Entity[]
function PeopleManager:unassigned(employer)
    local list = {}
    for person, career in Registry:view(People.Career) do
        if career:getEmployer() == employer.id and not person:get(People.Assignment) then list[#list + 1] = person end
    end
    table.sort(list, function(a, b) return a.id < b.id end)
    return list
end

---What a post earned on a job of `activity`: everyone aboard gains experience, and each employed
---member gets their role's share, paid by the post's owner when it can. A self-employed member (an
---owner-captain) keeps what the post earns in their own account already.
---@param post Entity
---@param activity string "trade", "mining", "patrol"...
---@param credits number
function PeopleManager:reportEarnings(post, activity, credits)
    local ownerAccount = AccountManager:accountOf(post)
    local commander = PeopleManager:commanderOf(post)
    for _, entry in ipairs(PeopleManager:membersOf(post)) do
        local person, role = entry.person, CrewRoles[entry.role]
        local career = person:get(People.Career)
        career:addExperience(activity, 1)
        local employer = PeopleManager:employerOf(person)
        local wallet = person:get(Economy.PlayerBankAccount)
        if employer and role and credits > 0 and wallet ~= ownerAccount then
            local share = math.floor(credits * role.share)
            if share > 0 and AccountManager:transfer(ownerAccount, wallet, share, "wages") then
                career:addEarned(share)
                PeopleManager:experience(person, "Paid", employer.id)
            end
        end
        if commander and commander.id ~= person.id then PeopleManager:experience(person, "JobDone", commander.id) end
    end
end

return PeopleManager()
