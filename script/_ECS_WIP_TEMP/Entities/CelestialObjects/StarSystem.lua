local GlobalStorage = require("_ECS_WIP_TEMP.Systems.GlobalStorage") --!temp path

-- Entities
local Entity = require("_ECS_WIP_TEMP.Entities.Entity")                      --!temp path
local Star = require("_ECS_WIP_TEMP.Entities.CelestialObjects.Star")         --!temp path
local Planet = require("_ECS_WIP_TEMP.Entities.CelestialObjects.Planet")     --!temp path
local Asteroid = require("_ECS_WIP_TEMP.Entities.CelestialObjects.Asteroid") --!temp path

-- Components
local RandomNumberGeneratorComponent = require("_ECS_WIP_TEMP.Components.Core.RandomNumberGenerator") --!temp path
local NameComponent = require("_ECS_WIP_TEMP.Components.Core.EntityName")                             --!temp path
local HierarchyComponent = require("_ECS_WIP_TEMP.Components.Core.EntityHierarchy")                   --!temp path
local PlayerListComponent = require("_ECS_WIP_TEMP.Components.Economy.PlayerList")                    --!temp path

-- Utils
local Words = require('Systems.Gen.Words')

---@class StarSystem: Entity
---@overload fun(self: StarSystem, seed: integer): StarSystem subclass internal
---@overload fun(seed: integer): StarSystem subclass external
local StarSystem = Subclass(Entity, function(self, seed)
    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.StarSystemEntity)

    -- RandomNumberGeneratorComponent
    local _, rngComponent = self:addComponent(RandomNumberGeneratorComponent(seed, true))
    ---@cast rngComponent RandomNumberGeneratorComponent
    -- Name Component
    local starSystemName = Words.getCoolName(rngComponent:getRNG())
    self:addComponent(NameComponent(starSystemName))
    -- PlayerList Component
    self:addComponent(PlayerListComponent())
    --todo: include projectiles

    -- Hierarchy/Children Component
    local _, hierarchyComponent = self:addComponent(HierarchyComponent(self))
    ---@cast hierarchyComponent EntityHierarchyComponent

    -- Add Children
    --local star = Star(seed)
    --GlobalStorage:storeEntity(star)
    --hierarchyComponent:addChild({ id = star:getGuid(), archetype = star:getArchetype() })
    --
    --local planet = Planet(seed)
    --GlobalStorage:storeEntity(planet)
    --hierarchyComponent:addChild({ id = planet:getGuid(), archetype = planet:getArchetype() })
    --
    --local asteroid = Asteroid(seed)
    --GlobalStorage:storeEntity(asteroid)
    --hierarchyComponent:addChild({ id = planet:getGuid(), archetype = planet:getArchetype() })
end)

return StarSystem
