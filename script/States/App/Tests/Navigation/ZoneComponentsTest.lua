-- Checks the optional navigation components of a zone: the speed limit and the path cost keep their
-- values and reject values outside their range, the tags use the shared tag component, and only
-- zones that carry a component are found by a query for it. Logs each failed check and ends with
-- a failure exit code.

local Application          = require("States.Application")
local Registry             = require("Core.ECS.Registry")
local CoreComponents       = require("Modules.Core.Components")
local NavigationComponents = require("Modules.Navigation.Components")
local SpatialEntities      = require("Modules.Spatial.Entities")
local Tags                 = require("Shared.Registries.Tags")
local UniverseScaleConfig  = require("Config.Gen.UniverseScaleConfig")

---@class NavigationZoneComponentsTest: Application
local ZoneComponentsTest = Subclass("NavigationZoneComponentsTest", Application)

local SPEED_LIMIT_MS = 250 -- m/s, authored
local PATH_COST      = 2.5

function ZoneComponentsTest:check(ok, what)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        Log.Error("NavigationZoneComponentsTest: %s failed", what)
    end
end

---Collect the ids of the entities that a query for `component` finds. A query returns new handles,
---so the set is keyed by id.
local function found(component)
    local set = {}
    for entity in Registry:iterEntities(component) do
        set[entity.id] = true
    end
    return set
end

function ZoneComponentsTest:checkValues()
    local limit = UniverseScaleConfig:toGameUnits(SPEED_LIMIT_MS)
    local speed = NavigationComponents.SpeedLimit(limit)
    self:check(speed:getLimit() == limit, "a speed limit keeps its value")
    speed:setLimit(2 * limit)
    self:check(speed:getLimit() == 2 * limit, "a speed limit can change")

    self:check(NavigationComponents.PathCost(PATH_COST):getCost() == PATH_COST, "a path cost keeps its value")
    self:check(NavigationComponents.PathCost(0):getCost() == 0, "a path cost of zero is allowed")
end

function ZoneComponentsTest:checkRanges()
    local SpeedLimit, PathCost = NavigationComponents.SpeedLimit, NavigationComponents.PathCost
    self:check(not pcall(SpeedLimit, 0), "a speed limit of zero is rejected")
    self:check(not pcall(SpeedLimit, -1), "a negative speed limit is rejected")
    self:check(not pcall(SpeedLimit, nil), "a missing speed limit is rejected")
    self:check(not pcall(SpeedLimit, "fast"), "a speed limit that is not a number is rejected")
    self:check(not pcall(PathCost, -0.5), "a negative path cost is rejected")
    self:check(not pcall(PathCost, nil), "a missing path cost is rejected")
end

function ZoneComponentsTest:checkQueries()
    Tags:new("ZoneKind", { "AsteroidField", "HostileSpace" })

    local belt = SpatialEntities.Zone()
    belt:add(NavigationComponents.SpeedLimit(UniverseScaleConfig:toGameUnits(SPEED_LIMIT_MS)))
    belt:add(NavigationComponents.PathCost(PATH_COST))
    belt:add(CoreComponents.Tag("AsteroidField"))

    local plain = SpatialEntities.Zone()
    local costOnly = SpatialEntities.Zone()
    costOnly:add(NavigationComponents.PathCost(1))

    local limited = found(NavigationComponents.SpeedLimit)
    self:check(limited[belt.id] and not limited[plain.id] and not limited[costOnly.id],
        "a query for the speed limit finds only the zones that have one")
    local costly = found(NavigationComponents.PathCost)
    self:check(costly[belt.id] and costly[costOnly.id] and not costly[plain.id],
        "a query for the path cost finds only the zones that have one")

    local tags = belt:get(CoreComponents.Tag)
    self:check(tags:hasTag("AsteroidField") and not tags:hasTag("HostileSpace"), "a zone carries its tags")
    self:check(plain:get(CoreComponents.Tag) == nil, "a zone without tags has no tag component")
end

function ZoneComponentsTest:onInit()
    self.passed, self.failed = 0, 0

    self:checkValues()
    self:checkRanges()
    self:checkQueries()

    Log.Info("NavigationZoneComponentsTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return ZoneComponentsTest
