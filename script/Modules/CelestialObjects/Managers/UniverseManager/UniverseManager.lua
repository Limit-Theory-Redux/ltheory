local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local CelestialEntities = require("Modules.CelestialObjects.Entities")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local SpatialEntities = require("Modules.Spatial.Entities")
local SpatialComponents = require("Modules.Spatial.Components")
local CoreComponents = require("Modules.Core.Components")
local PhysicsComponents = require("Modules.Physics.Components")
local RuleEvaluator = require("Modules.CelestialObjects.Managers.UniverseManager.RuleEvaluator")
local GenerationContext = require("Modules.CelestialObjects.Managers.UniverseManager.GenerationContext")
local RulesetValidator = require("Modules.CelestialObjects.Managers.UniverseManager.RulesetValidator")
local PositionCalculator = require("Modules.CelestialObjects.Managers.UniverseManager.PositionCalculator")
local UniverseScaleConfig = require("Config.Gen.UniverseScaleConfig")
local Items = require("Shared.Registries.Items")

---@class UniverseManager
---@field profiler QuickProfiler
---@field rulesets table<string, Ruleset>
---@field scaleConfig UniverseScaleConfig
local UniverseManager = Class("UniverseManager", function(self)
    self:registerVars()
end)

function UniverseManager:registerVars()
    self.profiler = QuickProfiler("UniverseManager", false, false)
    self.rulesets = {}
    self.scaleConfig = UniverseScaleConfig
end

--- Register a ruleset for universe generation
---@param name string
---@param ruleset Ruleset
function UniverseManager:registerRuleset(name, ruleset)
    RulesetValidator.validate(ruleset)
    self.rulesets[name] = ruleset
    ruleset.name = name
end

--- Create a universe with the specified ruleset and seed
---@param ruleset string|Ruleset
---@param seed number
---@return Entity|nil
function UniverseManager:createUniverse(ruleset, seed)
    --self.profiler:start()
    local cfg = self:_resolveRuleset(ruleset)
    local universeSeed = seed or cfg.seed
    if not universeSeed then
        Log.Error("No seed provided for universe generation")
        return nil
    end

    local rng = RNG.Create(universeSeed)
    if not rng then
        Log.Error("Failed to create master RNG")
        return nil
    end

    Log.Debug("Creating universe with seed '%s' and ruleset '%s'", tostring(universeSeed), cfg.name)

    local universe = CelestialEntities.Universe(rng:get64())
    if not universe then
        Log.Error("Failed to create Universe entity")
        return nil
    end

    universe:add(CoreComponents.Type("None"))
    self:_applyOverrides(rng, universe, cfg.overrides and cfg.overrides.universe)

    local context = GenerationContext()
    local sysCount = RuleEvaluator.evaluate(rng, cfg.starSystems.count, context)
    Log.Debug("Generating %d star systems", sysCount)

    for i = 1, sysCount do
        local starSystem = self:_generateStarSystem(rng, cfg, context, i)
        if starSystem then
            Registry:attachEntity(universe, starSystem)
            Log.Debug("Attached StarSystem %d to Universe", i)
        end
    end

    --self.profiler:stop()
    return universe
end

--- Resolve a ruleset by name or direct reference
---@param ruleset string|Ruleset
---@return Ruleset
function UniverseManager:_resolveRuleset(ruleset)
    if type(ruleset) == "string" then
        local s = self.rulesets[ruleset]
        if not s then error("Ruleset not registered: " .. ruleset) end
        return s
    end
    return ruleset
end

--- Apply override configurations to an entity
---@param rng RNG
---@param parent Entity
---@param overrides table|nil
function UniverseManager:_applyOverrides(rng, parent, overrides)
    if not overrides or not parent then return end
    for _, def in ipairs(overrides) do
        local entity = CelestialEntities[def.type](rng:get64())
        if not entity then
            Log.Error("Failed to create entity of type %s", def.type)
            return
        end
        if def.position then
            entity:get(PhysicsComponents.Transform):setPosition(def.position)
        end
        for compName, value in pairs(def.components or {}) do
            local component = CoreComponents[compName] or SpatialComponents[compName] or CelestialComponents[compName]
            if component then
                entity:add(component(value))
            else
                Log.Warn("Unknown component %s for entity %s", compName, def.type)
            end
        end
        Registry:attachEntity(parent, entity)
    end
