local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local CelestialEntities = require("Modules.CelestialObjects.Entities")
local CelestialComponents = require("Modules.CelestialObjects.Components")
local SpatialEntities = require("Modules.Spatial.Entities")
local SpatialComponents = require("Modules.Spatial.Components")
local CoreComponents = require("Modules.Core.Components")
local PhysicsComponents = require("Modules.Physics.Components")
local Items = require("Shared.Registries.Items")
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

---@param name string
---@param ruleset Ruleset
function UniverseManager:registerRuleset(name, ruleset)
    RulesetValidator.validate(ruleset)
    self.rulesets[name] = ruleset
    ruleset.name = name
end

---@param ruleset string|Ruleset
---@param seed? integer
---@return Entity|nil
function UniverseManager:createUniverse(ruleset, seed)
    self.profiler:start()
    local cfg = self:_resolveRuleset(ruleset)
    local rng = RNG.Create(seed or cfg.seed)
    if not rng then
        Log.Error("Failed to create master RNG")
        self.profiler:stop()
        return nil
    end

    local universe = CelestialEntities.Universe(rng:get64())
    if not universe then
        Log.Error("Failed to create Universe entity")
        self.profiler:stop()
        return nil
    end

    self:_applyOverrides(universe, cfg.overrides and cfg.overrides.universe)

    local context = GenerationContext()
    local sysCount = RuleEvaluator.evaluate(rng, cfg.starSystems.count, context)
    for i = 1, sysCount do
        local sysSeed = rng:get64()
        local starSystem = CelestialEntities.StarSystem(sysSeed)
        if not starSystem then
            Log.Error("Failed to create StarSystem (seed: %s)", sysSeed)
            self.profiler:stop()
            return nil
        end

        local sysRNG = RNG.Create(sysSeed)
        if not sysRNG then
            Log.Error("Failed to create RNG for StarSystem (seed: %s)", sysSeed)
            self.profiler:stop()
            return nil
        end

        if not self:_generateStarSystem(starSystem, sysRNG, cfg, context) then
            Log.Error("StarSystem generation failed")
            self.profiler:stop()
            return nil
        end

        Registry:attachEntity(universe, starSystem)
    end

    Registry:printHierarchy(universe)
    self.profiler:stop()
    return universe
end

---@private
---@param ruleset string|Ruleset
---@return Ruleset
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

---@private
---@param parent Entity
---@param overrides EntityOverride[]|nil
function UniverseManager:_applyOverrides(parent, overrides)
    if not overrides or not parent then return end
    for _, def in ipairs(overrides) do
        local entity = CelestialEntities[def.type](self.rng:get64())
        if not entity then
            Log.Error("Override entity creation failed for type: %s", def.type)
            return
        end

        if def.position then
            entity:get(PhysicsComponents.Transform):setPosition(def.position)
        end
        for compName, value in pairs(def.components or {}) do
            local component = CoreComponents[compName] or SpatialComponents[compName]
            if component then
                entity:add(component(value))
            end
        end

        Registry:attachEntity(parent, entity)
    end
end

---@private
---@param starSystem Entity
---@param rng RNG
---@param cfg Ruleset
---@param context GenerationContext
---@return boolean
function UniverseManager:_generateStarSystem(starSystem, rng, cfg, context)
    if not starSystem or not rng then return false end

    context:set("systemAge", RuleEvaluator.evaluate(rng, cfg.starSystems.aspects.age, context))
    context:set("systemMetallicity", RuleEvaluator.evaluate(rng, cfg.starSystems.aspects.metallicity, context))
    context:set("stability", RuleEvaluator.evaluate(rng, cfg.starSystems.aspects.stability, context))

    starSystem:add(CelestialComponents.Age(context:get("systemAge")))
    starSystem:add(CelestialComponents.Metallicity(context:get("systemMetallicity")))
    starSystem:add(CelestialComponents.Stability(context:get("stability")))

    local star = self:_generateStar(rng, cfg, context)
    if not star then return false end
    Registry:attachEntity(starSystem, star)

    local planetCount = RuleEvaluator.evaluate(rng, cfg.planets.count, context)
    context:set("planetCount", planetCount)
    local orbits = self:_calculateOrbits(rng, cfg.planets.aspects.orbitRadius, planetCount)
    for i = 1, planetCount do
        local planet = self:_generatePlanet(rng, cfg, context, orbits[i])
        if not planet then return false end
        Registry:attachEntity(star, planet)
    end

    if RuleEvaluator.evaluate(rng, cfg.asteroidBelts.count, context) then
        local belt = self:_generateAsteroidBelt(rng, cfg, context)
        if not belt then return false end
        Registry:attachEntity(star, belt)
    end

    self:_applyOverrides(starSystem, cfg.overrides and cfg.overrides.starSystem)
    return true
