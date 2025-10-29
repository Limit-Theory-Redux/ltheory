local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local CelestialEntities = require("Modules.CelestialObjects.Entities")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local SpatialEntities = require("Modules.Spatial.Entities")
local SpatialComponents = require("Modules.Spatial.Components")
local CoreComponents = require("Modules.Core.Components")
local PhysicsComponents = require("Modules.Physics.Components")
local EconomyComponents = require("Modules.Economy.Components")
local RuleEvaluator = require("Modules.CelestialObjects.Managers.UniverseManager.RuleEvaluator")
local GenerationContext = require("Modules.CelestialObjects.Managers.UniverseManager.GenerationContext")
local RulesetValidator = require("Modules.CelestialObjects.Managers.UniverseManager.RulesetValidator")

---@class UniverseManager
---@field profiler QuickProfiler
---@field rulesets table<string, Ruleset>
---@overload fun(self: UniverseManager): UniverseManager class internal
---@overload fun(): UniverseManager class external
local UniverseManager = Class("UniverseManager", function(self)
    self:registerVars()
end)

function UniverseManager:registerVars()
    self.profiler = QuickProfiler("UniverseManager", false, false)
    self.rulesets = {}
end

function UniverseManager:registerRuleset(name, ruleset)
    RulesetValidator.validate(ruleset)
    self.rulesets[name] = ruleset
    ruleset.name = name
end

function UniverseManager:createUniverse(ruleset, seed)
    self.profiler:start()
    local cfg = self:_resolveRuleset(ruleset)
    local universeSeed = seed or cfg.seed

    if not universeSeed then
        Log.Error("Did not provide a seed for universe generation directly or via ruleset")
        return
    end

    local rng = RNG.Create(universeSeed)
    if not rng then
        Log.Error("Failed to create master RNG")
        self.profiler:stop()
        return nil
    end

    Log.Debug("Creating universe with seed '%s' and ruleset '%s'", tostring(universeSeed), cfg.name)

    local universe = CelestialEntities.Universe(rng:get64())
    if not universe then
        Log.Error("Failed to create Universe entity")
        self.profiler:stop()
        return nil
    end

    universe:add(CoreComponents.Type("None"))
    self:_applyOverrides(rng, universe, cfg.overrides and cfg.overrides.universe)

    local context = GenerationContext()
    local sysCount = RuleEvaluator.evaluate(rng, cfg.starSystems.count, context)
    Log.Debug("Generating %d star systems", sysCount)
    for i = 1, sysCount do
        local starSystem = self:_generateStarSystem(rng, cfg, context)
        if not starSystem then
            Log.Error("Failed to generate StarSystem %d", i)
            self.profiler:stop()
            return nil
        end
        Registry:attachEntity(universe, starSystem)
        Log.Debug("Attached StarSystem %d to Universe", i)
    end

    self.profiler:stop()
    return universe
end

function UniverseManager:_resolveRuleset(ruleset)
    if type(ruleset) == "string" then
        local s = self.rulesets[ruleset]
        if not s then
            Log.Error("Ruleset not registered: %s", ruleset)
            error("Ruleset not registered: " .. ruleset)
        end
        return s
    end
    return ruleset
end

function UniverseManager:_applyOverrides(rng, parent, overrides)
    if not overrides or not parent then return end
    for _, def in ipairs(overrides) do
        local entity = CelestialEntities[def.type](rng:get64())
        if not entity then
            Log.Error("Override entity creation failed for type: %s", def.type)
            return
        end

        if def.position then
            entity:get(PhysicsComponents.Transform):setPosition(def.position)
        end
        for compName, value in pairs(def.components or {}) do
            local component = CoreComponents[compName] or SpatialComponents[compName] or CelestialComponents[compName]
            if component then
                entity:add(component(value))
            end
        end

        Registry:attachEntity(parent, entity)
    end
end