end

--- Generate a star system
---@param rng RNG
---@param cfg Ruleset
---@param context GenerationContext
---@param systemIndex number
---@return Entity|nil
function UniverseManager:_generateStarSystem(rng, cfg, context, systemIndex)
    local ssSeed = rng:get64()
    local starSystem = CelestialEntities.StarSystem(ssSeed)
    if not starSystem then
        Log.Error("Failed to create StarSystem with seed: %s", tostring(ssSeed))
        return nil
    end

    local ssRNG = RNG.Create(ssSeed)
    if not ssRNG then
        Log.Error("Failed to create RNG for StarSystem with seed: %s", tostring(ssSeed))
        return nil
    end

    -- Set star system aspects
    local starCount = RuleEvaluator.evaluate(ssRNG, cfg.stars.count, context)
    context:set("starCount", starCount)

    local aspects = {
        { key = "systemType",        rule = cfg.starSystems.aspects.type },
        { key = "systemAge",         rule = cfg.starSystems.aspects.age },
        { key = "systemMetallicity", rule = cfg.starSystems.aspects.metallicity },
        { key = "stability",         rule = cfg.starSystems.aspects.stability }
    }
    for _, aspect in ipairs(aspects) do
        local value = RuleEvaluator.evaluate(ssRNG, aspect.rule, context)
        if value then
            context:set(aspect.key, value)
        else
            Log.Warn("Failed to evaluate aspect %s", aspect.key)
        end
    end

    starSystem:add(CoreComponents.Type(context:get("systemType") or "Single"))
    starSystem:add(CelestialComponents.Age(context:get("systemAge") or 4.6e9))
    starSystem:add(CelestialComponents.Metallicity(context:get("systemMetallicity") or 0.02))
    starSystem:add(CelestialComponents.Stability(context:get("stability") or Enums.Gen.StarSystem.Stability.Stable))

    -- Set position
    local systemPosition = PositionCalculator.universePosition(
        ssRNG,
        self.scaleConfig.universe.distributionType,
        self.scaleConfig.universe.radius,
        self.scaleConfig.universe.gridSpacing,
        self.scaleConfig.universe.gridJitter,
        systemIndex
    )
    starSystem:get(PhysicsComponents.Transform):setPosition(systemPosition)
    context:set("parentPosition", systemPosition)

    -- Generate stars
    for i = 1, starCount do
        context:set("starIndex", i)
        local star = self:_generateStar(ssRNG, cfg, context)
        if star then
            Registry:attachEntity(starSystem, star)
        end
    end

    -- Generate planets
    local planetCount = RuleEvaluator.evaluate(ssRNG, cfg.planets.count, context)
    context:set("planetCount", planetCount)
    local orbits = self:_calculateOrbits(ssRNG, cfg.planets.aspects.orbitRadius, planetCount)
    for i = 1, planetCount do
        context:set("orbitRadius", orbits[i])
        local planet = self:_generatePlanet(ssRNG, cfg, context)
        if planet then
            Registry:attachEntity(starSystem, planet)
        end
    end

    -- Generate asteroid belt
    local hasAsteroidBelt = RuleEvaluator.evaluate(ssRNG, cfg.asteroidBelts.count, context)
    if hasAsteroidBelt then
        local belt = self:_generateAsteroidBelt(ssRNG, cfg, context)
        if belt then
            Registry:attachEntity(starSystem, belt)
        end
    end

    self:_applyOverrides(ssRNG, starSystem, cfg.overrides and cfg.overrides.starSystem)
    return starSystem