end

---@private
---@param rng RNG
---@param rule Rule
---@param count integer
---@return number[]
function UniverseManager:_calculateOrbits(rng, rule, count)
    local orbits = {}
    for i = 1, count do
        local orbit = RuleEvaluator.evaluate(rng, rule, {})
        table.insert(orbits, math.exp(math.log(0.1) + (math.log(10.0) - math.log(0.1)) * (i - 1) / math.max(1, count - 1)))
    end
    table.sort(orbits)
    return orbits
end

---@private
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

    context:set("starType", RuleEvaluator.evaluate(starRNG, cfg.stars.aspects.type, context))
    context:set("starMass", RuleEvaluator.evaluate(starRNG, cfg.stars.aspects.mass, context))
    context:set("luminosity", RuleEvaluator.evaluate(starRNG, cfg.stars.aspects.luminosity, context))

    star:add(CoreComponents.Type(context:get("starType")))
    star:add(PhysicsComponents.Mass(context:get("starMass")))
    star:add(CelestialComponents.Luminosity(context:get("luminosity")))

    local zone = SpatialEntities.Zone()
    if not zone then
        Log.Error("Failed to create Zone")
        return nil
    end
    local shape = zone:get(SpatialComponents.Shape)
    if not shape then
        Log.Error("Zone missing Shape component")
        return nil
    end
    shape:setShape(Enums.ZoneShape.Sphere)
    shape:setRadius(RuleEvaluator.evaluate(starRNG, cfg.starZoneRadius, context))
    Registry:attachEntity(star, zone)

    return star
end

---@private
---@param rng RNG
---@param cfg Ruleset
---@param context GenerationContext
---@param orbitRadius number
---@return Entity|nil
function UniverseManager:_generatePlanet(rng, cfg, context, orbitRadius)
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

    context:set("orbitRadius", orbitRadius)
    context:set("planetType", RuleEvaluator.evaluate(pRNG, cfg.planets.aspects.type, context))
    context:set("planetSize", RuleEvaluator.evaluate(pRNG, cfg.planets.aspects.size, context))
    local atmosphere = RuleEvaluator.evaluate(pRNG, cfg.planets.aspects.atmosphere, context)
    context:set("asteroidRingType", RuleEvaluator.evaluate(pRNG, cfg.planets.aspects.asteroidRing, context))
    local temperature = RuleEvaluator.evaluate(pRNG, cfg.planets.aspects.temperature, context)
    local gravity = RuleEvaluator.evaluate(pRNG, cfg.planets.aspects.gravity, context)
    local rotationPeriod = RuleEvaluator.evaluate(pRNG, cfg.planets.aspects.rotationPeriod, context)
    local eccentricity = RuleEvaluator.evaluate(pRNG, cfg.planets.aspects.eccentricity, context)
    local magneticField = RuleEvaluator.evaluate(pRNG, cfg.planets.aspects.magneticField, context)
    local inclination = RuleEvaluator.evaluate(pRNG, cfg.planets.aspects.inclination, context)

    planet:add(SpatialComponents.Orbit(orbitRadius))
    planet:add(CoreComponents.Type(context:get("planetType")))
    planet:add(PhysicsComponents.Transform(context:get("planetSize")))
    if atmosphere then planet:add(CelestialComponents.Atmosphere()) end
    planet:add(CelestialComponents.Temperature(temperature))
    planet:add(CelestialComponents.Gravity(gravity))
    planet:add(CelestialComponents.Rotation(rotationPeriod))
    planet:add(CelestialComponents.Eccentricity(eccentricity))
    if magneticField then planet:add(CelestialComponents.MagneticField()) end
    planet:add(SpatialComponents.Inclination(inclination))

    if context:get("asteroidRingType") ~= "None" then
        local ring = self:_generateAsteroidRing(pRNG, cfg, context)
        if not ring then return nil end
        Registry:attachEntity(planet, ring)
    end

    local moonCount = RuleEvaluator.evaluate(pRNG, cfg.moons.count, context)
    for i = 1, moonCount do
        local moon = self:_generateMoon(pRNG, cfg, context)
        if not moon then return nil end
        Registry:attachEntity(planet, moon)
    end

    return planet
end