function UniverseManager:_generateStarSystem(rng, cfg, context)
    local ssSeed = rng:get64()
    local starSystem = CelestialEntities.StarSystem(ssSeed)
    if not starSystem then
        Log.Error("Failed to create StarSystem entity with seed: %s", tostring(ssSeed))
        return nil
    end

    local ssRNG = RNG.Create(ssSeed)
    if not ssRNG then
        Log.Error("Failed to create RNG for StarSystem with seed: %s", tostring(ssSeed))
        return nil
    end

    local starCount = RuleEvaluator.evaluate(ssRNG, cfg.stars.count, context)
    if not starCount then
        Log.Error("Failed to evaluate starCount for StarSystem with seed: %s", tostring(ssSeed))
        return nil
    end
    context:set("starCount", starCount)
    Log.Debug("starCount: %s", tostring(starCount))

    local aspects = {
        { key = "systemType",        rule = cfg.starSystems.aspects.type },
        { key = "systemAge",         rule = cfg.starSystems.aspects.age },
        { key = "systemMetallicity", rule = cfg.starSystems.aspects.metallicity },
        { key = "stability",         rule = cfg.starSystems.aspects.stability }
    }

    for _, aspect in ipairs(aspects) do
        local value = RuleEvaluator.evaluate(ssRNG, aspect.rule, context)
        if not value then
            Log.Error("Failed to evaluate %s for StarSystem with seed: %s", aspect.key, tostring(ssSeed))
            return nil
        end
        context:set(aspect.key, value)
        Log.Debug("%s: %s", aspect.key, tostring(value))
    end

    starSystem:add(CoreComponents.Type(context:get("systemType")))
    starSystem:add(CelestialComponents.Age(context:get("systemAge")))
    starSystem:add(CelestialComponents.Metallicity(context:get("systemMetallicity")))
    starSystem:add(CelestialComponents.Stability(context:get("stability")))
    starSystem:get(PhysicsComponents.Transform):setPosition(Position(0, 0, 0))

    Log.Debug("Generating %d stars for StarSystem", starCount)
    for i = 1, starCount do
        context:set("starIndex", i) -- Set starIndex for position evaluation
        local star = self:_generateStar(ssRNG, cfg, context)
        if not star then
            Log.Error("Failed to generate Star %d for StarSystem with seed: %s", i, tostring(ssSeed))
            return nil
        end
        Registry:attachEntity(starSystem, star)
        Log.Debug("Attached Star %d to StarSystem", i)
    end

    local planetCount = RuleEvaluator.evaluate(ssRNG, cfg.planets.count, context)
    context:set("planetCount", planetCount)
    Log.Debug("Generating %d planets for StarSystem", planetCount)
    local orbits = self:_calculateOrbits(ssRNG, cfg.planets.aspects.orbitRadius, planetCount)
    for i = 1, planetCount do
        --Log.Debug("Planet %d context before evaluation:\n%s", i, table.tostring(context:getAll(), true))
        local planet = self:_generatePlanet(ssRNG, cfg, context, orbits[i])
        if not planet then
            Log.Error("Failed to generate Planet %d for StarSystem with seed: %s", i, tostring(ssSeed))
            return nil
        end
        Registry:attachEntity(starSystem, planet)
        Log.Debug("Attached Planet %d to StarSystem", i)
    end

    local hasAsteroidBelt = RuleEvaluator.evaluate(ssRNG, cfg.asteroidBelts.count, context)
    if hasAsteroidBelt then
        local belt = self:_generateAsteroidBelt(ssRNG, cfg, context)
        if not belt then
            Log.Error("Failed to generate AsteroidBelt for StarSystem with seed: %s", tostring(ssSeed))
            return nil
        end
        Registry:attachEntity(starSystem, belt)
        Log.Debug("Attached AsteroidBelt to StarSystem")
    end

    self:_applyOverrides(ssRNG, starSystem, cfg.overrides and cfg.overrides.starSystem)
    Log.Debug("Generated StarSystem with seed: %s", tostring(ssSeed))
    return starSystem
end

function UniverseManager:_calculateOrbits(rng, rule, count)
    local orbits = {}
    for i = 1, count do
        local orbit = RuleEvaluator.evaluate(rng, rule, GenerationContext())
        if not orbit then
            Log.Warn("Failed to evaluate orbitRadius, using default range")
            orbit = rng:getUniformRange(rule.min or 0.7, rule.max or 4.0)
        end
        table.insert(orbits,
            math.exp(math.log(rule.min or 0.7) + (math.log(rule.max or 4.0) - math.log(rule.min or 0.7)) * (i - 1) / math.max(1, count - 1)))
    end
    table.sort(orbits)
    return orbits
end

