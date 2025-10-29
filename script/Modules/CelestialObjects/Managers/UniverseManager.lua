---@alias RuleValue
---| number
---| boolean
---| fun(rng: RNG, rule: GenerationRule): any

---@class GenerationRule
---@field type integer
---@field min? number
---@field max? number
---@field value? RuleValue
---@field fn? fun(rng: RNG, rule: GenerationRule): any

---@class Scenario
---@field name string
---@field seed? integer
---@field rules table<string, GenerationRule>   -- key = generation step
---@field overrides table<string, table>       -- key = step, value = array of forced entities

-- Systems/UniverseManager.lua
local Registry          = require("Core.ECS.Registry")
local QuickProfiler     = require("Shared.Tools.QuickProfiler")
local CelestialObjects  = require("Modules.CelestialObjects.Entities")
local SpatialEntities   = require("Modules.Spatial.Entities")
local SpatialComponents = require("Modules.Spatial.Components")
local CoreComponents    = require("Modules.Core.Components")
local PhysicsComponents = require("Modules.Physics.Components")
local Items             = require("Shared.Registries.Items")

---@class UniverseManager
---@overload fun(self: UniverseManager): UniverseManager class internal
---@overload fun(): UniverseManager class external
local UniverseManager   = Class("UniverseManager", function(self)
    self:registerVars()
end)

function UniverseManager:registerVars()
    self.profiler  = QuickProfiler("UniverseManager", false, false)
    self.scenarios = {} -- name → Scenario table
end

-- --------------------------------------------------------------------- --
-- PUBLIC API
-- --------------------------------------------------------------------- --

---@param name string
---@param scenario table
function UniverseManager:registerScenario(name, scenario)
    scenario.name = name
    self.scenarios[name] = scenario
end

---@param scenario string|table
---@param seed? integer
---@return Entity|nil
function UniverseManager:createUniverse(scenario, seed)
    self.profiler:start()

    local cfg = self:_resolveScenario(scenario)
    self.rng = RNG.Create(seed or cfg.seed)
    if not self.rng then
        Log.Error("Failed to create master RNG")
        self.profiler:stop()
        return nil
    end

    local universe = CelestialObjects.Universe(self.rng:get64())
    if not universe then
        Log.Error("Failed to create Universe entity")
        self.profiler:stop()
        return nil
    end

    self:_applyOverrides(universe, cfg.overrides and cfg.overrides.universe)

    local sysCount = self:_evaluateRule(self.rng, cfg.rules.starSystems or { type = Enums.Gen.Rule.Count, min = 1, max = 3 })
    for i = 1, sysCount do
        local sysSeed = self.rng:get64()
        local starSystem = CelestialObjects.StarSystem(sysSeed)
        if not starSystem then
            Log.Error("Failed to create StarSystem (seed: " .. sysSeed .. ")")
            self.profiler:stop()
            return nil
        end

        local sysRNG = RNG.Create(sysSeed)
        if not sysRNG then
            Log.Error("Failed to create RNG for StarSystem (seed: " .. sysSeed .. ")")
            self.profiler:stop()
            return nil
        end

        if not self:_generateStarSystem(starSystem, sysRNG, cfg) then
            Log.Error("StarSystem generation failed")
            self.profiler:stop()
            return nil
        end

        Registry:attachEntity(universe, starSystem)
    end

    self.profiler:stop()
    return universe
end

-- --------------------------------------------------------------------- --
-- INTERNAL HELPERS
-- --------------------------------------------------------------------- --
---@private
function UniverseManager:_resolveScenario(scenario)
    if type(scenario) == "string" then
        local s = self.scenarios[scenario]
        if not s then error("Scenario not registered: " .. scenario) end
        return s
    end
    return scenario
end

---@private
---@param rng RNG
---@param rule GenerationRule
---@return any
function UniverseManager:_evaluateRule(rng, rule)
    if not rng then return 0 end

    if rule.type == Enums.Gen.Rule.Count then
        local min = rule.min or 0
        local max = rule.max or 0
        if min > max then min, max = max, min end
        return rng:getInt(min, max)
    elseif rule.type == Enums.Gen.Rule.Chance then
        local prob = rule.value or 0
        return rng:getUniform() < prob
    elseif rule.type == Enums.Gen.Rule.Fixed then
        return rule.value
    elseif rule.type == Enums.Gen.Rule.Custom then
        if rule.fn then
            local ok, result = pcall(rule.fn, rng, rule)
            if ok then return result end
            Log.Error("Custom rule failed: " .. result)
        end
        return 0
    end

    return 0
end

---@private
---@param parent Entity
---@param overrides table|nil
function UniverseManager:_applyOverrides(parent, overrides)
    if not overrides or not parent then return end
    for _, def in ipairs(overrides) do
        local entity = def.type
        ---@cast entity Entity

        if not entity then
            Log.Error("Override entity creation failed")
            return
        end

        if def.position then
            entity:get(PhysicsComponents.Transform):setPosition(def.position)
        end

        Registry:attachEntity(parent, entity)
    end
end

-- --------------------------------------------------------------------- --
-- GENERATION PHASES – fail-fast, deterministic
-- --------------------------------------------------------------------- --

