---@class RuleEvaluator
local RuleEvaluator = {}

---@type table<integer, fun(rng: RNG, rule: Rule, context: GenerationContext): any>
---@type table<integer, fun(rng: RNG, rule: Rule, context: GenerationContext): any>
local conditionHandlers = {
    [Enums.Gen.Condition.OrbitRadius] = function(rng, rule, context)
        local orbitRadius = context:get("orbitRadius")
        if not orbitRadius then return nil end
        for _, range in ipairs(rule.condition.ranges) do
            if orbitRadius >= range.min and orbitRadius <= range.max then
                if rule.type == Enums.Gen.Rule.Range then
                    return rng:getUniformRange(
                        range.minSize or range.minTemp or range.minGravity or range.minWidth or range.minDensity or range.minDistance or
                        range.min,
                        range.maxSize or range.maxTemp or range.maxGravity or range.maxWidth or range.maxDensity or range.maxDistance or
                        range.max
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
        --Log.Debug("Evaluating PlanetType condition with planetType: %s, rule condition: %s", tostring(planetType), table.tostring(rule.condition))
        if planetType and rule.condition.types[planetType] then
            local typeConfig = rule.condition.types[planetType]
            if rule.type == Enums.Gen.Rule.Weighted then
                local weights = typeConfig.weights or {}
                local totalWeight = 0
                for _, weight in pairs(weights) do
                    totalWeight = totalWeight + weight
                end
                if totalWeight == 0 then
                    Log.Warn("No valid weights for PlanetType %s, falling back to rule.values", tostring(planetType))
                    return RuleEvaluator.evaluateWeighted(rng, rule.values, rule.default)
                end
                local randomValue = rng:getUniformRange(0, totalWeight)
                local cumulativeWeight = 0
                for value, weight in pairs(weights) do
                    cumulativeWeight = cumulativeWeight + weight
                    if randomValue <= cumulativeWeight then
                        return value
                    end
                end
            end
            return typeConfig.value or
                (typeConfig.weights and next(typeConfig.weights) and typeConfig.weights[next(typeConfig.weights)].value)
        end
        Log.Warn("PlanetType %s not found in condition, falling back to rule.values or default", tostring(planetType))
        if rule.values and #rule.values > 0 then
            return RuleEvaluator.evaluateWeighted(rng, rule.values, rule.default)
        end
        return rule.default or nil
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
    [Enums.Gen.Condition.StarCount] = function(rng, rule, context)
        local starCount = context:get("starCount")
        if not starCount then
            Log.Error("StarCount condition requires starCount in context")
            return rule.default or Enums.Gen.StarSystemTypes.Single
        end
        for _, range in ipairs(rule.condition.ranges) do
            if starCount >= range.min and starCount <= range.max then
                if rule.type == Enums.Gen.Rule.Fixed then
                    return range.value
                end
            end
        end
        Log.Debug("StarCount condition not met, using default")
        return rule.default or Enums.Gen.StarSystemTypes.Single
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
    [Enums.Gen.Condition.StarIndex] = function(rng, rule, context)
        local starIndex = context:get("starIndex")
        if not starIndex then
            Log.Error("StarIndex condition requires starIndex in context")
            return rule.default or Position(0, 0, 0)
        end
        for _, range in ipairs(rule.condition.ranges) do
            if starIndex >= range.min and starIndex <= range.max then
                return range.value
            end
        end
        return rule.default or Position(0, 0, 0)
    end,
    [Enums.Gen.Condition.SystemMetallicity] = function(rng, rule, context)
        local systemMetallicity = context:get("systemMetallicity") or 0.01
        for _, range in ipairs(rule.condition.ranges) do
            if systemMetallicity >= range.min and systemMetallicity <= range.max then
                local weights = {}
                for _, v in ipairs(rule.values) do
                    local key = type(v.value) == "table" and v.value.type or v.value
                    local adjustedWeight = v.weight * (range.weights[key] or 1.0)
                    table.insert(weights, { value = v.value, weight = adjustedWeight })
                end
                return RuleEvaluator.evaluateWeighted(rng, weights, nil)
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
    if not rule or not rule.type then
        Log.Error("Invalid rule")
        return nil
    end

    if rule.condition then
        local handler = conditionHandlers[rule.condition.type]
        if handler then
            local result = handler(rng, rule, context)
            if result ~= nil then return result end
            Log.Debug("Condition %s not met for rule type: %s", tostring(rule.condition.type), tostring(rule.type))
            return rule.default or (rule.type == Enums.Gen.Rule.Weighted and rule.values[1].value) or rule.value or 0
        end
        Log.Error("Unknown condition type: %s", tostring(rule.condition.type))
        return rule.default or 0
    end

    if rule.type == Enums.Gen.Rule.Fixed then
        return rule.value
    elseif rule.type == Enums.Gen.Rule.Range then
        return rng:getUniformRange(rule.min or 0, rule.max or 1)
    elseif rule.type == Enums.Gen.Rule.Weighted then
        return RuleEvaluator.evaluateWeighted(rng, rule.values, nil)
    elseif rule.type == Enums.Gen.Rule.Chance then
        if rule.condition then
            return conditionHandlers[rule.condition.type](rng, rule, context)
        else
            return rng:getUniformRange(0, 1) < (rule.value or 0.5)
        end
    elseif rule.type == Enums.Gen.Rule.Count then
        local min = rule.min or 0
        local max = rule.max or 0
        if min > max then min, max = max, min end
        return rng:getInt(min, max)
    elseif rule.type == Enums.Gen.Rule.Custom then
        if rule.fn then
            local ok, result = pcall(rule.fn, rng, rule, context)
            if ok then return result end
            Log.Error("Custom rule failed: %s", result)
        end
        return rule.default or 0
    end

    Log.Error("Unknown rule type: %s", tostring(rule.type))
    return rule.default or 0
end

---@param rng RNG
---@param values { value: any, weight: number }[]
---@param overrideWeights table<integer, number>|nil
---@return any
function RuleEvaluator.evaluateWeighted(rng, values, overrideWeights)
    if not values or #values == 0 then
        Log.Error("Weighted rule has no values")
        return nil
    end

    local totalWeight = 0
    for _, v in ipairs(values) do
        local key = type(v.value) == "table" and v.value.type or v.value
        totalWeight = totalWeight + (overrideWeights and overrideWeights[key] or v.weight)
    end
    if totalWeight == 0 then
        Log.Error("Weighted rule has zero total weight")
        return values[1].value
    end
    local r = rng:getUniformRange(0, totalWeight)
    local current = 0
    for _, v in ipairs(values) do
        local key = type(v.value) == "table" and v.value.type or v.value
        current = current + (overrideWeights and overrideWeights[key] or v.weight)
        if r <= current then
            return v.value
        end
    end
    Log.Error("Weighted rule fallback to first value")
    return values[1].value
end

return RuleEvaluator