function UniverseManager:_generateStar(rng, cfg, context)
    local starSeed = rng:get64()
    local star = CelestialEntities.Star(starSeed)
    if not star then
        Log.Error("Failed to create Star with seed: %s", tostring(starSeed))
        return nil
    end

    local starRNG = RNG.Create(starSeed)
    if not starRNG then
        Log.Error("Failed to create RNG for Star with seed: %s", tostring(starSeed))
        return nil
    end

    local aspects = {
        { key = "starType",   rule = cfg.stars.aspects.type },
        { key = "starMass",   rule = cfg.stars.aspects.mass },
        { key = "luminosity", rule = cfg.stars.aspects.luminosity },
        { key = "position",   rule = cfg.stars.aspects.position }
    }

    for _, aspect in ipairs(aspects) do
        --Log.Debug("Evaluating aspect %s with rule: %s", aspect.key, table.tostring(aspect.rule, true))
        local value = RuleEvaluator.evaluate(starRNG, aspect.rule, context)
        if not value then
            if aspect.key == "position" then
                value = cfg.stars.aspects.position.default or Position(0, 0, 0)
                Log.Warn("Failed to evaluate %s for Star with seed: %s, using default: %s", aspect.key, tostring(starSeed),
                    table.tostring(value, true))
            else
                Log.Error("Failed to evaluate %s for Star with seed: %s", aspect.key, tostring(starSeed))
                return nil
            end
        end
        context:set(aspect.key, value)
        Log.Debug("%s: %s", aspect.key, tostring(value))
    end

    star:add(CoreComponents.Type(context:get("starType")))
    star:add(CelestialComponents.Luminosity(context:get("luminosity")))
    star:get(PhysicsComponents.Mass):setMass(context:get("starMass"))
    star:get(PhysicsComponents.Transform):setPosition(context:get("position"))

    local zone = SpatialEntities.Zone()
    if not zone then
        Log.Error("Failed to create Zone for Star with seed: %s", tostring(starSeed))
        return nil
    end
    local shape = zone:get(SpatialComponents.Shape)
    if not shape then
        Log.Error("Zone missing Shape component for Star with seed: %s", tostring(starSeed))
        return nil
    end
    shape:setShape(Enums.ZoneShape.Sphere)
    shape:setRadius(RuleEvaluator.evaluate(starRNG, cfg.starZoneRadius, context))
    Registry:attachEntity(star, zone)
    Log.Debug("Attached Zone to Star with seed: %s", tostring(starSeed))

    Log.Debug("Generated Star with seed: %s", tostring(starSeed))
    return star
end

