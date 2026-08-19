local ProceduralContentIdentity = require("Shared.Content.ProceduralContentIdentity")
local ProceduralCatalog = require("Shared.Content.ProceduralCatalog")
local WeaponRegistry = require("Shared.Registries.WeaponRegistry")

---@class WeaponGenerator
---@overload fun(): WeaponGenerator
local WeaponGenerator = Class("WeaponGenerator", function() end)

local function copyTable(value)
    if rawtype(value) ~= "table" then
        return value
    end
    local result = {}
    for key, entry in pairs(value) do
        result[key] = copyTable(entry)
    end
    return result
end

local function cloneEffect(effect)
    assert(rawtype(effect) == "table", "weapon effect must be a table")
    local result = {
        kind = effect.kind,
        name = effect.name,
        speed = effect.speed,
        lifetime = effect.lifetime,
        scale = effect.scale,
        tickInterval = effect.tickInterval,
        sway = copyTable(effect.sway),
        visual = copyTable(effect.visual),
    }
    for key, value in pairs(effect) do
        if result[key] == nil then
            result[key] = copyTable(value)
        end
    end
    return result
end

local function clamp(value, minimum, maximum)
    return math.max(minimum, math.min(maximum, value))
end

---@param args table
---@return table
function WeaponGenerator:generate(args)
    assert(type(args) == "table")
    assert(args.universeSeed ~= nil and args.contentSeed ~= nil,
        "weapon generation requires universeSeed and contentSeed")
    assert(type(args.family) == "string" and #args.family > 0,
        "weapon generation requires a family")
    assert(type(args.variant) == "string" and #args.variant > 0,
        "weapon generation requires a variant")
    local baseWeapon = WeaponRegistry:get(args.baseWeaponId)
    assert(baseWeapon, "weapon generator base ID is not registered")

    local identity = ProceduralContentIdentity.new({
        domain = "weapon",
        generatorId = "weapon:" .. args.family .. ":" .. args.variant,
        generatorVersion = args.generatorVersion or 2,
        universeSeed = args.universeSeed,
        contentSeed = args.contentSeed,
        ordinal = args.ordinal or 0,
    })
    local existing = ProceduralCatalog:resolve(identity.canonicalKey)
    if existing then
        return existing
    end

    local identityRng = RNG.FromStr(identity.canonicalKey)
    assert(identityRng, "weapon generation could not create an identity RNG")
    local rng = RNG.Create(identityRng:get64())
    assert(rng, "weapon generation could not create a deterministic child RNG")
    local effect = cloneEffect(baseWeapon.effect)
    local visual = effect.visual and copyTable(effect.visual) or nil
    if visual then
        effect.visual = visual
    end
    local weapon = {
        kind = "procedural-weapon",
        schemaVersion = 1,
        name = args.name or (args.variant .. " " .. baseWeapon.name),
        category = baseWeapon.category,
        range = baseWeapon.range,
        damage = baseWeapon.damage,
        damagePerSecond = baseWeapon.damagePerSecond,
        laserProfileId = baseWeapon.laserProfileId,
        laserProfile = copyTable(baseWeapon.laserProfile),
        cooldown = baseWeapon.cooldown,
        interShotGap = baseWeapon.interShotGap,
        capacitorCost = baseWeapon.capacitorCost,
        capacitorGroup = baseWeapon.capacitorGroup,
        turretScale = baseWeapon.turretScale,
        accuracy = copyTable(baseWeapon.accuracy),
        tracking = copyTable(baseWeapon.tracking),
        firePolicy = copyTable(baseWeapon.firePolicy),
        capacityPolicy = copyTable(baseWeapon.capacityPolicy),
        ai = copyTable(baseWeapon.ai),
        effect = effect,
        ref = identity,
        weaponRef = { kind = "procedural", canonicalKey = identity.canonicalKey },
        family = args.family,
        variant = args.variant,
        market = {
            canonicalKey = identity.canonicalKey,
            tier = clamp(math.floor(args.tier or 1), 1, 10),
            value = math.floor(100 + (args.tier or 1) * 180),
        },
    }

    if args.family == "laser" and args.variant == "short-burst" then
        effect.duration = args.duration or rng:getUniformRange(0.06, 0.11)
        effect.burst = {
            count = args.burstCount or rng:getInt(2, 4),
            gap = args.burstGap or rng:getUniformRange(0.035, 0.060),
        }
        weapon.cooldown = args.cooldown or math.max(baseWeapon.cooldown,
            rng:getUniformRange(0.30, 0.42))
        weapon.interShotGap = effect.burst.gap
        weapon.damagePerSecond = baseWeapon.damagePerSecond
    elseif args.family == "laser" and args.variant == "fast-bolt" then
        assert(effect.kind == Enums.Weapon.Effect.Projectile,
            "fast-bolt generation requires a projectile base weapon")
        local multiplier = args.speedMultiplier or rng:getUniformRange(1.18, 1.34)
        local scaleMultiplier = args.scaleMultiplier or rng:getUniformRange(0.72, 0.86)
        effect.speed = effect.speed * multiplier
        effect.scale = effect.scale * scaleMultiplier
        effect.shaderKey = "laserbolt"
        effect.archetype = "laser-bolt"
        if effect.visual then
            effect.visual.headSize = (effect.visual.headSize or 0.11)
                * rng:getUniformRange(0.78, 0.90)
            effect.visual.tailWidth = (effect.visual.tailWidth or 0.045)
                * rng:getUniformRange(0.76, 0.88)
            effect.visual.tailLength = (effect.visual.tailLength or 0.34)
                * rng:getUniformRange(0.78, 0.92)
        end
        effect.burst = {
            count = args.burstCount or 3,
            gap = args.burstGap or rng:getUniformRange(0.030, 0.050),
        }
        weapon.damage = weapon.damage
            * (args.damageMultiplier or rng:getUniformRange(0.72, 0.86))
        weapon.cooldown = args.cooldown
            or baseWeapon.cooldown * rng:getUniformRange(0.72, 0.86)
        weapon.interShotGap = effect.burst.gap
    elseif args.family == "plasma" and args.variant == "fast-small" then
        local multiplier = args.speedMultiplier or rng:getUniformRange(1.9, 2.5)
        local scaleMultiplier = args.scaleMultiplier or rng:getUniformRange(0.45, 0.65)
        effect.speed = effect.speed * multiplier
        effect.scale = effect.scale * scaleMultiplier
        if effect.visual then
            effect.visual.headSize = (effect.visual.headSize or 0.2)
                * rng:getUniformRange(0.66, 0.78)
            effect.visual.tailWidth = (effect.visual.tailWidth or 0.1)
                * rng:getUniformRange(0.64, 0.76)
            effect.visual.tailLength = (effect.visual.tailLength or 1.0)
                * rng:getUniformRange(0.62, 0.74)
        end
        weapon.damage = weapon.damage
            * (args.damageMultiplier or rng:getUniformRange(0.52, 0.64))
        weapon.cooldown = args.cooldown
            or baseWeapon.cooldown * rng:getUniformRange(0.56, 0.68)
        weapon.interShotGap = args.interShotGap
            or baseWeapon.interShotGap * rng:getUniformRange(0.56, 0.68)
    elseif args.family == "missile" then
        effect.guidance = {
            kind = "missile",
            thrust = args.thrust or rng:getUniformRange(2.2, 3.0),
            maximumAcceleration = args.maximumAcceleration
                or rng:getUniformRange(2.2, 3.0),
            maximumTurnRate = args.maximumTurnRate
                or math.rad(rng:getUniformRange(48, 62)),
            maximumSpeed = args.maximumSpeed
                or effect.speed * rng:getUniformRange(1.65, 1.95),
            fuelLifetime = args.fuelLifetime or effect.lifetime,
            proximityRadius = args.proximityRadius or rng:getUniformRange(0.10, 0.16),
        }
        effect.speed = args.launchSpeed
            or effect.speed * rng:getUniformRange(0.62, 0.78)
        effect.scale = args.scale
            or effect.scale * rng:getUniformRange(1.25, 1.45)
        weapon.damage = args.damage
            or baseWeapon.damage * rng:getUniformRange(2.2, 2.8)
        weapon.cooldown = args.cooldown
            or baseWeapon.cooldown * rng:getUniformRange(2.2, 2.9)
        weapon.interShotGap = args.interShotGap
            or baseWeapon.interShotGap * rng:getUniformRange(1.8, 2.3)
    else
        error("unsupported generated weapon family/variant: "
            .. args.family .. "/" .. args.variant)
    end

    effect.generatedRef = identity
    effect.archetype = effect.guidance and "missile" or effect.archetype or effect.kind
    if effect.archetype == "missile" then
        effect.shaderKey = "missile"
    end
    weapon.fingerprint = table.concat({
        tostring(weapon.schemaVersion),
        weapon.family,
        weapon.variant,
        tostring(weapon.damage),
        tostring(effect.speed),
        tostring(effect.scale),
        tostring(effect.guidance and effect.guidance.maximumTurnRate),
    }, "|")
    return ProceduralCatalog:register(identity, weapon)
end

return WeaponGenerator()
