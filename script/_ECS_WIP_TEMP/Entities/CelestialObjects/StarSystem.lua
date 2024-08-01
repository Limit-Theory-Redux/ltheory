local Entity = require("Entities.Entity")

-- Entities
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

    -- RandomNumberGeneratorComponent
    local rngComponentIndex = self:addComponent(RandomNumberGeneratorComponent(seed, true))
    local rngComponent = self:getComponent(rngComponentIndex)
    ---@cast rngComponent RandomNumberGeneratorComponent

    -- Name Component
    local starSystemName = Words.getCoolName(rngComponent:getRNG())
    self:addComponent(NameComponent(starSystemName))
    -- Type Component
    self:addComponent(TypeComponent(Enums.EntityType.StarSystem))
    -- Economy Component
    self:addComponent(EconomyComponent()) --todo
    -- PlayerList Component
    self:addComponent(PlayerListComponent())
    --todo: include projectiles

    -- Hierarchy/Children Component
    local hierarchyComponentIndex = self:addComponent(HierarchyComponent(self))
    local hierarchyComponent = self:getComponent(hierarchyComponentIndex)
    ---@cast hierarchyComponent HierarchyComponent

    -- Add Children
    local star = Star(seed)
    hierarchyComponent:addChild(star)
    local planet = Planet(seed)
    hierarchyComponent:addChild(planet)
    local asteroid = Asteroid(seed)
    hierarchyComponent:addChild(asteroid)

    self:registerEventHandlers()
end)

return StarSystem
