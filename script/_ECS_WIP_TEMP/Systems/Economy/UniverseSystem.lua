-- Systems
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.GlobalStorage") --!temp path

-- Entities
local UniverseEntity = require("_ECS_WIP_TEMP.Entities.CelestialObjects.UniverseEntity")
local StarSystemEntity = require("_ECS_WIP_TEMP.Entities.CelestialObjects.StarSystemEntity")

-- Utilities
local QuickProfiler = require("_ECS_WIP_TEMP.Shared.Performance.QuickProfiler")

---@class UniverseSystem
---@overload fun(self: UniverseSystem) class internal
---@overload fun() class external
local UniverseSystem = Class(function(self)
    ---@diagnostic disable-next-line: invisible
    self:registerVars()
    ---@diagnostic disable-next-line: invisible
    self:registerEvents()
end)

---@private
function UniverseSystem:registerVars()
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

    -- Add entities to storage
    local universeEntityInfo = GlobalStorage:storeEntity(universe)
    local starSystemInfo = GlobalStorage:storeEntity(starSystem)

    -- Add star system as a child of universe
    ---@type EntityHierarchyComponent
    local universeHierarchyComponent = universe:findComponentByArchetype(Enums.ComponentArchetype.HierarchyComponent)
    universeHierarchyComponent:addChild(starSystemInfo)
end

function UniverseSystem:onPreRender() end

return UniverseSystem()