end

--- Calculate orbital radii for planets
---@param rng RNG
---@param rule table
---@param count number
---@return table
function UniverseManager:_calculateOrbits(rng, rule, count)
    local orbits = {}
    local min = rule.min or 0.7
    local max = rule.max or 4.0
    for i = 1, count do
        local t = (i - 1) / math.max(1, count - 1)
        local radius = math.exp(math.log(min) + (math.log(max) - math.log(min)) * t)
        table.insert(orbits, radius)
    end
    table.sort(orbits)
    return orbits
end

--- Compute gravity and mass for a celestial body
---@param size number
---@param bodyType string
---@return number, number
local function computeGravity(size, bodyType)
    local G = 6.67430e-11
    local earthRadius = 6.371e6
    local radius = size * earthRadius
    local density = bodyType == Enums.Gen.PlanetTypes.Rocky and 5500
        or bodyType == Enums.Gen.PlanetTypes.Desert and 5500
        or bodyType == Enums.Gen.PlanetTypes.Icy and 2000
        or bodyType == Enums.Gen.PlanetTypes.GasGiant and 1300
        or 5500
    local mass = density * (4 / 3) * math.pi * radius ^ 3
    local gravity = (G * mass) / (radius ^ 2) / 9.81
    return math.max(0.05, math.min(5.0, gravity)), mass
end

--- Compute rotation period
---@param bodyType string
---@param size number
---@param orbitRadius number
---@return number
local function computeRotationPeriod(bodyType, size, orbitRadius)
    local basePeriod = bodyType == Enums.Gen.PlanetTypes.GasGiant and 10
        or bodyType == Enums.Gen.PlanetTypes.Icy and 50
        or 24
    return math.max(8, math.min(100, basePeriod / math.max(1.0, size) * (1 + orbitRadius * 0.1)))
end

--- Resolve orbital parameters
---@param rng RNG
---@param cfg Ruleset
---@param context GenerationContext
---@param bodyKind string
local function resolveOrbitalParams(rng, cfg, context, bodyKind)
    local aspects = bodyKind == "moon" and cfg.moons.aspects or cfg.planets.aspects
    local params = {
        longitudeOfAscendingNode = 0,
        argumentOfPeriapsis = 0,
        meanAnomaly = 0
    }
    for key, default in pairs(params) do
        context:set(key, context:get(key) or RuleEvaluator.evaluate(rng, aspects[key], context) or default)
    end
end

--- Compute temperature
---@param starLuminosity number
---@param orbitRadius number
---@param planetType string
---@return number
local function computeTemperature(starLuminosity, orbitRadius, planetType)
    local sigma = 5.6704e-8
    local AU = 1.496e11
    local distance = orbitRadius * AU
    local albedo = planetType == Enums.Gen.PlanetTypes.Rocky and 0.3
        or planetType == Enums.Gen.PlanetTypes.Desert and 0.3
        or planetType == Enums.Gen.PlanetTypes.Icy and 0.6
        or planetType == Enums.Gen.PlanetTypes.GasGiant and 0.5
        or 0.3
    local solarConstant = 1361
    local luminosityFactor = starLuminosity * solarConstant
    local temp = ((1 - albedo) * luminosityFactor / (16 * math.pi * sigma * (distance ^ 2))) ^ 0.25
    return math.max(100, math.min(500, temp))
end

--- Determine if a body has a magnetic field
---@param bodyType string
---@param size number
---@return boolean
local function hasMagneticField(bodyType, size)
    return (bodyType == Enums.Gen.PlanetTypes.Rocky and size > 0.5)
        or bodyType == Enums.Gen.PlanetTypes.GasGiant
        or (bodyType == Enums.Gen.PlanetTypes.Icy and size > 0.3)
end

