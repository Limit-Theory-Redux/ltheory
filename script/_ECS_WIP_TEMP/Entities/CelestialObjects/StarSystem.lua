local GlobalStorage = require("Systems.GlobalStorage")

-- Entities
local Entity = require("Entities.Entity")
local Star = require("Entities.CelestialObjects.Star")
local Planet = require("Entities.CelestialObjects.Planet")
local Asteroid = require("Entities.CelestialObjects.Asteroid")

-- Components
local RandomNumberGeneratorComponent = require("Components.RandomNumberGenerator")
local NameComponent = require("Components.EntityName")
local TypeComponent = require("Components.EntityType")
local HierarchyComponent = require("Components.Hierarchy")
local PlayerListComponent = require("Components.PlayerList")
local EconomyComponent = require("Components.Economy")

-- Utils
local Words = require('Systems.Gen.Words')

---@class StarSystem: Entity
local StarSystem = Subclass(Entity, function(self, seed)
    ---@cast self StarSystem

    -- Set Entity Archetype
    self:setArchetype(Enums.EntityArchetype.StarSystemEntity)

    -- RandomNumberGeneratorComponent
    local _, rngComponent = self:addComponent(RandomNumberGeneratorComponent(seed, true))
    ---@cast rngComponent RandomNumberGeneratorComponent
    -- Name Component
    local starSystemName = Words.getCoolName(rngComponent:getRNG())
    self:addComponent(NameComponent(starSystemName))
    -- Economy Component
    self:addComponent(EconomyComponent()) --todo
    -- PlayerList Component
    self:addComponent(PlayerListComponent())
    --todo: include projectiles

    -- Hierarchy/Children Component
    local _, hierarchyComponent = self:addComponent(HierarchyComponent(self))
    ---@cast hierarchyComponent EntityHierarchyComponent

    -- Add Children
    local star = Star(seed)
    GlobalStorage:storeEntity(star)
    hierarchyComponent:addChild({ id = star:getGuid(), archetype = star:getArchetype() })

    local planet = Planet(seed)
    GlobalStorage:storeEntity(planet)
    hierarchyComponent:addChild({ id = planet:getGuid(), archetype = planet:getArchetype() })

    local asteroid = Asteroid(seed)
    GlobalStorage:storeEntity(asteroid)
    hierarchyComponent:addChild({ id = planet:getGuid(), archetype = planet:getArchetype() })
end)

return StarSystem
