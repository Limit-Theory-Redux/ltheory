return {
    StandardSolarSystem = require("Config.Gen.Rulesets.StandardSolarSystem"),
    StandardBinarySolarSystem = require("Config.Gen.Rulesets.StandardBinarySolarSystem"),
    StandardTrinarySolarSystem = require("Config.Gen.Rulesets.StandardTrinarySolarSystem"),

    Tests = {
        TwoAsteroidsOnePlayerShip = require("Config.Gen.Rulesets.Tests.TwoAsteroidsOnePlayerShip")
    }
}

---@class Rule
---@field type GenRule
---@field min? number
---@field max? number
---@field value? any
---@field values? { value: any, weight: number }[]
---@field fn? fun(rng: RNG, rule: Rule, context: GenerationContext): any
---@field condition? Condition
---@field default? any

---@class Condition
---@field type GenCondition
---@field ranges? { min: number, max: number, minSize?: number, maxSize?: number, minTemp?: number, maxTemp?: number, minGravity?: number, maxGravity?: number, minWidth?: number, maxWidth?: number, minDensity?: number, maxDensity?: number, minDistance?: number, maxDistance?: number, minLuminosity?: number, maxLuminosity?: number, weights?: table<integer, number>, itemWeights?: table<integer, table<string, number>>, value?: any }[]
---@field types? table<integer, { min?: number, max?: number, chance?: number, weights?: table<integer, number> }>

---@class Ruleset
---@field name string
---@field seed? integer
---@field starSystems StarSystemRules
---@field stars StarRules
---@field planets PlanetRules
---@field moons MoonRules
---@field asteroidRings RingRules
---@field asteroidBelts AsteroidBeltRules
---@field starZoneRadius Rule
---@field overrides? { universe?: EntityOverride[], starSystem?: EntityOverride[] }

---@class EntityOverride
---@field type string
---@field position? Position
---@field components? table<string, any>

---@class StarSystemRules
---@field count Rule
---@field aspects { age: Rule, metallicity: Rule, stability: Rule }

---@class StarRules
---@field count Rule
---@field aspects { type: Rule, mass: Rule, luminosity: Rule }

---@class PlanetRules
---@field count Rule
---@field aspects { orbitRadius: Rule, size: Rule, type: Rule, atmosphere: Rule, asteroidRing: Rule, temperature: Rule, gravity: Rule, rotationPeriod: Rule, eccentricity: Rule, magneticField: Rule, inclination: Rule }

---@class MoonRules
---@field count Rule
---@field aspects { size: Rule, type: Rule, orbitalDistance: Rule, inclination: Rule }

---@class RingRules
---@field count Rule
---@field aspects { composition: Rule, thickness: Rule }

---@class AsteroidBeltRules
---@field count Rule
---@field aspects { density: Rule, composition: Rule, width: Rule }