--- Attach a spatial zone to an entity
---@param parent Entity
---@param rng RNG
---@param cfg Ruleset
---@param context GenerationContext
---@param bodyKind string
local function attachZone(self, parent, rng, cfg, context, bodyKind)
    local zone = SpatialEntities.Zone()
    if not zone then return end
    zone:get(SpatialComponents.Shape):setShape(Enums.ZoneShape.Sphere)
    local radiusRule = bodyKind == "planet" and cfg.planetZoneRadius
        or bodyKind == "moon" and cfg.moonZoneRadius
        or cfg.starZoneRadius
    local zoneRadiusReal = RuleEvaluator.evaluate(rng, radiusRule, context) or 1.0
    local zoneRadiusGame = self.scaleConfig:toGameUnits(zoneRadiusReal, "zone")
    zone:get(SpatialComponents.Shape):setRadius(zoneRadiusGame)
    Registry:attachEntity(parent, zone)
end

--- Generate a star
---@param rng RNG
---@param cfg Ruleset
---@param context GenerationContext
---@return Entity|nil
function UniverseManager:_generateStar(rng, cfg, context)
    local starSeed = rng:get64()
    local star = CelestialEntities.Star(starSeed)
    if not star then
        Log.Error("Failed to create Star")
        return nil
    end

    local starRNG = RNG.Create(starSeed)
    if not starRNG then
        Log.Error("Failed to create RNG for Star")
        return nil
    end

    local aspects = {
        { key = "starType",   rule = cfg.stars.aspects.type,       default = Enums.Gen.StarTypes.MainSequence },
        { key = "starMass",   rule = cfg.stars.aspects.mass,       default = 1.0 },
        { key = "luminosity", rule = cfg.stars.aspects.luminosity, default = 1.0 },
        { key = "position",   rule = cfg.stars.aspects.position,   default = Position(0, 0, 0) }
    }

    for _, aspect in ipairs(aspects) do
        local val = RuleEvaluator.evaluate(starRNG, aspect.rule, context) or aspect.default
        context:set(aspect.key, val)
    end

    star:add(CoreComponents.Type(context:get("starType")))
    star:add(CelestialComponents.Luminosity(context:get("luminosity")))
    star:get(PhysicsComponents.Mass):setMass(context:get("starMass"))

    local parentPos = context:get("parentPosition") or Position(0, 0, 0)
    local absPos = Position(
        parentPos.x + context:get("position").x,
        parentPos.y + context:get("position").y,
        parentPos.z + context:get("position").z
    )
    star:get(PhysicsComponents.Transform):setPosition(absPos)
    star:get(PhysicsComponents.Transform):setScale(self.scaleConfig:getStarRadius(context:get("starMass")))

    attachZone(self, star, starRNG, cfg, context, "star")
    return star
end

