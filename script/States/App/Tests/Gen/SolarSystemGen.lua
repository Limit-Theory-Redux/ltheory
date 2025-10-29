local Application         = require('States.Application')

---@class SolarSystemGenTest: Application
local SolarSystemGenTest  = Subclass("SolarSystemGenTest", Application)

local CelestialEntities   = require("Modules.CelestialObjects.Entities")

local CoreComponents      = require("Modules.Core.Components")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local EconomyComponents   = require("Modules.Economy.Components")
local PhysicsComponents   = require("Modules.Physics.Components")
local SpatialComponents   = require("Modules.Spatial.Components")

local Registry            = require("Core.ECS.Registry")
require("Shared.Definitions.ItemDefs")
local Items               = require("Shared.Registries.Items")
local UniverseManager     = require("Modules.CelestialObjects.Managers").UniverseManager

local Rulesets            = require("Config.Gen.Rulesets")

local UniverseScaleConfig = require("Config.Gen.UniverseScaleConfig")

local seed                = 1

-- Helper function to extract entity class from tostring(entity)
local function getEntityClass(entity)
    local str = tostring(entity)                         -- e.g., "UniverseEntity(U123)"
    return str:match("^(%w+)Entity%(.+%)$") or "Unknown" -- Extracts "Universe" from "UniverseEntity(U123)"
end

---@diagnostic disable-next-line: duplicate-set-field
function SolarSystemGenTest:onInit()
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

    self:quit()
end

