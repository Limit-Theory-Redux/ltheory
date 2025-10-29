local SolarSystemGenTest = require("States.Application")

local CelestialEntities = require("Modules.CelestialObjects.Entities")

local CoreComponents = require("Modules.Core.Components")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local EconomyComponents = require("Modules.Economy.Components")
local PhysicsComponents = require("Modules.Physics.Components")
local SpatialComponents = require("Modules.Spatial.Components")

local Registry = require("Core.ECS.Registry")
local Items = require("Shared.Registries.Items")
local UniverseManager = require("Modules.CelestialObjects.Managers").UniverseManager
require("Shared.Definitions.ItemDefs")

local Rulesets = require("Config.Gen.Rulesets")

local seed = 1

-- Helper function to extract entity class from tostring(entity)
local function getEntityClass(entity)
    local str = tostring(entity)                         -- e.g., "UniverseEntity(U123)"
    return str:match("^(%w+)Entity%(.+%)$") or "Unknown" -- Extracts "Universe" from "UniverseEntity(U123)"
end

---@diagnostic disable-next-line: duplicate-set-field
function SolarSystemGenTest:onInit()
    --self.profiler:start()

    local rulesets = {
        { name = "StandardSolarSystem",        ruleset = Rulesets.StandardSolarSystem,        seed = 1 },
        { name = "StandardBinarySolarSystem",  ruleset = Rulesets.StandardBinarySolarSystem,  seed = 1 },
        { name = "StandardTrinarySolarSystem", ruleset = Rulesets.StandardTrinarySolarSystem, seed = 1 }
    }

    for _, test in ipairs(rulesets) do
        Log.Info("Testing %s", test.name)
        local success, universe = pcall(function()
            return UniverseManager:createUniverse(test.ruleset, test.seed)
        end)
        if not success then
            Log.Error("Failed to create universe with %s: %s", test.name, tostring(universe))
        elseif universe then
            Log.Info("Universe Hierarchy for %s:", test.name)
            self:processEntityHierarchy(universe, "", true)
        else
            Log.Error("Universe creation returned nil for %s", test.name)
        end
    end

    --self.profiler:stop()
    self:quit()
end

