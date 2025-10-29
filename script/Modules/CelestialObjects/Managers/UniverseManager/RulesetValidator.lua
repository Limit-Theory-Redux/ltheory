local Items = require("Shared.Registries.Items")

---@class RulesetValidator
local RulesetValidator = {}

---@param rule Rule
local function validateRule(rule)
    if not rule.type then
        Log.Error("Rule missing type")
        error("Invalid rule: missing type")
    end

    if rule.type == Enums.Gen.Rule.Range or rule.type == Enums.Gen.Rule.Count then
        if rule.min == nil or rule.max == nil then
            Log.Error("Rule type %s requires min and max", tostring(rule.type))
            error("Invalid rule: missing min or max")
        end
        if rule.min > rule.max then
            Log.Error("Rule type %s has min > max: %s > %s", tostring(rule.type), rule.min, rule.max)
            error("Invalid rule: min > max")
        end
    elseif rule.type == Enums.Gen.Rule.Weighted then
        if not rule.values or #rule.values == 0 then
            Log.Error("Weighted rule has no values")
            error("Invalid rule: no values")
        end
        for _, v in ipairs(rule.values) do
            if v.weight < 0 then
                Log.Error("Weighted rule has negative weight: %s", v.weight)
                error("Invalid rule: negative weight")
            end
        end
    elseif rule.type == Enums.Gen.Rule.Chance then
        if rule.value == nil or rule.value < 0 or rule.value > 1 then
            Log.Error("Chance rule has invalid probability: %s", tostring(rule.value))
            error("Invalid rule: invalid probability")
        end
    elseif rule.type == Enums.Gen.Rule.Custom then
        if not rule.fn then
            Log.Error("Custom rule missing function")
            error("Invalid rule: missing function")
        end
    elseif rule.type == Enums.Gen.Rule.Fixed then
        if rule.value == nil then
            Log.Error("Fixed rule missing value")
            error("Invalid rule: missing value")
        end
    end

    if rule.condition then
        if not rule.condition.type then
            Log.Error("Condition missing type")
            error("Invalid condition: missing type")
        end
        if rule.condition.ranges then
            for _, range in ipairs(rule.condition.ranges) do
                if range.min > range.max then
                    Log.Error("Condition range has min > max: %s > %s", range.min, range.max)
                    error("Invalid condition: min > max")
                end
            end
        end
    end
end

---@param ruleset Ruleset
function RulesetValidator.validate(ruleset)
    if not ruleset.name then
        Log.Error("Ruleset missing name")
        error("Invalid ruleset: missing name")
    end

    -- Validate starSystems
    validateRule(ruleset.starSystems.count)
    for aspectName, rule in pairs(ruleset.starSystems.aspects) do
        validateRule(rule)
        if aspectName == "type" then
            if rule.type == Enums.Gen.Rule.Fixed and rule.condition and rule.condition.type == Enums.Gen.Condition.StarCount then
                for _, range in ipairs(rule.condition.ranges) do
                    if not Enums.Gen.StarSystemTypes[range.value] then
                        Log.Error("Invalid StarSystemType in rule: %s", tostring(range.value))
                        error("Invalid StarSystemType")
                    end
                end
            end
        end
    end

    -- Validate stars
    validateRule(ruleset.stars.count)
    if ruleset.stars.count.type == Enums.Gen.Rule.Weighted then
        for _, v in ipairs(ruleset.stars.count.values) do
            if v.value < 1 or v.value > 3 then
                Log.Error("Star count must be between 1 and 3, got: %d", v.value)
                error("Invalid star count")
            end
        end
    end
    for _, rule in pairs(ruleset.stars.aspects) do
        validateRule(rule)
    end

    -- Validate planets
    validateRule(ruleset.planets.count)
    for aspectName, rule in pairs(ruleset.planets.aspects) do
        validateRule(rule)
        if aspectName == "atmosphere" and rule.type == Enums.Gen.Rule.Chance then
            if rule.condition and rule.condition.type == Enums.Gen.Condition.PlanetType then
                for planetType, config in pairs(rule.condition.types) do
                    if not Enums.Gen.PlanetTypes[planetType] then
                        Log.Error("Invalid PlanetType in atmosphere rule: %s", tostring(planetType))
                        error("Invalid PlanetType")
                    end
                    if config.chance and (config.chance < 0 or config.chance > 1) then
                        Log.Error("Invalid chance value %f for PlanetType %s", config.chance, planetType)
                        error("Invalid chance value")
                    end
                end
            end
            if rule.default == nil then
                Log.Warn("Atmosphere rule missing default value, assuming false")
                rule.default = false
            end
        end
    end

    -- Validate moons
    validateRule(ruleset.moons.count)
    for _, rule in pairs(ruleset.moons.aspects) do
        validateRule(rule)
    end

    -- Validate asteroidRings
    validateRule(ruleset.asteroidRings.count)
    for _, rule in pairs(ruleset.asteroidRings.aspects) do
        validateRule(rule)
        if rule == ruleset.asteroidRings.aspects.composition then
            if rule.type ~= Enums.Gen.Rule.Weighted then
                Log.Error("AsteroidRing composition rule must be Weighted")
                error("Invalid composition rule")
            end
            for _, v in ipairs(rule.values or {}) do
                if not v.value.type or not v.value.items then
                    Log.Error("Invalid composition value structure")
                    error("Invalid composition rule")
                end
                for _, item in ipairs(v.value.items) do
                    if not Items:getDefinition(item.id) then
                        Log.Error("Invalid itemId in composition rule: %s", item.id)
                        error("Invalid composition rule")
                    end
                end
            end
            if rule.condition then
                for _, range in ipairs(rule.condition.ranges or {}) do
                    if range.itemWeights then
                        for compType, weights in pairs(range.itemWeights) do
                            for itemId, _ in pairs(weights) do
                                if not Items:getDefinition(itemId) then
                                    Log.Error("Invalid itemId in composition condition itemWeights: %s", itemId)
                                    error("Invalid composition rule")
                                end
                            end
                        end
                    end
                end
            end
        end
    end

    -- Validate asteroidBelts
    validateRule(ruleset.asteroidBelts.count)
    for _, rule in pairs(ruleset.asteroidBelts.aspects) do
        validateRule(rule)
        if rule == ruleset.asteroidBelts.aspects.composition then
            if rule.type ~= Enums.Gen.Rule.Weighted then
                Log.Error("AsteroidBelt composition rule must be Weighted")
                error("Invalid composition rule")
            end
            for _, v in ipairs(rule.values or {}) do
                if not v.value.type or not v.value.items then
                    Log.Error("Invalid composition value structure")
                    error("Invalid composition rule")
                end
                for _, item in ipairs(v.value.items) do
                    if not Items:getDefinition(item.id) then
                        Log.Error("Invalid itemId in composition rule: %s", item.id)
                        error("Invalid composition rule")
                    end
                end
            end
            if rule.condition then
                for _, range in ipairs(rule.condition.ranges or {}) do
                    if range.itemWeights then
                        for compType, weights in pairs(range.itemWeights) do
                            for itemId, _ in pairs(weights) do
                                if not Items:getDefinition(itemId) then
                                    Log.Error("Invalid itemId in composition condition itemWeights: %s", itemId)
                                    error("Invalid composition rule")
                                end
                            end
                        end
                    end
                end
            end
        end
    end

    -- Validate starZoneRadius
    validateRule(ruleset.starZoneRadius)
end

return RulesetValidator