--- Generate a planet
---@param rng RNG
---@param cfg Ruleset
---@param context GenerationContext
---@return Entity|nil
function UniverseManager:_generatePlanet(rng, cfg, context)
    local pSeed = rng:get64()
    local planet = CelestialEntities.Planet(pSeed)
    if not planet then
        Log.Error("Failed to create Planet")
        return nil
    end

    local pRNG = RNG.Create(pSeed)
    if not pRNG then
        Log.Error("Failed to create RNG for Planet")
        return nil
    end

    local orbitRadius = context:get("orbitRadius") or 1.0
    local planetType = RuleEvaluator.evaluate(pRNG, cfg.planets.aspects.type, context) or Enums.Gen.PlanetTypes.Rocky
    context:set("planetType", planetType)

    local size
    if planetType == Enums.Gen.PlanetTypes.Rocky or planetType == Enums.Gen.PlanetTypes.Desert then
        size = math.min(2.0, math.max(0.5, (0.5 + orbitRadius * 0.3) * pRNG:getUniformRange(0.8, 1.2)))
    elseif planetType == Enums.Gen.PlanetTypes.Icy then
        size = math.min(5.0, math.max(1.0, (1.0 + orbitRadius * 0.5) * pRNG:getUniformRange(0.8, 1.3)))
    elseif planetType == Enums.Gen.PlanetTypes.GasGiant then
        size = math.min(10.0, math.max(5.0, (6.0 + orbitRadius * 0.3) * pRNG:getUniformRange(0.85, 1.15)))
    else
        size = 1.0
    end
    context:set("planetSize", size)

    local gravity, _ = computeGravity(size, planetType)
    context:set("gravity", gravity)
    local period = computeRotationPeriod(planetType, size, orbitRadius)
    context:set("rotationPeriod", period)

    local stability = context:get("stability") or Enums.Gen.StarSystem.Stability.Stable
    local ecc = stability == Enums.Gen.StarSystem.Stability.Stable and 0.01 + orbitRadius * 0.005
        or stability == Enums.Gen.StarSystem.Stability.Unstable and 0.05 + orbitRadius * 0.01
        or 0.1 + orbitRadius * 0.02
    context:set("eccentricity", math.min(0.2, math.max(0.0, ecc)))

    local inc = stability == Enums.Gen.StarSystem.Stability.Stable and 0.0
        or stability == Enums.Gen.StarSystem.Stability.Unstable and 5.0
        or 10.0
    context:set("inclination", math.min(10.0, math.max(0.0, inc)))

    resolveOrbitalParams(pRNG, cfg, context, "planet")

    local starLuminosity = context:get("luminosity") or 1.0
    local temp = computeTemperature(starLuminosity, orbitRadius, planetType)
    context:set("temperature", temp)

    local orbitalRadiusGame = self.scaleConfig:auToGameUnits(orbitRadius, "starSystem")
    local planetRelPos = PositionCalculator.orbitalPosition(
        orbitalRadiusGame,
        context:get("eccentricity"),
        context:get("inclination"),
        context:get("longitudeOfAscendingNode"),
        context:get("argumentOfPeriapsis"),
        context:get("meanAnomaly")
    )
    local parentPos = context:get("parentPosition") or Position(0, 0, 0)
    local planetAbsPos = Position(parentPos.x + planetRelPos.x, parentPos.y + planetRelPos.y, parentPos.z + planetRelPos.z)

    planet:add(CoreComponents.Type(planetType))
    planet:add(SpatialComponents.Orbit(orbitalRadiusGame))
    planet:add(CelestialComponents.Temperature(temp))
    planet:get(PhysicsComponents.Transform):setScale(self.scaleConfig:earthRadiiToGameUnits(size, "planet"))
    planet:get(PhysicsComponents.Transform):setPosition(planetAbsPos)
    planet:add(CelestialComponents.Gravity(gravity))
    planet:add(CelestialComponents.RotationPeriod(period))
    planet:add(CelestialComponents.Eccentricity(context:get("eccentricity")))
    planet:add(SpatialComponents.Inclination(context:get("inclination")))
    if hasMagneticField(planetType, size) then
        planet:add(CelestialComponents.MagneticField())
    end
    if RuleEvaluator.evaluate(pRNG, cfg.planets.aspects.atmosphere, context) then
        planet:add(CelestialComponents.Atmosphere())
    end

    attachZone(self, planet, pRNG, cfg, context, "planet")

    local asteroidRingType = RuleEvaluator.evaluate(pRNG, cfg.asteroidRings.aspects.type, context)
    if asteroidRingType and asteroidRingType ~= Enums.Gen.AsteroidRingTypes.None then
        local ring = self:_generateAsteroidRing(pRNG, cfg, context)
        if ring then
            Registry:attachEntity(planet, ring)
        end
    end

    local moonCount = RuleEvaluator.evaluate(pRNG, cfg.moons.count, context) or 0
    for i = 1, moonCount do
        local moonContext = GenerationContext()
        moonContext:set("parentPosition", planetAbsPos)
        moonContext:set("eccentricity", context:get("eccentricity"))
        moonContext:set("inclination", context:get("inclination"))
        moonContext:set("longitudeOfAscendingNode", context:get("longitudeOfAscendingNode"))
        moonContext:set("argumentOfPeriapsis", context:get("argumentOfPeriapsis"))
        moonContext:set("meanAnomaly", context:get("meanAnomaly"))
        moonContext:set("planetType", planetType)
        moonContext:set("planetSize", size)
        moonContext:set("gravity", gravity)
        moonContext:set("rotationPeriod", period)
        moonContext:set("temperature", temp)

        local moon = self:_generateMoon(pRNG, cfg, moonContext)
        if moon then
            Registry:attachEntity(planet, moon)
        end
    end

    self:_applyOverrides(pRNG, planet, cfg.overrides and cfg.overrides.planet)
    return planet
