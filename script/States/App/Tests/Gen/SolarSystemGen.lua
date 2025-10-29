local SolarSystemGenTest = require("States.Application")

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

---@diagnostic disable-next-line: duplicate-set-field
function SolarSystemGenTest:onInit()
    local universe = UniverseManager:createUniverse(Rulesets.StandardSolarSystem, seed)

    -- Test for universe creation on 24.Oct.2025 @IllustrisJack
    if not universe then
        Log.Error("Failed to create universe for Single Star Solar System")
        self:quit()
        return
    end

    -- Print the entity hierarchy with tree representation
    Log.Info("Universe Hierarchy:")
    self:processEntityHierarchy(universe, "", true)

    self:quit()
end

--- Recursively processes an entity and its children, logging relevant properties with tree representation
---@param entity Entity The entity to process
---@param prefix string The prefix for tree visualization
---@param isLast boolean Whether this entity is the last child at its level
function SolarSystemGenTest:processEntityHierarchy(entity, prefix, isLast)
    -- Determine the line prefix for the current entity
    local linePrefix = prefix .. (isLast and "└── " or "├── ")

    -- Log entity information
    Log.Info("%sEntity: %s", linePrefix, tostring(entity))

    -- Get common components
    local typeComp = entity:get(CoreComponents.Type)
    local transform = entity:get(PhysicsComponents.Transform)
    local type = typeComp and typeComp:getType() or "Unknown"
    local position = transform and transform:getPosition() or Position(0, 0, 0)

    -- Indent subsequent properties for readability
    local propPrefix = prefix .. (isLast and "    " or "│   ")
    Log.Info("%sType: %s, Position: (%s, %s, %s)", propPrefix, type, position.x, position.y, position.z)

    -- Log specific properties based on entity type
    if type == "StarSystem" then
        local age = entity:get(CelestialComponents.Age)
        local metallicity = entity:get(CelestialComponents.Metallicity)
        local stability = entity:get(CelestialComponents.Stability)
        Log.Info("%sAge: %s years", propPrefix, age and age:getAge() or "N/A")
        Log.Info("%sMetallicity: %s", propPrefix, metallicity and metallicity:getMetallicity() or "N/A")
        Log.Info("%sStability: %s", propPrefix, stability and stability:getStability() or "N/A")
    elseif type == "Star" then
        local mass = entity:get(PhysicsComponents.Mass)
        local luminosity = entity:get(CelestialComponents.Luminosity)
        Log.Info("%sMass: %s solar masses", propPrefix, mass and mass:getMass() or "N/A")
        Log.Info("%sLuminosity: %s solar luminosities", propPrefix, luminosity and luminosity:getLuminosity() or "N/A")
    elseif type == "Planet" then
        local transform = entity:get(PhysicsComponents.Transform)
        local orbit = entity:get(SpatialComponents.Orbit)
        local atmosphere = entity:get(CelestialComponents.Atmosphere)
        local temperature = entity:get(CelestialComponents.Temperature)
        local gravity = entity:get(CelestialComponents.Gravity)
        local rotation = entity:get(CelestialComponents.Rotation)
        local eccentricity = entity:get(CelestialComponents.Eccentricity)
        local magneticField = entity:get(CelestialComponents.MagneticField)
        local inclination = entity:get(SpatialComponents.Inclination)
        Log.Info("%sSize: %s Earth radii", propPrefix, transform and transform:getScale() or "N/A")
        Log.Info("%sOrbit Radius: %s AU", propPrefix, orbit and orbit:getOrbitRadius() or "N/A")
        Log.Info("%sAtmosphere: %s", propPrefix, atmosphere and "Present" or "None")
        Log.Info("%sTemperature: %s K", propPrefix, temperature and temperature:getTemperature() or "N/A")
        Log.Info("%sGravity: %s g", propPrefix, gravity and gravity:getGravity() or "N/A")
        Log.Info("%sRotation Period: %s hours", propPrefix, rotation and rotation:getRotationPeriod() or "N/A")
        Log.Info("%sEccentricity: %s", propPrefix, eccentricity and eccentricity:getEccentricity() or "N/A")
        Log.Info("%sMagnetic Field: %s", propPrefix, magneticField and "Present" or "None")
        Log.Info("%sInclination: %s degrees", propPrefix, inclination and inclination:getInclination() or "N/A")
    elseif type == "Moon" then
        local transform = entity:get(PhysicsComponents.Transform)
        local orbit = entity:get(SpatialComponents.Orbit)
        local inclination = entity:get(SpatialComponents.Inclination)
        Log.Info("%sSize: %s Earth radii", propPrefix, transform and transform:getScale() or "N/A")
        Log.Info("%sOrbital Distance: %s km", propPrefix, orbit and orbit:getOrbitRadius() or "N/A")
        Log.Info("%sInclination: %s degrees", propPrefix, inclination and inclination:getInclination() or "N/A")
    elseif type == "AsteroidRing" then
        local composition = entity:get(CelestialComponents.Composition)
        local thickness = entity:get(CelestialComponents.Thickness)
        Log.Info("%sComposition: %s", propPrefix, composition and composition:getComposition() or "N/A")
        Log.Info("%sThickness: %s meters", propPrefix, thickness and thickness:getThickness() or "N/A")
    elseif type == "AsteroidBelt" then
        local density = entity:get(CelestialComponents.Density)
        local composition = entity:get(CelestialComponents.Composition)
        local width = entity:get(SpatialComponents.Width)
        local orbit = entity:get(SpatialComponents.Orbit)
        Log.Info("%sDensity: %s", propPrefix, density and density:getDensity() or "N/A")
        Log.Info("%sComposition: %s", propPrefix, composition and composition:getComposition() or "N/A")
        Log.Info("%sWidth: %s km", propPrefix, width and width:getWidth() or "N/A")
        Log.Info("%sOrbit Radius: %s AU", propPrefix, orbit and orbit:getOrbitRadius() or "N/A")
    elseif type == "Asteroid" then
        local itemCmp = entity:get(EconomyComponents.Item)
        local quantityCmp = entity:get(EconomyComponents.Quantity)
        if itemCmp and quantityCmp then
            local itemDef = Items:getDefinition(itemCmp:getItem())
            Log.Info("%sItem: %s, Quantity: %s", propPrefix, itemDef and itemDef.name or "Unknown", quantityCmp:getQuantity())
        end
    end

    -- Log item and quantity components for any entity that has them
    local itemCmp = entity:get(EconomyComponents.Item)
    local quantityCmp = entity:get(EconomyComponents.Quantity)
    if itemCmp and quantityCmp then
        local itemDef = Items:getDefinition(itemCmp:getItem())
        Log.Info("%sItem: %s, Quantity: %s", propPrefix, itemDef and itemDef.name or "Unknown", quantityCmp:getQuantity())
    end

    -- Recursively process children
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