---@private
---@param starSystem Entity
---@param rng RNG
---@param cfg table
---@return boolean success
function UniverseManager:_generateStarSystem(starSystem, rng, cfg)
    if not starSystem or not rng then return false end

    local starSeed = rng:get64()
    local star = CelestialObjects.Star(starSeed)
    if not star then
        Log.Error("Failed to create Star")
        return false
    end

    local starRNG = RNG.Create(starSeed)
    if not starRNG then
        Log.Error("Failed to create RNG for Star")
        return false
    end

    -- Star zone
    local zone = SpatialEntities.Zone()
    if not zone then
        Log.Error("Failed to create Zone")
        return false
    end
    local shape = zone:get(SpatialComponents.Shape)
    if not shape then
        Log.Error("Zone missing Shape component")
        return false
    end
    shape:setShape(Enums.ZoneShape.Sphere)
    shape:setRadius(cfg.rules.starZoneRadius or 1.7952e13)

    Registry:attachEntity(starSystem, star)

    -- PLANETS
    local planetRule = cfg.rules.planets or { type = Enums.Gen.Rule.Count, min = 1, max = 3 }
    local planetCount = self:_evaluateRule(starRNG, planetRule)
    for i = 1, planetCount do
        local pSeed = rng:get64()
        local planet = CelestialObjects.Planet(pSeed)
        if not planet then
            Log.Error("Failed to create Planet")
            return false
        end

        local pRNG = RNG.Create(pSeed)
        if not pRNG then
            Log.Error("Failed to create RNG for Planet")
            return false
        end

        if not self:_generatePlanetaryFeatures(planet, pRNG, cfg) then
            Log.Error("Planetary feature generation failed")
            return false
        end

        Registry:attachEntity(star, planet)
    end

    -- OVERRIDES
    self:_applyOverrides(starSystem, cfg.overrides and cfg.overrides.starSystem)

    -- CONSTRUCTS
    if not self:_generateConstructs(starSystem, rng, cfg) then
        Log.Error("Construct generation failed")
        return false
    end

    return true
end

---@private
---@param planet Entity
---@param rng RNG
---@param cfg table
---@return boolean success
function UniverseManager:_generatePlanetaryFeatures(planet, rng, cfg)
    if not planet or not rng then return false end

    -- MOONS
    local moonRule = cfg.rules.moons or { type = Enums.Gen.Rule.Count, min = 0, max = 3 }
    local moonCount = self:_evaluateRule(rng, moonRule)
    for i = 1, moonCount do
        local mSeed = rng:get64()
        local moon = CelestialObjects.Moon(mSeed)
        if not moon then
            Log.Error("Failed to create Moon")
            return false
        end
        Registry:attachEntity(planet, moon)
    end

    -- RINGS
    local ringRule = cfg.rules.rings or { type = Enums.Gen.Rule.Chance, value = 0.3 }
    if self:_evaluateRule(rng, ringRule) then
        local ringSeed = rng:get64()
        local ring = CelestialObjects.AsteroidRing(ringSeed)
        if not ring then
            Log.Error("Failed to create AsteroidRing")
            return false
        end

        local ringRNG = RNG.Create(ringSeed)
        if not ringRNG then
            Log.Error("Failed to create RNG for AsteroidRing")
            return false
        end

        Registry:attachEntity(planet, ring)

        local astCount = ringRNG:getInt(5, 20)
        local itemCount = Items:getItemCount()
        if itemCount <= 0 then
            Log.Error("No items registered – cannot populate asteroids")
            return false
        end

        for j = 1, astCount do
            local asteroidSeed = ringRNG:get64()
            local itemIdx = ringRNG:getInt(1, itemCount)
            local asteroidItem = Items:getDefinition(itemIdx)
            if not asteroidItem then
                Log.Error("Item definition missing for index: " .. itemIdx)
                return false
            end

            local asteroid = CelestialObjects.Asteroid(asteroidSeed, asteroidItem)
            if not asteroid then
                Log.Error("Failed to create Asteroid with item")
                return false
            end

            Registry:attachEntity(ring, asteroid)
        end
    end

    return true
end

---@private
---@param starSystem Entity
---@param rng RNG
---@param cfg table
---@return boolean success
function UniverseManager:_generateConstructs(starSystem, rng, cfg)
    if not starSystem or not rng then return false end

    local stationRule  = cfg.rules.stations or { type = Enums.Gen.Rule.Count, min = 0, max = 3 }
    local shipRule     = cfg.rules.ships or { type = Enums.Gen.Rule.Count, min = 0, max = 5 }

    local stationCount = self:_evaluateRule(rng, stationRule)
    for i = 1, stationCount do
        local station = CelestialObjects.SpaceStation(rng:get64())
        if not station then
            Log.Error("Failed to create SpaceStation")
            return false
        end
        Registry:attachEntity(starSystem, station)
    end

    local shipCount = self:_evaluateRule(rng, shipRule)
    for i = 1, shipCount do
        local ship = CelestialObjects.Spaceship(rng:get64())
        if not ship then
            Log.Error("Failed to create Spaceship")
            return false
        end
        Registry:attachEntity(starSystem, ship)
    end

    return true
end

return UniverseManager()