--- Recursively processes an entity and its children, logging relevant properties with tree representation
---@param entity Entity The entity to process
---@param prefix string The prefix for tree visualization
---@param isLast boolean Whether this entity is the last child at its level
function SolarSystemGenTest:processEntityHierarchy(entity, prefix, isLast)
    local linePrefix = prefix .. (isLast and "└── " or "├── ")
    local propPrefix = prefix .. (isLast and "    " or "│   ")

    local typeComp = entity:get(CoreComponents.Type)
    local entityClass = getEntityClass(entity) -- e.g., "Universe", "Planet"
    local subtype = typeComp and typeComp:getSubtype() or "No Subtype"

    Log.Info("%sEntity: %s (%s)", linePrefix, entityClass, subtype)

    local transform = entity:get(PhysicsComponents.Transform)
    local position = transform and transform:getPosition() or { x = 0, y = 0, z = 0 }
    Log.Info("%sPosition: (%s, %s, %s)", propPrefix, position.x, position.y, position.z)

    if entityClass == "Universe" then
        -- Universe has no additional properties
    elseif entityClass == "StarSystem" then
        local age = entity:get(CelestialComponents.Age)
        local metallicity = entity:get(CelestialComponents.Metallicity)
        local stability = entity:get(CelestialComponents.Stability)
        local stabilityEnum = stability and Enums.Gen.StarSystem.Stability[stability:getStability()] or "N/A"
        Log.Info("%sType: %s", propPrefix, subtype) -- Explicitly log StarSystemType
        Log.Info("%sAge: %s years", propPrefix, age and age:getAge() or "N/A")
        Log.Info("%sMetallicity: %s", propPrefix, metallicity and metallicity:getMetallicity() or "N/A")
        Log.Info("%sStability: %s (Value: %d)", propPrefix, stabilityEnum, stability and stability:getStability() or 0)
    elseif entityClass == "Star" then
        local mass = entity:get(PhysicsComponents.Mass)
        local luminosity = entity:get(CelestialComponents.Luminosity)
        Log.Info("%sMass: %s solar masses", propPrefix, mass and mass:getMass() or "N/A")
        Log.Info("%sLuminosity: %s solar luminosities", propPrefix, luminosity and luminosity:getLuminosity() or "N/A")
    elseif entityClass == "Planet" then
        local orbit = entity:get(SpatialComponents.Orbit)
        local atmosphere = entity:get(CelestialComponents.Atmosphere)
        local temperature = entity:get(CelestialComponents.Temperature)
        local gravity = entity:get(CelestialComponents.Gravity)
        local rotation = entity:get(CelestialComponents.RotationPeriod)
        local eccentricity = entity:get(CelestialComponents.Eccentricity)
        local magneticField = entity:get(CelestialComponents.MagneticField)
        local inclination = entity:get(CelestialComponents.Inclination)
        Log.Info("%sSize: %s Earth radii", propPrefix, transform and transform:getScale() or "N/A")
        Log.Info("%sOrbit Radius: %s AU", propPrefix, orbit and orbit:getOrbitRadius() or "N/A")
        Log.Info("%sAtmosphere: %s", propPrefix, atmosphere and "Present" or "None")
        Log.Info("%sTemperature: %s K", propPrefix, temperature and temperature:getTemperature() or "N/A")
        Log.Info("%sGravity: %s g", propPrefix, gravity and gravity:getGravity() or "N/A")
        Log.Info("%sRotation Period: %s hours", propPrefix, rotation and rotation:getRotationPeriod() or "N/A")
        Log.Info("%sEccentricity: %s", propPrefix, eccentricity and eccentricity:getEccentricity() or "N/A")
        Log.Info("%sMagnetic Field: %s", propPrefix, magneticField and "Present" or "None")
        Log.Info("%sInclination: %s degrees", propPrefix, inclination and inclination:getInclination() or "N/A")
    elseif entityClass == "Moon" then
        local orbit = entity:get(SpatialComponents.Orbit)
        local inclination = entity:get(CelestialComponents.Inclination)
        Log.Info("%sSize: %s Earth radii", propPrefix, transform and transform:getScale() or "N/A")
        Log.Info("%sOrbital Distance: %s km", propPrefix, orbit and orbit:getOrbitRadius() or "N/A")
        Log.Info("%sInclination: %s degrees", propPrefix, inclination and inclination:getInclination() or "N/A")
    elseif entityClass == "AsteroidRing" then
        local composition = entity:get(CelestialComponents.Composition)
        local thickness = entity:get(CelestialComponents.Thickness)
        Log.Info("%sComposition: %s", propPrefix, composition and composition:getComposition() or "N/A")
        Log.Info("%sThickness: %s meters", propPrefix, thickness and thickness:getThickness() or "N/A")
    elseif entityClass == "AsteroidBelt" then
        local density = entity:get(CelestialComponents.Density)
        local composition = entity:get(CelestialComponents.Composition)
        local width = entity:get(SpatialComponents.Width)
        local orbit = entity:get(SpatialComponents.Orbit)
        Log.Info("%sDensity: %s", propPrefix, density and density:getDensity() or "N/A")
        Log.Info("%sComposition: %s", propPrefix, composition and composition:getComposition() or "N/A")
        Log.Info("%sWidth: %s km", propPrefix, width and width:getWidth() or "N/A")
        Log.Info("%sOrbit Radius: %s AU", propPrefix, orbit and orbit:getOrbitRadius() or "N/A")
    elseif entityClass == "Asteroid" then
        local itemCmp = entity:get(EconomyComponents.Item)
        local quantityCmp = entity:get(EconomyComponents.Quantity)
        if itemCmp and quantityCmp then
            local itemDef = Items:getDefinition(itemCmp:getItem())
            Log.Info("%sItem: %s, Quantity: %s, Price: %d", propPrefix,
                itemDef and itemDef.name or "Unknown",
                quantityCmp:getQuantity(),
                itemDef and itemDef.startEquilibriumPrice or 0)
        end
    end

    local children = entity:get(CoreComponents.Children)
    if children then
        local childList = children:iterChildren()
        local childCount = 0
        for _ in childList do
            childCount = childCount + 1
        end
        local i = 0
        for child in children:iterChildren() do
            i = i + 1
            ---@cast child Entity
            local newPrefix = prefix .. (isLast and "    " or "│   ")
            self:processEntityHierarchy(child, newPrefix, i == childCount)
        end
    end
end

return SolarSystemGenTest
