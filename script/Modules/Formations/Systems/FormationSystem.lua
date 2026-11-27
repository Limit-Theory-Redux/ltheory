local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local ScaleConfig = require("Config.Gen.UniverseScaleConfig")
local FormationDefs = require("Shared.Definitions.FormationDefs")
local Nav = require("Modules.Navigation")
local PhysicsComponents = require("Modules.Physics.Components")
local FlightProfile = require("Modules.Constructs.Components.FlightProfileComponent")
local Formations = require("Modules.Formations.Components")

-- Design: doc/script/formations.md

---Flies ships in formation. A formation is an entity (`FormationComponent`) with a shape
---(`FormationDefs`) and its ships in slot order; the first leads and flies its own orders, and each
---other ship holds its slot with a navigation Follow order on the leader, so the wing turns and
---speeds with it. The leader is held to `paceShare` of the slowest member's top speed
---(`Nav.setMaxSpeed`), so the outer slots can keep up through turns. The slots are spaced by the
---wing's largest ship.
---
---Every `intervalS` the system drops members that are gone, gives the lead to the next ship when the
---leader is gone, and gives out the slots again when the wing changed. A wingman's Follow is a
---Routine order: an order of higher priority (a fight) takes over its ship, and it holds its slot
---again when that ends. Runs at PreSim.
---@class FormationSystem
local FormationSystem = Class("FormationSystem", function(self)
    self:registerVars()
    self:registerEvents()
end)

function FormationSystem:registerVars()
    self.profiler = QuickProfiler("FormationSystem", false, false)
    self.intervalS = 0.25
    self.untilNext = 0
    ---@type Entity[] Orders cancelled, released once they have ended
    self.cancelled = {}
end

function FormationSystem:registerEvents()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
end

---@param id integer|nil
---@return Entity|nil
local function alive(id)
    if not id then return nil end
    local entity = Entity(id)
    return Registry:hasEntity(entity) and entity or nil
end

-- API ------------------------------------------------------------------------------------------------

---Makes a formation of `ships` in slot order (the first leads) in `shape`.
---@param shape string A `FormationDefs` id
---@param ships Entity[]
---@param options { spacingRadii: number|nil, paceShare: number|nil }|nil
---@return Entity formation
function FormationSystem:create(shape, ships, options)
    assert(FormationDefs[shape], "FormationSystem: no formation shape " .. tostring(shape))
    options = options or {}
    local formation = Entity.Create("Formation", Formations.Formation(shape, options.spacingRadii, options.paceShare))
    for _, ship in ipairs(ships) do self:join(formation, ship) end
    return formation
end

