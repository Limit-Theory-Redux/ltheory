local ProceduralContentIdentity = require("Shared.Content.ProceduralContentIdentity")
local ProceduralCatalog = require("Shared.Content.ProceduralCatalog")

---@class TrackingModuleGenerator
---@overload fun(): TrackingModuleGenerator
local TrackingModuleGenerator = Class("TrackingModuleGenerator", function() end)

local function clamp(value, minimum, maximum)
    return math.max(minimum, math.min(maximum, value))
end

local function round(value, places)
    local scale = 10 ^ (places or 6)
    return math.floor(value * scale + 0.5) / scale
end

---@param args table
---@return table
function TrackingModuleGenerator:generate(args)
    assert(type(args) == "table")
    local universeSeed = args.universeSeed
    local contentSeed = args.contentSeed
    assert(universeSeed ~= nil and contentSeed ~= nil,
        "tracking module generation requires universeSeed and contentSeed")
    local generatorId = args.generatorId or "tracking-module"
    local generatorVersion = args.generatorVersion or 2
    local ordinal = args.ordinal or 0
    local identity = ProceduralContentIdentity.new({
        domain = "equipment.tracking",
        generatorId = generatorId,
        generatorVersion = generatorVersion,
        universeSeed = universeSeed,
        contentSeed = contentSeed,
        ordinal = ordinal,
    })
    local existing = ProceduralCatalog:resolve(identity.canonicalKey)
    if existing then
        return existing
    end

    local identityRng = RNG.FromStr(identity.canonicalKey)
    assert(identityRng, "tracking module generation could not create an identity RNG")
    local rng = RNG.Create(identityRng:get64())
    assert(rng, "tracking module generation could not create a deterministic child RNG")
    local tier = clamp(math.floor(args.tier or 1), 1, 10)
    local quality = clamp(
        0.18 + (tier - 1) * 0.075 + rng:getUniform() * 0.12,
        0.05,
        0.98)
    local manufacturerBias = clamp(args.manufacturerBias or 0, -0.20, 0.20)
    local response = clamp(quality + manufacturerBias, 0.05, 1.0)
    local accuracy = clamp(quality * 0.85 + rng:getUniform() * 0.10, 0.05, 1.0)
    local prediction = clamp(quality * 0.80 + rng:getUniform() * 0.15, 0.05, 1.0)
    local stats = {
        sampleRate = round(8.0 + response * 18.0, 5),
        velocityResponse = round(0.42 + response * 0.50, 5),
        accelerationResponse = round(0.16 + response * 0.54, 5),
        predictionHorizon = round(3.0 + prediction * 7.0, 5),
        predictionDamping = round(0.65 - prediction * 0.38, 5),
        turnRateFloor = round(0.0015 - prediction * 0.0010, 6),
        turnRateLimit = round(3.0 + prediction * 9.0, 5),
        traverseRate = round(math.rad(55.0 + response * 105.0), 6),
        aimTolerance = round(math.rad(2.4 - accuracy * 1.65), 6),
        confidenceFloor = round(0.12 - prediction * 0.07, 6),
        measurementError = round(0.35 - accuracy * 0.28, 6),
    }

    local record = {
        kind = "tracking-module",
        schemaVersion = 1,
        tier = tier,
        quality = round(quality, 6),
        name = string.format("Tracking Array T%d-%03d", tier, math.floor(quality * 999)),
        stats = stats,
        market = {
            canonicalKey = identity.canonicalKey,
            tier = tier,
            value = math.floor(80 + tier * 120 + quality * 900),
        },
    }
    record.fingerprint = table.concat({
        tostring(record.schemaVersion),
        tostring(record.tier),
        tostring(record.quality),
        tostring(stats.sampleRate),
        tostring(stats.predictionHorizon),
        tostring(stats.traverseRate),
        tostring(stats.aimTolerance),
    }, "|")
    return ProceduralCatalog:register(identity, record)
end

return TrackingModuleGenerator()