function UniverseManager:_generatePlanet(rng, cfg, context, orbitRadius)
    local pSeed = rng:get64()
    local planet = CelestialEntities.Planet(pSeed)
    if not planet then
        Log.Error("Failed to create Planet entity with seed: %s", tostring(pSeed))
        return nil
    end

    local pRNG = RNG.Create(pSeed)
    if not pRNG then
        Log.Error("Failed to create RNG for Planet with seed: %s", tostring(pSeed))
        return nil
    end

    context:set("orbitRadius", orbitRadius)
    Log.Debug("Generating planet with orbitRadius: %s", tostring(orbitRadius))

    local aspects = {
        { key = "planetType",       rule = cfg.planets.aspects.type },
        { key = "planetSize",       rule = cfg.planets.aspects.size },
        { key = "atmosphere",       rule = cfg.planets.aspects.atmosphere },
        { key = "asteroidRingType", rule = cfg.planets.aspects.asteroidRing },
        { key = "temperature",      rule = cfg.planets.aspects.temperature },
        { key = "gravity",          rule = cfg.planets.aspects.gravity },
        { key = "rotationPeriod",   rule = cfg.planets.aspects.rotationPeriod },
        { key = "eccentricity",     rule = cfg.planets.aspects.eccentricity },
        { key = "magneticField",    rule = cfg.planets.aspects.magneticField },
        { key = "inclination",      rule = cfg.planets.aspects.inclination }
    }

    for _, aspect in ipairs(aspects) do
        local value = RuleEvaluator.evaluate(pRNG, aspect.rule, context)
        if not value and (aspect.key == "planetType" or aspect.key == "planetSize") then
            Log.Error("Failed to evaluate %s for Planet with seed: %s", aspect.key, tostring(pSeed))
            return nil
        end
        context:set(aspect.key, value)
        Log.Debug("%s: %s", aspect.key, tostring(value))
    end

    planet:add(SpatialComponents.Orbit(context:get("orbitRadius")))
    planet:add(CoreComponents.Type(context:get("planetType")))
    planet:add(PhysicsComponents.Transform(context:get("planetSize")))
    if context:get("atmosphere") then
        planet:add(CelestialComponents.Atmosphere())
        Log.Debug("Added Atmosphere component to planet")
    end
    planet:add(CelestialComponents.Temperature(context:get("temperature")))
    planet:add(CelestialComponents.Gravity(context:get("gravity")))
    planet:add(CelestialComponents.RotationPeriod(context:get("rotationPeriod")))
    planet:add(CelestialComponents.Eccentricity(context:get("eccentricity")))
    if context:get("magneticField") then
        planet:add(CelestialComponents.MagneticField())
        Log.Debug("Added MagneticField component to planet")
    end
    planet:add(SpatialComponents.Inclination(context:get("inclination")))

    local asteroidRingType = context:get("asteroidRingType")
    if asteroidRingType and asteroidRingType ~= Enums.Gen.AsteroidRingTypes.None then
        local ring = self:_generateAsteroidRing(pRNG, cfg, context)
        if not ring then
            Log.Error("Failed to generate AsteroidRing for Planet with seed: %s", tostring(pSeed))
            return nil
        end
        Registry:attachEntity(planet, ring)
        Log.Debug("Attached AsteroidRing (%s) to Planet", tostring(asteroidRingType))
    end

    local moonCount = RuleEvaluator.evaluate(pRNG, cfg.moons.count, context)
    Log.Debug("Generating %d moons for Planet with seed %s, planetType: %s", moonCount, tostring(pSeed), tostring(context:get("planetType")))
    for i = 1, moonCount do
        local moonContext = GenerationContext()
        moonContext:set("planetType", context:get("planetType"))
        moonContext:set("orbitRadius", context:get("orbitRadius"))
        moonContext:set("planetCount", context:get("planetCount"))
        moonContext:set("starCount", context:get("starCount"))
        moonContext:set("systemAge", context:get("systemAge"))
        moonContext:set("systemMetallicity", context:get("systemMetallicity"))
        moonContext:set("stability", context:get("stability"))

        local moon = self:_generateMoon(pRNG, cfg, moonContext)
        if not moon then
            Log.Error("Failed to generate Moon %d for Planet with seed: %s", i, tostring(pSeed))
            return nil
        end
        Registry:attachEntity(planet, moon)
        Log.Debug("Attached Moon %d to Planet", i)
    end

    self:_applyOverrides(pRNG, planet, cfg.overrides and cfg.overrides.planet)
    Log.Debug("Generated Planet with seed: %s", tostring(pSeed))
    return planet
end

function UniverseManager:_generateMoon(rng, cfg, context)
    local mSeed = rng:get64()
    local moon = CelestialEntities.Moon(mSeed)
    if not moon then
        Log.Error("Failed to create Moon entity with seed: %s", tostring(mSeed))
        return nil
    end

    local moonRNG = RNG.Create(mSeed)
    if not moonRNG then
        Log.Error("Failed to create RNG for Moon with seed: %s", tostring(mSeed))
        return nil
    end

    --Log.Debug("Moon context with seed %s:\n%s", tostring(mSeed), table.tostring(context:getAll(), true))

    local aspects = {
        { key = "moonType",        rule = cfg.moons.aspects.type },
        { key = "moonSize",        rule = cfg.moons.aspects.size },
        { key = "orbitalDistance", rule = cfg.moons.aspects.orbitalDistance },
        { key = "inclination",     rule = cfg.moons.aspects.inclination }
    }

    for _, aspect in ipairs(aspects) do
        local value = RuleEvaluator.evaluate(moonRNG, aspect.rule, context)
        if not value then
            if aspect.key == "moonType" then
                value = Enums.Gen.MoonTypes.Rocky
                Log.Warn("Failed to evaluate %s for Moon with seed: %s, using default: %s", aspect.key, tostring(mSeed), tostring(value))
            else
                Log.Error("Failed to evaluate %s for Moon with seed: %s", aspect.key, tostring(mSeed))
                return nil
            end
        end
        context:set(aspect.key, value)
        Log.Debug("%s: %s", aspect.key, tostring(value))
    end

    moon:add(CoreComponents.Type(context:get("moonType")))
    moon:add(PhysicsComponents.Transform(context:get("moonSize")))
    moon:add(SpatialComponents.Orbit(context:get("orbitalDistance")))
    moon:add(SpatialComponents.Inclination(context:get("inclination")))

    Log.Debug("Generated Moon with seed: %s", tostring(mSeed))
    return moon
