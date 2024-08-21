-- Systems
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.GlobalStorage") --!temp path

-- Entities
local UniverseEntity = require("_ECS_WIP_TEMP.Entities.CelestialObjects.UniverseEntity")
local StarSystemEntity = require("_ECS_WIP_TEMP.Entities.CelestialObjects.StarSystemEntity")
local StarEntity = require("_ECS_WIP_TEMP.Entities.CelestialObjects.StarEntity")
local PlanetEntity = require("_ECS_WIP_TEMP.Entities.CelestialObjects.PlanetEntity")

-- Utilities
local QuickProfiler = require("_ECS_WIP_TEMP.Shared.Performance.QuickProfiler")

---@class UniverseSystem
---@overload fun(self: UniverseSystem): UniverseSystem class internal
---@overload fun(): UniverseSystem class external
local UniverseSystem = Class(function(self)
    ---@diagnostic disable-next-line: invisible
    self:registerVars()
    ---@diagnostic disable-next-line: invisible
    self:registerEvents()
end)

---@private
function UniverseSystem:registerVars()
    ---@private
    self.profiler = QuickProfiler("UniverseSystem", false, false)
end

---@private
function UniverseSystem:registerEvents()
    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
end

function UniverseSystem:createUniverse(seed)
    -- Construct universe entity
    local universe = UniverseEntity(seed)
    local universeRNG = RNG.Create(seed):managed()
    --todo: store rng
    --todo: rng info component

    -- Construct star system entity
    local starSystemSeed = universeRNG:get64()
    local starSystem = StarSystemEntity(starSystemSeed)
    local starSystemRNG = RNG.Create(starSystemSeed):managed()
    --todo: store rng
    --todo: rng info component

    local starSeed = starSystemRNG:get64()
    local star = StarEntity(starSeed)

    local planetASeed = starSystemRNG:get64()
    local planetA = PlanetEntity(planetASeed)

    local planetBSeed = starSystemRNG:get64()
    local planetB = PlanetEntity(planetBSeed)

    -- Add entities to storage
    local universeEntityInfo = GlobalStorage:storeEntity(universe)
    local starSystemEntityInfo = GlobalStorage:storeEntity(starSystem)
    local starEntityInfo = GlobalStorage:storeEntity(star)
    local planetAEntityInfo = GlobalStorage:storeEntity(planetA)
    local planetBEntityInfo = GlobalStorage:storeEntity(planetB)

    -- Add star system as a child of universe
    ---@type EntityHierarchyComponent
    local universeHierarchyComponent = universe:findComponentByArchetype(Enums.ComponentArchetype.HierarchyComponent)
    universeHierarchyComponent:addChild(starSystemEntityInfo)

    -- Add star system children
    ---@type EntityHierarchyComponent
    local starSystemHierarchyComponent = starSystem:findComponentByArchetype(Enums.ComponentArchetype.HierarchyComponent)
    starSystemHierarchyComponent:addChild(starEntityInfo)
    starSystemHierarchyComponent:addChild(planetAEntityInfo)
    starSystemHierarchyComponent:addChild(planetBEntityInfo)
end

---@private
function UniverseSystem:onPreRender() end

return UniverseSystem()