end

--- Generate a moon
---@param rng RNG
---@param cfg Ruleset
---@param context GenerationContext
---@return Entity|nil
function UniverseManager:_generateMoon(rng, cfg, context)
    local mSeed = rng:get64()
    local moon = CelestialEntities.Moon(mSeed)
    if not moon then
        Log.Error("Failed to create Moon")
        return nil
    end

    local mRNG = RNG.Create(mSeed)
    if not mRNG then
        Log.Error("Failed to create RNG for Moon")
        return nil
    end

    local planetType = context:get("planetType")
    local planetSize = context:get("planetSize")
    local planetGravity = context:get("gravity")
    local planetRotation = context:get("rotationPeriod")
    local parentPos = context:get("parentPosition")

    local moonOrbitRadius, err = RuleEvaluator.evaluate(mRNG, cfg.moons.aspects.orbitalRadius, context)
    if not moonOrbitRadius then
        Log.Error("Missing or invalid moon orbitalRadius rule: %s", err or "Rule is nil")
        return nil
    end
    context:set("orbitRadius", moonOrbitRadius)

    local size = planetType == Enums.Gen.PlanetTypes.GasGiant and mRNG:getUniformRange(0.1, 0.5)
        or planetType == Enums.Gen.PlanetTypes.Icy and mRNG:getUniformRange(0.05, 0.3)
        or mRNG:getUniformRange(0.05, 0.2)
    context:set("moonSize", size)

    context:set("gravity", planetGravity)
    context:set("rotationPeriod", planetRotation)
    context:set("inclination", context:get("inclination") or mRNG:getUniformRange(0, 5))
    context:set("eccentricity", context:get("eccentricity") or mRNG:getUniformRange(0, 0.05))
    resolveOrbitalParams(mRNG, cfg, context, "moon")

    local orbitalRadiusGame = self.scaleConfig:kmToGameUnits(moonOrbitRadius, "moon")
    local moonRelPos = PositionCalculator.orbitalPosition(
        orbitalRadiusGame,
        context:get("eccentricity"),
        context:get("inclination"),
        context:get("longitudeOfAscendingNode"),
        context:get("argumentOfPeriapsis"),
        context:get("meanAnomaly")
    )
    local moonAbsPos = Position(parentPos.x + moonRelPos.x, parentPos.y + moonRelPos.y, parentPos.z + moonRelPos.z)

    moon:add(CoreComponents.Type("Moon"))
    moon:get(PhysicsComponents.Transform):setScale(self.scaleConfig:earthRadiiToGameUnits(size, "moon"))
    moon:get(PhysicsComponents.Transform):setPosition(moonAbsPos)
    moon:add(SpatialComponents.Orbit(orbitalRadiusGame))
    moon:add(SpatialComponents.Inclination(context:get("inclination")))
    moon:add(CelestialComponents.Gravity(planetGravity))
    moon:add(CelestialComponents.RotationPeriod(planetRotation))
    moon:add(CelestialComponents.Temperature(context:get("temperature") or 150))
    if context:get("magneticField") then
        moon:add(CelestialComponents.MagneticField())
    end

    attachZone(self, moon, mRNG, cfg, context, "moon")
    return moon
end