---@private
---@param rng RNG
---@param cfg Ruleset
---@param context GenerationContext
---@return Entity|nil
function UniverseManager:_generateAsteroidRing(rng, cfg, context)
    local ringSeed = rng:get64()
    local ring = CelestialEntities.AsteroidRing(ringSeed)
    if not ring then
        Log.Error("Failed to create AsteroidRing")
        return nil
    end

    local ringRNG = RNG.Create(ringSeed)
    if not ringRNG then
        Log.Error("Failed to create RNG for AsteroidRing")
        return nil
    end

    local composition = RuleEvaluator.evaluate(ringRNG, cfg.rings.aspects.composition, context)
    local thickness = RuleEvaluator.evaluate(ringRNG, cfg.rings.aspects.thickness, context)
    ring:add(CelestialComponents.Composition(composition))
    ring:add(CelestialComponents.Thickness(thickness))
    ring:add(CoreComponents.Type(context:get("asteroidRingType")))

    local astCount = ringRNG:getInt(5, 20)
    local itemCount = #Items.RawMaterials
    if itemCount <= 0 then
        Log.Error("No items registered - cannot populate asteroids")
        return nil
    end

    -- insert item id´s into a temp array
    local tempItemArray = {}
    for _, item in IteratorIndexed(Items.RawMaterials) do
        table.insert(tempItemArray, item.id)
    end

    --todo: resource weights

    for j = 1, astCount do
        local asteroidSeed = ringRNG:get64()
        local randomIndex = ringRNG:getInt(1, itemCount)
        local asteroidItem = Items:getDefinition(tempItemArray[randomIndex])
        if not asteroidItem then
            Log.Error("Item definition missing for index: %s", tempItemArray[randomIndex])
            return nil
        end

        local asteroid = CelestialEntities.Asteroid(asteroidSeed, asteroidItem)
        if not asteroid then
            Log.Error("Failed to create Asteroid with item")
            return nil
        end
        Registry:attachEntity(ring, asteroid)
    end

    return ring
end

---@private
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

    local moonRNG = RNG.Create(mSeed)
    if not moonRNG then
        Log.Error("Failed to create RNG for Moon")
        return nil
    end

    local moonSize = RuleEvaluator.evaluate(moonRNG, cfg.moons.aspects.size, context)
    local moonType = RuleEvaluator.evaluate(moonRNG, cfg.moons.aspects.type, context)
    local orbitalDistance = RuleEvaluator.evaluate(moonRNG, cfg.moons.aspects.orbitalDistance, context)
    local inclination = RuleEvaluator.evaluate(moonRNG, cfg.moons.aspects.inclination, context)

    moon:add(PhysicsComponents.Transform(moonSize))
    moon:add(CoreComponents.Type(moonType))
    moon:add(SpatialComponents.Orbit(orbitalDistance))
    moon:add(SpatialComponents.Inclination(inclination))

    return moon
end

---@private
---@param rng RNG
---@param cfg Ruleset
---@param context GenerationContext
---@return Entity|nil
function UniverseManager:_generateAsteroidBelt(rng, cfg, context)
    local beltSeed = rng:get64()
    local belt = CelestialEntities.AsteroidBelt(beltSeed)
    if not belt then
        Log.Error("Failed to create AsteroidBelt")
        return nil
    end

    local beltRNG = RNG.Create(beltSeed)
    if not beltRNG then
        Log.Error("Failed to create RNG for AsteroidBelt")
        return nil
    end

    context:set("orbitRadius", beltRNG:getUniformRange(2.0, 10.0))
    local density = RuleEvaluator.evaluate(beltRNG, cfg.asteroidBelts.aspects.density, context)
    local composition = RuleEvaluator.evaluate(beltRNG, cfg.asteroidBelts.aspects.composition, context)
    local width = RuleEvaluator.evaluate(beltRNG, cfg.asteroidBelts.aspects.width, context)

    belt:add(PhysicsComponents.Density(density))
    belt:add(CelestialComponents.Composition(composition))
    belt:add(SpatialComponents.Width(width))
    belt:add(SpatialComponents.Orbit(context:get("orbitRadius")))

    local astCount = beltRNG:getInt(10, 50)
    local itemCount = #Items.RawMaterials
    if itemCount <= 0 then
        Log.Error("No items registered - cannot populate asteroids")
        return nil
    end

    -- insert item id´s into a temp array
    local tempItemArray = {}
    for _, item in IteratorIndexed(Items.RawMaterials) do
        table.insert(tempItemArray, item.id)
    end

    --todo: resource weights

    for j = 1, astCount do
        local asteroidSeed = beltRNG:get64()
        local randomIndex = beltRNG:getInt(1, itemCount)
        local asteroidItem = Items:getDefinition(tempItemArray[randomIndex])
        if not asteroidItem then
            Log.Error("Item definition missing for index: %s", randomIndex)
            return nil
        end

        local asteroid = CelestialEntities.Asteroid(asteroidSeed, asteroidItem)
        if not asteroid then
            Log.Error("Failed to create Asteroid with item")
            return nil
        end
        Registry:attachEntity(belt, asteroid)
    end

    return belt
end

return UniverseManager()
