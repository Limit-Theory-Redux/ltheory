---@class RuleEvaluator
local RuleEvaluator = {}

---@type table<ConditionType, fun(rng: RNG, rule: Rule, context: GenerationContext): any>
local conditionHandlers = {
    [Enums.Gen.Condition.OrbitRadius] = function(rng, rule, context)
        local orbitRadius = context:get("orbitRadius")
        if not orbitRadius then return nil end
        for _, range in ipairs(rule.condition.ranges) do
            if orbitRadius >= range.min and orbitRadius <= range.max then
                if rule.type == Enums.Gen.Rule.Range then
                    return rng:getUniformRange(
                        range.minSize or range.minTemp or range.minGravity or range.minWidth or range.minDensity or range.min,
                        range.maxSize or range.maxTemp or range.maxGravity or range.maxWidth or range.maxDensity or range.max
                    )
                elseif rule.type == Enums.Gen.Rule.Weighted then
                    return RuleEvaluator.evaluateWeighted(rng, rule.values, range.weights)
                end
            end
        end
        return nil
    end,
    [Enums.Gen.Condition.PlanetType] = function(rng, rule, context)
        local planetType = context:get("planetType")
        if not planetType then return nil end
        local typeRules = rule.condition.types[planetType]
        if typeRules then
            if rule.type == Enums.Gen.Rule.Chance then
                return planetType == "Desert" and false or rng:getUniform() < rule.value
            elseif rule.type == Enums.Gen.Rule.Weighted then
                return RuleEvaluator.evaluateWeighted(rng, rule.values, typeRules.weights)
            elseif rule.type == Enums.Gen.Rule.Range then
                return rng:getUniformRange(typeRules.min, typeRules.max)
            end
        end
        return nil
    end,
    [Enums.Gen.Condition.PlanetSize] = function(rng, rule, context)
        local planetSize = context:get("planetSize")
        if not planetSize then return nil end
        for _, range in ipairs(rule.condition.ranges) do
            if planetSize >= range.min and planetSize <= range.max then
                return rng:getUniformRange(range.minGravity or range.minDistance, range.maxGravity or range.maxDistance)
            end
        end
        return nil
    end,
    [Enums.Gen.Condition.SystemAge] = function(rng, rule, context)
        local systemAge = context:get("systemAge")
        if not systemAge then return nil end
        for _, range in ipairs(rule.condition.ranges) do
            if systemAge >= range.min and systemAge <= range.max then
                if rule.type == Enums.Gen.Rule.Weighted then
                    return RuleEvaluator.evaluateWeighted(rng, rule.values, range.weights)
                elseif rule.type == Enums.Gen.Rule.Fixed then
                    return range.value
                end
            end
        end
        return nil
    end,
    [Enums.Gen.Condition.StarType] = function(rng, rule, context)
        local starType = context:get("starType")
        if not starType then return nil end
        local typeRules = rule.condition.types[starType]
        if typeRules then
            return rng:getUniformRange(typeRules.min, typeRules.max)
        end
        return nil
    end,
    [Enums.Gen.Condition.StarMass] = function(rng, rule, context)
        local starMass = context:get("starMass")
        if not starMass then return nil end
        for _, range in ipairs(rule.condition.ranges) do
            if starMass >= range.min and starMass <= range.max then
                return rng:getUniformRange(range.minLuminosity, range.maxLuminosity)
            end
        end
        return nil
    end,
    [Enums.Gen.Condition.SystemMetallicity] = function(rng, rule, context)
        local systemMetallicity = context:get("systemMetallicity")
        if not systemMetallicity then return nil end
        for _, range in ipairs(rule.condition.ranges) do
            if systemMetallicity >= range.min and systemMetallicity <= range.max then
                return RuleEvaluator.evaluateWeighted(rng, rule.values, range.weights)
            end
        end
        return nil
    end,
    [Enums.Gen.Condition.PlanetCount] = function(rng, rule, context)
        local planetCount = context:get("planetCount")
        if not planetCount then return nil end
        for _, range in ipairs(rule.condition.ranges) do
            if planetCount >= range.min and planetCount <= range.max then
                return rng:getInt(range.min, range.max)
            end
        end
        return nil
    end
}

---@param rng RNG
---@param rule Rule
---@param context GenerationContext
---@return any
function RuleEvaluator.evaluate(rng, rule, context)
    if not rng then
        Log.Error("No RNG provided for rule type: %s", rule.type)
        return rule.default or 0
    end

    if rule.condition then
        local handler = conditionHandlers[rule.condition.type]
        if handler then
            local result = handler(rng, rule, context)
            if result ~= nil then return result end
            Log.Debug("Condition %s not met for rule type: %s", tostring(rule.condition.type), tostring(rule.type))
            return rule.default or (rule.type == Enums.Gen.Rule.Weighted and rule.values[1].value) or rule.value or 0
        end
        Log.Error("Unknown condition type: %s", rule.condition.type)
        return rule.default or 0
    end

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
    elseif rule.type == Enums.Gen.Rule.Weighted then
        return RuleEvaluator.evaluateWeighted(rng, rule.values, nil)
    elseif rule.type == Enums.Gen.Rule.Range then
        return rng:getUniformRange(rule.min or 0, rule.max or 1)
    elseif rule.type == Enums.Gen.Rule.Custom then
        if rule.fn then
            local ok, result = pcall(rule.fn, rng, rule, context)
            if ok then return result end
            Log.Error("Custom rule failed: %s", result)
        end
        return rule.default or 0
    end

    Log.Error("Unknown rule type: %s", rule.type)
    return rule.default or 0
end

---@param rng RNG
---@param values { value: any, weight: number }[]
---@param overrideWeights table<string, number>|nil
---@return any
function RuleEvaluator.evaluateWeighted(rng, values, overrideWeights)
    local totalWeight = 0
    local weights = overrideWeights or {}
    for _, v in ipairs(values) do
        totalWeight = totalWeight + (weights[v.value] or v.weight)
    end
    local r = rng:getUniformRange(0, totalWeight)
    local current = 0
    for _, v in ipairs(values) do
        current = current + (weights[v.value] or v.weight)
        if r <= current then
            return v.value
        end
    end
    Log.Error("Weighted rule fallback to first value")
    return values[1].value
end

return RuleEvaluator