--- Generate an asteroid belt
---@param rng RNG
---@param cfg Ruleset
---@param context GenerationContext
---@return Entity|nil
function UniverseManager:_generateAsteroidBelt(rng, cfg, context)
    local bSeed = rng:get64()
    local belt = CelestialEntities.AsteroidBelt(bSeed)
    if not belt then
        Log.Error("Failed to create AsteroidBelt")
        return nil
    end

    local bRNG = RNG.Create(bSeed)
    if not bRNG then
        Log.Error("Failed to create RNG for AsteroidBelt")
        return nil
    end

    local beltOrbitRadius = RuleEvaluator.evaluate(bRNG, cfg.asteroidBelts.aspects.orbitRadius, context) or 1.0
    context:set("beltOrbit", beltOrbitRadius)

    local inclination = RuleEvaluator.evaluate(bRNG, cfg.asteroidBelts.aspects.inclination, context) or 0
    context:set("beltInclination", inclination)

    local density = RuleEvaluator.evaluate(bRNG, cfg.asteroidBelts.aspects.density, context) or 0.5
    context:set("beltDensity", density)

    local composition = RuleEvaluator.evaluate(bRNG, cfg.asteroidBelts.aspects.composition, context) or
        { type = Enums.Gen.AsteroidRingTypes.Rocky }
    context:set("beltComposition", composition)

    local widthAU = RuleEvaluator.evaluate(bRNG, cfg.asteroidBelts.aspects.width, context) or 1.0
    context:set("beltWidth", widthAU)

    local beltPos = Position(self.scaleConfig:auToGameUnits(beltOrbitRadius, "starSystem"), 0, 0)
    belt:get(PhysicsComponents.Transform):setPosition(beltPos)
    belt:add(CoreComponents.Type(composition.type))
    belt:add(CelestialComponents.Composition(composition))
    belt:add(CelestialComponents.Density(density))
    belt:add(SpatialComponents.Width(self.scaleConfig:auToGameUnits(widthAU, "belt")))
    belt:add(SpatialComponents.Inclination(inclination))

    return belt
end

--- Generate an asteroid ring
---@param rng RNG
---@param cfg Ruleset
---@param context GenerationContext
---@return Entity|nil
function UniverseManager:_generateAsteroidRing(rng, cfg, context)
    local rSeed = rng:get64()
    local ring = CelestialEntities.AsteroidRing(rSeed)
    if not ring then
        Log.Error("Failed to create AsteroidRing")
        return nil
    end

    local rRNG = RNG.Create(rSeed)
    if not rRNG then
        Log.Error("Failed to create RNG for AsteroidRing")
        return nil
    end

    local orbitRadius = context:get("orbitRadius") or 1.0
    context:set("ringOrbit", orbitRadius)

    local density = RuleEvaluator.evaluate(rRNG, cfg.asteroidRings.aspects.density, context) or 0.5
    context:set("ringDensity", density)

    local composition = RuleEvaluator.evaluate(rRNG, cfg.asteroidRings.aspects.composition, context) or
        { type = Enums.Gen.AsteroidRingTypes.Rocky }
    context:set("ringComposition", composition)

    local widthAU = RuleEvaluator.evaluate(rRNG, cfg.asteroidRings.aspects.width, context) or 0.1
    context:set("ringWidth", widthAU)

    local parentIncl = context:get("inclination") or 0
    local ringInclination = parentIncl + rRNG:getUniformRange(-3, 3)
    context:set("ringInclination", ringInclination)

    local ringPos = Position(self.scaleConfig:auToGameUnits(orbitRadius, "planet"), 0, 0)
    ring:get(PhysicsComponents.Transform):setPosition(ringPos)
    ring:add(CoreComponents.Type(composition.type))
    ring:add(CelestialComponents.Composition(composition))
    ring:add(CelestialComponents.Density(density))
    ring:add(SpatialComponents.Width(self.scaleConfig:auToGameUnits(widthAU, "ring")))
    ring:add(SpatialComponents.Inclination(ringInclination))

    return ring
end

return UniverseManager()