---Adds a ship to the formation, at `slot` (1 leads; the last slot when nil). It leaves any formation
---it was in.
---@param formation Entity
---@param ship Entity
---@param slot integer|nil
function FormationSystem:join(formation, ship, slot)
    self:leave(ship)
    local component = formation:get(Formations.Formation)
    slot = math.max(1, math.min(slot or #component.members + 1, #component.members + 1))
    table.insert(component.members, slot, ship.id)
    component.dirty = true
    ship:add(Formations.FormationMember(formation.id))
end

---Takes a ship out of its formation: its Follow ends and, if it led, the next ship leads.
---@param ship Entity
function FormationSystem:leave(ship)
    local member = ship:get(Formations.FormationMember)
    if not member then return end
    self:dropOrder(member)
    local formation = alive(member:getFormation())
    local component = formation and formation:get(Formations.Formation)
    if component then
        for i, id in ipairs(component.members) do
            if id == ship.id then table.remove(component.members, i) break end
        end
        component.dirty = true
    end
    if component and component.pacedId == ship.id then
        Nav.setMaxSpeed(ship, nil)
        component.pacedId = nil
    end
    ship:remove(Formations.FormationMember)
end

---Ends the formation: every ship leaves it, and its entity is destroyed.
---@param formation Entity
function FormationSystem:disband(formation)
    local component = formation:get(Formations.Formation)
    if not component then return end
    for _, id in ipairs({ table.unpack(component.members) }) do
        local ship = alive(id)
        if ship then self:leave(ship) end
    end
    Registry:destroyEntity(formation)
end

---@param ship Entity
---@return Entity|nil formation
function FormationSystem:formationOf(ship)
    local member = ship:get(Formations.FormationMember)
    return member and alive(member:getFormation()) or nil
end

---@param ship Entity
---@return Entity|nil leader The leader of the ship's formation (maybe the ship itself)
function FormationSystem:leaderOf(ship)
    local formation = self:formationOf(ship)
    local component = formation and formation:get(Formations.Formation)
    return component and alive(component:getLeader()) or nil
end

---Whether the ship holds a slot in a formation it doesn't lead: its own flight is the formation's.
---@param ship Entity
---@return boolean
function FormationSystem:isWingman(ship)
    local leader = self:leaderOf(ship)
    return leader ~= nil and leader.id ~= ship.id
end

-- Update ---------------------------------------------------------------------------------------------

---@param e EventData
function FormationSystem:onPreSim(e)
    self:releaseCancelled()
    self.untilNext = self.untilNext - e:deltaTime()
    if self.untilNext > 0 then return end
    self.untilNext = self.intervalS
    self.profiler:start()
    local empty = {}
    for formation, component in Registry:view(Formations.Formation) do
        self:tend(formation, component)
        if #component.members == 0 then empty[#empty + 1] = formation end
    end
    for _, formation in ipairs(empty) do Registry:destroyEntity(formation) end
    self.profiler:stop()
end

---Drops lost members, keeps the lead filled, and gives out the slots when the wing changed or a
---wingman's Follow has ended.
---@param formation Entity
---@param component FormationComponent
function FormationSystem:tend(formation, component)
    local kept = {}
    for _, id in ipairs(component.members) do
        if alive(id) then kept[#kept + 1] = id end
    end
    if #kept ~= #component.members then component.members, component.dirty = kept, true end
    if #kept == 0 then return end

    local leader = alive(kept[1])
    if component.leaderId ~= leader.id then
        component.leaderId, component.dirty = leader.id, true
        -- The new leader flies its own orders: its Follow ends.
        self:dropOrder(leader:get(Formations.FormationMember))
    end
    for i = 2, #kept do
        local member = alive(kept[i]):get(Formations.FormationMember)
        local order = member and member.order
        if order and (not Registry:hasEntity(order) or self:hasEnded(order)) then
            self:dropOrder(member)
            component.dirty = true
        end
    end
    if component.dirty then self:assignSlots(component, leader) end
end

---@param order Entity
---@return boolean
function FormationSystem:hasEnded(order)
    local status = order:get(Nav.Components.OrderStatus)
    return status == nil or status:hasEnded()
end

---Gives each wingman its slot, as a Follow on the leader, and paces the leader.
---@param component FormationComponent
---@param leader Entity
function FormationSystem:assignSlots(component, leader)
    component.dirty = false
    local shape = FormationDefs[component.shape]
    local largest, slowest = 0, math.huge
    for _, id in ipairs(component.members) do
        local ship = alive(id)
        local body = ship:get(PhysicsComponents.RigidBody):getRigidBody()
        largest = math.max(largest, body:getBoundingRadius())
        local profile = FlightProfile.ProfileOf(ship)
        if profile then slowest = math.min(slowest, profile.maxSpeed) end
    end
    local metresPerUnit = 1 / ScaleConfig:toGameUnits(1, "starSystem")
    local spacingM = largest * component.spacingRadii * metresPerUnit

    for i = 2, #component.members do
        local ship = alive(component.members[i])
        local member = ship:get(Formations.FormationMember)
        local right, up, forward = shape.slot(i - 1)
        local offset = Vec3d(right * spacingM, up * spacingM, forward * spacingM)
        local same = member.offset and member.offset.x == offset.x and member.offset.y == offset.y and member.offset.z == offset.z
        if not (member.order and same and member.leaderId == leader.id) then
            self:dropOrder(member)
            member.order = Nav.order(ship, Nav.Orders.Follow { target = leader, offset = offset })
            member.offset, member.leaderId = offset, leader.id
        end
        member.spacingM = spacingM
    end

    -- The pace: a share of the slowest member's top speed, so the outer slots keep up in turns.
    if component.pacedId and component.pacedId ~= leader.id then
        local before = alive(component.pacedId)
        if before then Nav.setMaxSpeed(before, nil) end
    end
    if #component.members > 1 and slowest < math.huge then
        Nav.setMaxSpeed(leader, slowest * component.paceShare)
        component.pacedId = leader.id
    else
        Nav.setMaxSpeed(leader, nil)
        component.pacedId = nil
    end
end

---Ends a member's Follow, releasing it once it has ended.
---@param member FormationMemberComponent|nil
function FormationSystem:dropOrder(member)
    local order = member and member.order
    if not member then return end
    member.order, member.offset, member.leaderId = nil, nil, nil
    if not order or not Registry:hasEntity(order) then return end
    if self:hasEnded(order) then
        Nav.release(order)
    else
        Nav.cancel(order)
        self.cancelled[#self.cancelled + 1] = order
    end
end

function FormationSystem:releaseCancelled()
    if #self.cancelled == 0 then return end
    local waiting = {}
    for _, order in ipairs(self.cancelled) do
        if Registry:hasEntity(order) then
            if self:hasEnded(order) then Nav.release(order) else waiting[#waiting + 1] = order end
        end
    end
    self.cancelled = waiting
end

return FormationSystem()
