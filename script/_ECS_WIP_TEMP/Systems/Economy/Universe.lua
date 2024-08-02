-- Entities
local StarSystemEntity = require("Entities.CelestialObjects.StarSystem")
local SpaceshipEntity = require("Entities.Constructs.Spaceship")

-- Systems
local GlobalStorage = require("Systems.GlobalStorage")
local UniverseEconomy = require("Systems.Economy.UniverseEconomy")

---@class Universe
local Universe = Class(function(self, seed)
    ---@cast self Universe
    self:init(seed)
end)

---@param seed integer
function Universe:init(seed)
    GameState.world.currentUniverse = self
    -- PlayerEntity

    self.universeSeed = seed
    self.universeRng = RNG.Create(seed):managed()
    self.starSystems = {}
    self.players = {}
    self.factions = {}
    self.economy = UniverseEconomy:init()
end

function Universe:getStarSystems()
    return self.starSystems
end

function Universe:getPlayers()
    return self.players
end

function Universe:getFactions()
    return self.factions
end

---@param withEconomy boolean
---@return integer EntityInfo
function Universe:createStarSystem(withEconomy)
    -- Spawn a new star system entity
    local systemEntity = StarSystemEntity(self.universeRng:get64())

    -- Store the id in the global game state & universe class
    local systemEntityInfo = systemEntity:getEntityInfo()
    GameState.world.currentSystem = systemEntityInfo
    insert(self.starSystems, systemEntityInfo)

    -- Store the entity in the GlobalStorage
    GlobalStorage:storeEntity(systemEntity)

    -- Add System Generics
    UniverseEconomy:addSystemGenerics(systemEntity)

    return systemEntityInfo
end

---@param systemId integer
---@param pos Vec3f
---@param constructor SpaceshipConstructor
function Universe:createShip(systemId, pos, constructor)
    -- Create spaceship entity
    local spaceship = SpaceshipEntity(constructor.name, constructor.ownerId, constructor.hullType,
        constructor.seed or self.universeRng:get64())

    -- Get spaceship transform component and set position
    local spaceshipTransformComponent = spaceship:findComponentByName("Transform")
    ---@cast spaceshipTransformComponent TransformComponent
    spaceshipTransformComponent:setPosition(pos)

    -- Add spaceship entity to global storage
    GlobalStorage:storeEntity(spaceship)

    -- Get star system entity from storage
    local systemEntity = GlobalStorage:getEntity({ archetype = Enums.EntityArchetype.StarSystemEntity, id = systemId })
    ---@cast systemEntity StarSystem

    -- Get star system hierarchy component & add spaceship as a child
    local systemHierarchyComponent = systemEntity:findComponentByName("Hierarchy")
    ---@cast systemHierarchyComponent EntityHierarchyComponent
    systemHierarchyComponent:addChild(spaceship:getEntityInfo())
end

return Universe