end

function UniverseManager:_generateAsteroidRing(rng, cfg, context)
    local ringSeed = rng:get64()
    local ring = CelestialEntities.AsteroidRing(ringSeed)
    if not ring then
        Log.Error("Failed to create AsteroidRing entity with seed: %s", tostring(ringSeed))
        return nil
    end

    local ringRNG = RNG.Create(ringSeed)
    if not ringRNG then
        Log.Error("Failed to create RNG for AsteroidRing with seed: %s", tostring(ringSeed))
        return nil
    end

    local aspects = {
        { key = "ringComposition", rule = cfg.asteroidRings.aspects.composition },
        { key = "ringThickness",   rule = cfg.asteroidRings.aspects.thickness }
    }

    for _, aspect in ipairs(aspects) do
        local value = RuleEvaluator.evaluate(ringRNG, aspect.rule, context)
        if not value then
            Log.Error("Failed to evaluate %s for AsteroidRing with seed: %s", aspect.key, tostring(ringSeed))
            return nil
        end
        context:set(aspect.key, value)
        Log.Debug("%s: %s", aspect.key, tostring(value))
    end

    local composition = context:get("ringComposition")
    if not composition or not composition.type then
        Log.Error("Invalid composition rule result for AsteroidRing with seed: %s", tostring(ringSeed))
        return nil
    end

    ring:add(CoreComponents.Type(context:get("asteroidRingType")))
    ring:add(CelestialComponents.Composition(composition.type))
    ring:add(CelestialComponents.Thickness(context:get("ringThickness")))

    Log.Debug("Generated AsteroidRing with seed: %s", tostring(ringSeed))
    return ring
end

function UniverseManager:_generateAsteroidBelt(rng, cfg, context)
    local beltSeed = rng:get64()
    local belt = CelestialEntities.AsteroidBelt(beltSeed)
    if not belt then
        Log.Error("Failed to create AsteroidBelt entity with seed: %s", tostring(beltSeed))
        return nil
    end

    local beltRNG = RNG.Create(beltSeed)
    if not beltRNG then
        Log.Error("Failed to create RNG for AsteroidBelt with seed: %s", tostring(beltSeed))
        return nil
    end

    local aspects = {
        { key = "beltOrbitRadius", rule = { type = Enums.Gen.Rule.Range, min = 2.0, max = 10.0 } },
        { key = "beltDensity",     rule = cfg.asteroidBelts.aspects.density },
        { key = "beltComposition", rule = cfg.asteroidBelts.aspects.composition },
        { key = "beltWidth",       rule = cfg.asteroidBelts.aspects.width }
    }

    for _, aspect in ipairs(aspects) do
        local value = RuleEvaluator.evaluate(beltRNG, aspect.rule, context)
        if not value then
            Log.Error("Failed to evaluate %s for AsteroidBelt with seed: %s", aspect.key, tostring(beltSeed))
            return nil
        end
        context:set(aspect.key, value)
        Log.Debug("%s: %s", aspect.key, tostring(value))
    end

    local composition = context:get("beltComposition")
    if not composition or not composition.type then
        Log.Error("Invalid composition rule result for AsteroidBelt with seed: %s", tostring(beltSeed))
        return nil
    end

    belt:add(CoreComponents.Type(composition.type))
    belt:add(CelestialComponents.Density(context:get("beltDensity")))
    belt:add(CelestialComponents.Composition(composition.type))
    belt:add(SpatialComponents.Width(context:get("beltWidth")))
    belt:add(SpatialComponents.Orbit(context:get("beltOrbitRadius")))

    Log.Debug("Generated AsteroidBelt with seed: %s", tostring(beltSeed))
    return belt
end

return UniverseManager()
