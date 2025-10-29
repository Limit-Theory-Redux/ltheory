---@class RulesetValidator
local RulesetValidator = {}

---@param rule Rule
local function validateRule(rule)
    if not rule.type then
        Log.Error("Rule missing type")
        error("Invalid rule: missing type")
    end

    if rule.type == "Range" or rule.type == "Count" then
        if rule.min == nil or rule.max == nil then
            Log.Error("Rule type %s requires min and max", rule.type)
            error("Invalid rule: missing min or max")
        end
        if rule.min > rule.max then
            Log.Error("Rule type %s has min > max: %s > %s", rule.type, rule.min, rule.max)
            error("Invalid rule: min > max")
        end
    elseif rule.type == "Weighted" then
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
    elseif rule.type == "Chance" then
        if rule.value == nil or rule.value < 0 or rule.value > 1 then
            Log.Error("Chance rule has invalid probability: %s", tostring(rule.value))
            error("Invalid rule: invalid probability")
        end
    elseif rule.type == "Custom" then
        if not rule.fn then
            Log.Error("Custom rule missing function")
            error("Invalid rule: missing function")
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
    for _, rule in pairs(ruleset.starSystems.aspects) do
        validateRule(rule)
    end

    -- Validate stars
    validateRule(ruleset.stars.count)
    for _, rule in pairs(ruleset.stars.aspects) do
        validateRule(rule)
    end

    -- Validate planets (ensure at least 1 planet)
    validateRule(ruleset.planets.count)
    if ruleset.planets.count.type == "Weighted" then
        local hasNonZero = false
        for _, v in ipairs(ruleset.planets.count.values) do
            if v.value > 0 then
                hasNonZero = true
                break
            end
        end
        if not hasNonZero then
            Log.Error("Planet count rule allows zero planets")
            error("Invalid rule: planet count must allow non-zero values")
        end
    end
    for _, rule in pairs(ruleset.planets.aspects) do
        validateRule(rule)
    end

    -- Validate moons
    validateRule(ruleset.moons.count)
    for _, rule in pairs(ruleset.moons.aspects) do
        validateRule(rule)
    end

    -- Validate rings
    validateRule(ruleset.rings.count)
    for _, rule in pairs(ruleset.rings.aspects) do
        validateRule(rule)
    end

    -- Validate asteroidBelts
    validateRule(ruleset.asteroidBelts.count)
    for _, rule in pairs(ruleset.asteroidBelts.aspects) do
        validateRule(rule)
    end

    -- Validate starZoneRadius
    validateRule(ruleset.starZoneRadius)
end

return RulesetValidator
