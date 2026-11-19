local Registry            = require("Core.ECS.Registry")
local ConstructComponents = require("Modules.Constructs.Components")
local Components          = require("Modules.Navigation.Components")
local NavConfig           = require("Config.Navigation.NavConfig")
local NearbyBodies        = require("Shared.Helpers.NearbyBodies")
local Stagger             = require("Shared.Helpers.Stagger")

local RateClass = Enums.NavRateClass

---The highest rate among the zones the ship is inside that give a class.
---@param ship Entity
---@return NavRateClassId|nil class Nil when no zone the ship is inside gives one
local function zoneClass(ship)
    local membership = ship:get(Components.ZoneMembership)
    if not membership then return nil end
    local best = nil
    for _, id in ipairs(membership:ids()) do
        local zone = Registry:getEntityById(id)
        local given = zone and zone:get(Components.NavRateClass)
        -- A lower value is a higher rate.
        if given and (not best or given:getClass() < best) then best = given:getClass() end
    end
    return best
end

---Whether at least `NavConfig.trafficShips` ships other than `ship` are within the traffic radius.
---@param physics Physics
---@param ship Entity
---@return boolean
local function inTraffic(physics, ship)
    local isShip = function(entity) return entity:get(ConstructComponents.ControlInput) ~= nil end
    local ships = NearbyBodies.of(physics, ship, NavConfig.trafficRadius(), isShip)
    return #ships >= NavConfig.trafficShips
end

---@param world NavWorldComponent
---@param ship Entity
---@param slot integer
---@return NavRateClassId
local function judge(world, ship, slot)
    local fromZone = zoneClass(ship)
    if fromZone then return fromZone end
    local physics = world:getPhysics()
    if inTraffic(physics, ship) then return RateClass.High end
    if not NearbyBodies.anyWithin(physics, ship, world:getNav():sensingRange(slot)) then return RateClass.Low end
    return RateClass.Normal
end

--- Navigation rate system: gives each ship that has an agent a rate class, which sets how often its
--- avoidance runs and how often its plan is checked (`NavConfig.rateClasses`). A ship is High inside a
--- zone with a rate class and in dense traffic, Low with no body within its sensing range, Normal
--- otherwise. A ship is judged every `NavConfig.rateJudgePeriodS`, spread over ships by their ids, and
--- at once when it enters or leaves a zone. Design: doc/script/nav/navigation.md
---@class NavRateSystem
---@overload fun(): NavRateSystem
local NavRateSystem = Class("NavRateSystem", function(self)
    -- The seconds the system has run for: the clock the judging times are on.
    self.time = 0
end)

---The seconds between the checks of the ship's plan, from its class (Normal's before it is judged).
---@param ship Entity
---@return number
function NavRateSystem.replanPeriodS(ship)
    local rate = ship:get(Components.NavRate)
    return NavConfig.rateClasses[rate and rate.class or RateClass.Normal].replanPeriodS
end

---Gives the agent the class: its avoidance period. A change of class refreshes the agent, so that the
---new period does not wait for the old one to run out.
---@param world NavWorldComponent
---@param ship Entity
---@param slot integer
---@param class NavRateClassId
local function apply(world, ship, slot, class)
    local rate = ship:get(Components.NavRate)
    if rate and rate.class == class then return end
    local nav = world:getNav()
    -- Setting the period starts the agent's stagger over, so it is set only when the class changes.
    nav:setAvoidancePeriod(slot, NavConfig.rateClasses[class].avoidancePeriodS)
    if rate then
        nav:refresh(slot)
        rate.class = class
    else
        ship:add(Components.NavRate(class, 0))
    end
end

---Judges the ships that are due, or that entered or left a zone.
---@param world NavWorldComponent
---@param dt number Seconds
---@param heads OrderHead[] The ships flying orders, with their agents' slots
---@param changed table<EntityId, true> The ships that entered or left a zone since the last update
function NavRateSystem:update(world, dt, heads, changed)
    self.time = self.time + dt
    for _, head in ipairs(heads) do
        local ship = head.ship
        local rate = ship:get(Components.NavRate)
        -- An order that ended in this update's `beginOrder` took the ship's agent with it.
        local hasAgent = ship:get(Components.NavAgent) ~= nil
        if hasAgent and (not rate or changed[ship.id] or self.time >= rate.nextJudgeAt) then
            local first = rate == nil
            apply(world, ship, head.slot, judge(world, ship, head.slot))
            -- The first judging is followed by one put off by a share of the period, so that ships
            -- judged together are spread over frames; the next ones come a period apart.
            local after = first and Stagger.of(ship.id) * NavConfig.rateJudgePeriodS or NavConfig.rateJudgePeriodS
            ship:get(Components.NavRate).nextJudgeAt = self.time + after
        end
    end
end

return NavRateSystem