--- Recursively processes an entity and its children, logging relevant properties
---@param entity Entity The entity to process
---@param prefix string The prefix for tree visualization
---@param isLast boolean Whether this entity is the last child at its level
function SolarSystemGenTest:processEntityHierarchy(entity, prefix, isLast)
    local linePrefix = prefix .. (isLast and "└── " or "├── ")
    local propPrefix = prefix .. (isLast and "    " or "│   ")

    local typeCmp = entity:get(CoreComponents.Type)
    local entityClass = getEntityClass(entity)
    local subtype = typeCmp and typeCmp:getSubtype() or "No Subtype"

    Log.Info("%sEntity: %s (%s)", linePrefix, entityClass, subtype)

    local transformCmp = entity:get(PhysicsComponents.Transform)
    local position = transformCmp and transformCmp:getPosition() or { x = 0, y = 0, z = 0 }
    local scale = transformCmp and transformCmp:getScale() or 1

    Log.Info("%sPosition (scaled):   (%.2f, %.2f, %.2f)", propPrefix, position.x, position.y, position.z)
    Log.Info("%sScale (scaled): %s", propPrefix, scale)

    local orbitCmp = entity:get(SpatialComponents.Orbit)
    local inclinationCmp = entity:get(SpatialComponents.Inclination)

    if entityClass == "Planet" then
        local atmosphereCmp = entity:get(CelestialComponents.Atmosphere)
        local temperatureCmp = entity:get(CelestialComponents.Temperature)
        local gravityCmp = entity:get(CelestialComponents.Gravity)
        local rotationCmp = entity:get(CelestialComponents.RotationPeriod)
        local eccentricityCmp = entity:get(CelestialComponents.Eccentricity)
        local magneticFieldCmp = entity:get(CelestialComponents.MagneticField)

        -- Orbit radius
        if orbitCmp then
            local orbitRadius = orbitCmp:getOrbitRadius()
            local realMeters = UniverseScaleConfig:toRealMeters(orbitRadius)
            Log.Info("%sOrbit Radius (scaled): %.2f m", propPrefix, orbitRadius)
            Log.Info("%sOrbit Radius (real, km): %.2f", propPrefix, realMeters / 1000)
            Log.Info("%sOrbit Radius (real, AU): %.2f", propPrefix, UniverseScaleConfig:metersToAU(realMeters))
        end

        -- Planet radius
        local radiusUnit, realRadius
        if subtype == "GasGiant" then
            realRadius = scale / UniverseScaleConfig:jupiterRadiiToGameUnits(1, "planet")
            radiusUnit = "Jupiter radii"
        else
            realRadius = scale / UniverseScaleConfig:earthRadiiToGameUnits(1, "planet")
            radiusUnit = "Earth radii"
        end
        Log.Info("%sPlanet Radius: %.2f %s", propPrefix, realRadius, radiusUnit)

        -- Other properties
        Log.Info("%sAtmosphere: %s", propPrefix, atmosphereCmp and "Present" or "None")
        Log.Info("%sTemperature: %s K", propPrefix, temperatureCmp and temperatureCmp:getTemperature() or "N/A")
        Log.Info("%sGravity: %s g", propPrefix, gravityCmp and gravityCmp:getGravity() or "N/A")
        Log.Info("%sRotation Period: %s hours", propPrefix, rotationCmp and rotationCmp:getRotationPeriod() or "N/A")
        Log.Info("%sEccentricity: %s", propPrefix, eccentricityCmp and eccentricityCmp:getEccentricity() or "N/A")
        Log.Info("%sMagnetic Field: %s", propPrefix, magneticFieldCmp and "Present" or "None")
        Log.Info("%sInclination: %s degrees", propPrefix, inclinationCmp and inclinationCmp:getInclination() or "N/A")
    elseif entityClass == "Moon" then
        local atmosphereCmp = entity:get(CelestialComponents.Atmosphere)
        local gravityCmp = entity:get(CelestialComponents.Gravity)
        local rotationCmp = entity:get(CelestialComponents.RotationPeriod)
        local transformCmp = entity:get(PhysicsComponents.Transform)

        -- Orbit radius
        if orbitCmp then
            local orbitRadius = orbitCmp:getOrbitRadius()

            Log.Info("%sOrbit Radius (scaled): %.2f m", propPrefix, orbitRadius)
            Log.Info("%sOrbit Radius (real, meters): %.2f", propPrefix, UniverseScaleConfig:toRealMeters(orbitRadius))
            Log.Info("%sOrbit Radius (real, km): %.2f", propPrefix, UniverseScaleConfig:toRealMeters(orbitRadius) / 1000)
        end

        -- Moon radius
        if transformCmp then
            local moonRadius = transformCmp:getScale()
            local realMeters = UniverseScaleConfig:toRealMeters(moonRadius)
            local realRadius = UniverseScaleConfig:metersToEarthRadii(realMeters)
            Log.Info("%sMoon Radius: %.8f (scaled)", propPrefix, moonRadius)
            Log.Info("%sMoon Radius: %.8f (real, Earth radii)", propPrefix, realRadius)
            Log.Info("%sMoon Radius: %.8f (real, km)", propPrefix, realMeters / 1000)
        end

        -- Other properties
        Log.Info("%sAtmosphere: %s", propPrefix, atmosphereCmp and "Present" or "None")
        Log.Info("%sGravity: %s g", propPrefix, gravityCmp and gravityCmp:getGravity() or "N/A")
        Log.Info("%sRotation Period: %s hours", propPrefix, rotationCmp and rotationCmp:getRotationPeriod() or "N/A")
        Log.Info("%sInclination: %s degrees", propPrefix, inclinationCmp and inclinationCmp:getInclination() or "N/A")
    elseif entityClass == "AsteroidRing" then
        local compositionCmp = entity:get(CelestialComponents.Composition)
        local thicknessCmp = entity:get(CelestialComponents.Thickness)
        if thicknessCmp then
            local scaledMeters = thicknessCmp:getThickness()
            local realMeters = scaledMeters / (UniverseScaleConfig.globalScale * (UniverseScaleConfig.objectScales.asteroidRing or 1))
            local realKm = realMeters / 1000
            Log.Info("%sThickness (scaled): %.2f", propPrefix, scaledMeters)
            Log.Info("%sThickness (real, meters): %.2f", propPrefix, realMeters)
            Log.Info("%sThickness (real, km): %.2f", propPrefix, realKm)
        end
        Log.Info("%sComposition: %s", propPrefix, compositionCmp and compositionCmp:getComposition() or "N/A")
        Log.Info("%sInclination: %s degrees", propPrefix, inclinationCmp and inclinationCmp:getInclination() or "N/A")
    elseif entityClass == "AsteroidBelt" then
        local densityCmp = entity:get(CelestialComponents.Density)
        local compositionCmp = entity:get(CelestialComponents.Composition)
        local widthCmp = entity:get(SpatialComponents.Width)

        if widthCmp then
            local scaledMeters = widthCmp:getWidth()
            local realMeters = scaledMeters / (UniverseScaleConfig.globalScale * (UniverseScaleConfig.objectScales.asteroidBelt or 1))
            local realKm = realMeters / 1000
            Log.Info("%sWidth (scaled): %.2f", propPrefix, scaledMeters)
            Log.Info("%sWidth (real, meters): %.2f", propPrefix, realMeters)
            Log.Info("%sWidth (real, km): %.2f", propPrefix, realKm)
        end

        if orbitCmp then
            local radiusAU = orbitCmp:getOrbitRadius()
            local scaledMeters = UniverseScaleConfig:auToGameUnits(radiusAU, "asteroidBelt")
            local realMeters = scaledMeters / (UniverseScaleConfig.globalScale * (UniverseScaleConfig.objectScales.asteroidBelt or 1))
            local realKm = realMeters / 1000
            Log.Info("%sOrbit Radius (scaled): %.2f AU", propPrefix, radiusAU)
            Log.Info("%sOrbit Radius (real, meters): %.2f", propPrefix, realMeters)
            Log.Info("%sOrbit Radius (real, km): %.2f", propPrefix, realKm)
        end

        Log.Info("%sDensity: %s", propPrefix, densityCmp and densityCmp:getDensity() or "N/A")
        Log.Info("%sComposition: %s", propPrefix, compositionCmp and compositionCmp:getComposition() or "N/A")
        Log.Info("%sInclination: %s degrees", propPrefix, inclinationCmp and inclinationCmp:getInclination() or "N/A")
    end

    -- Recurse into children
    local childrenCmp = entity:get(CoreComponents.Children)
    if childrenCmp then
        local i = 0
        local childCount = 0
        for _ in childrenCmp:iterChildren() do childCount = childCount + 1 end
        for child in childrenCmp:iterChildren() do
            i = i + 1
            local newPrefix = prefix .. (isLast and "    " or "│   ")
            self:processEntityHierarchy(child, newPrefix, i == childCount)
        end
    end
end

return SolarSystemGenTest
