local ProceduralContentIdentity = require("Shared.Content.ProceduralContentIdentity")
local ProceduralCatalog = require("Shared.Content.ProceduralCatalog")
local WeaponRegistry = require("Shared.Registries.WeaponRegistry")
local LaserProfileRegistry = require("Shared.Registries.LaserProfileRegistry")

local function isEnumValue(enum, value)
    for _, candidate in pairs(enum) do
        if candidate == value then
            return true
        end
    end
    return false
end

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

local IDENTITY_ARGUMENTS = {
    "baseWeaponId",
    "name",
    "tier",
    "mountSizeClass",
    "combatRole",
    "launcherFamilyId",
    "capacitorCost",
    "capacitorGroup",
    "fireMode",
    "cooldown",
    "damagePerSecond",
    "duration",
    "laserProfileId",
    "speedMultiplier",
    "scaleMultiplier",
    "damageMultiplier",
    "burstCount",
    "burstGap",
    "interShotGap",
    "thrust",
    "maximumAcceleration",
    "maximumTurnRate",
    "maximumSpeed",
    "fuelLifetime",
    "proximityRadius",
    "launchSpeed",
    "scale",
    "damage",
}

local function appendIdentityArgument(parts, key, value)
    if value ~= nil then
        local valueType = type(value)
        local text = valueType:sub(1, 1) .. ":" .. tostring(value)
        table.insert(parts, key .. "#" .. tostring(#text) .. "=" .. text)
    end
end

---@param args table
---@return table
function WeaponGenerator:generate(args)
    assert(type(args) == "table")
    assert(args.universeSeed ~= nil and args.contentSeed ~= nil,
        "weapon generation requires universeSeed and contentSeed")
    assert(isEnumValue(Enums.Weapon.Family, args.family),
        "weapon generation requires a known Enums.Weapon.Family value")
    assert(isEnumValue(Enums.Weapon.Variant, args.variant),
        "weapon generation requires a known Enums.Weapon.Variant value")
    local baseWeapon = WeaponRegistry:get(args.baseWeaponId)
    assert(baseWeapon, "weapon generator base ID is not registered")

    local generatorParts = { "weapon" }
    appendIdentityArgument(generatorParts, "family", args.family)
    appendIdentityArgument(generatorParts, "variant", args.variant)
    for _, key in ipairs(IDENTITY_ARGUMENTS) do
        appendIdentityArgument(generatorParts, key, args[key])
    end
    local generatorId = table.concat(generatorParts, ":")

    local identity = ProceduralContentIdentity.new({
        domain = "weapon",
        generatorId = generatorId,
        generatorVersion = args.generatorVersion or 3,
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
        kind = Enums.Weapon.IdentityKind.ProceduralWeapon,
        schemaVersion = 1,
        name = args.name or (args.variant .. " " .. baseWeapon.name),
        category = baseWeapon.category,
        range = baseWeapon.range,
        damage = baseWeapon.damage,
        damagePerSecond = baseWeapon.damagePerSecond,
        laserProfileId = args.laserProfileId or baseWeapon.laserProfileId,
        laserProfile = copyTable(
            args.laserProfileId
                and LaserProfileRegistry:get(args.laserProfileId)
                or baseWeapon.laserProfile),
        cooldown = baseWeapon.cooldown,
        interShotGap = baseWeapon.interShotGap,
        capacitorCost = args.capacitorCost or baseWeapon.capacitorCost,
        capacitorGroup = args.capacitorGroup or baseWeapon.capacitorGroup,
        launcherFamilyId = args.launcherFamilyId or baseWeapon.launcherFamilyId,
        turretScale = baseWeapon.turretScale,
        mountSizeClass = args.mountSizeClass or baseWeapon.mountSizeClass,
        combatRole = args.combatRole or baseWeapon.combatRole,
        accuracy = copyTable(baseWeapon.accuracy),
        tracking = copyTable(baseWeapon.tracking),
        firePolicy = copyTable(baseWeapon.firePolicy),
        capacityPolicy = copyTable(baseWeapon.capacityPolicy),
        ai = copyTable(baseWeapon.ai),
        effect = effect,
        ref = identity,
        weaponRef = {
            kind = Enums.Weapon.IdentityKind.Procedural,
            canonicalKey = identity.canonicalKey,
        },
        family = args.family,
        variant = args.variant,
        market = {
            canonicalKey = identity.canonicalKey,
            tier = clamp(math.floor(args.tier or 1), 1, 10),
            value = math.floor(100 + (args.tier or 1) * 180),
        },
    }

    if args.fireMode then
        assert(args.fireMode == Enums.Weapon.FireMode.Volley
            or args.fireMode == Enums.Weapon.FireMode.Sequence,
            "generated weapon fireMode must be a known enum value")
        weapon.firePolicy.defaultMode = args.fireMode
    end

    if args.family == Enums.Weapon.Family.Laser and args.variant == Enums.Weapon.Variant.ShortBurst then
        effect.duration = args.duration or rng:getUniformRange(0.06, 0.11)
        effect.burst = {
            count = args.burstCount or rng:getInt(2, 4),
            gap = args.burstGap or rng:getUniformRange(0.035, 0.060),
        }
        weapon.cooldown = args.cooldown or math.max(baseWeapon.cooldown,
            rng:getUniformRange(0.30, 0.42))
        weapon.interShotGap = effect.burst.gap
        weapon.damagePerSecond = baseWeapon.damagePerSecond
    elseif args.family == Enums.Weapon.Family.Laser and args.variant == Enums.Weapon.Variant.LongCharge then
        assert(effect.kind == Enums.Weapon.Effect.Beam,
            "long-charge generation requires a beam base weapon")
        weapon.mountSizeClass = args.mountSizeClass or Enums.Weapon.MountSizeClass.XL
        weapon.combatRole = args.combatRole or Enums.Weapon.CombatRole.CapitalHeavy
        weapon.capacitorCost = args.capacitorCost or math.max(baseWeapon.capacitorCost or 0, 3.5)
        weapon.cooldown = args.cooldown or math.max(baseWeapon.cooldown or 0, 2.8)
        weapon.damagePerSecond = args.damagePerSecond or baseWeapon.damagePerSecond
        weapon.firePolicy.defaultMode = args.fireMode or Enums.Weapon.FireMode.Volley
        effect.duration = args.duration or rng:getUniformRange(0.65, 1.05)
        -- Scale thickness with the weapon's size class so the capital-grade
        -- laser reads as a heavy beam, not a hairline. BeamEntity renders the
        -- LASER PROFILE presentation when one is attached, so scale that too;
        -- effect.visual only governs non-profile beams.
        local widthScale = args.beamWidthScale
            or (weapon.mountSizeClass == Enums.Weapon.MountSizeClass.XL and 4.0
                or weapon.mountSizeClass == Enums.Weapon.MountSizeClass.L and 2.5
                or 1.0)
        if widthScale ~= 1.0 then
            local presentation = weapon.laserProfile
                and weapon.laserProfile.presentation
            if presentation and presentation.beamWidth then
                presentation.beamWidth = presentation.beamWidth * widthScale
            end
            if effect.visual and effect.visual.beamWidth then
                effect.visual.beamWidth = effect.visual.beamWidth * widthScale
            end
        end
    elseif args.family == Enums.Weapon.Family.Laser and args.variant == Enums.Weapon.Variant.FastBolt then
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
    elseif args.family == Enums.Weapon.Family.Plasma and args.variant == Enums.Weapon.Variant.FastSmall then
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
    elseif args.family == Enums.Weapon.Family.Missile then
        local isRocket = args.variant == Enums.Weapon.Variant.Rocket
        local isTorpedo = args.variant == Enums.Weapon.Variant.Torpedo
        assert(args.variant == Enums.Weapon.Variant.Guided or isRocket or isTorpedo,
            "unsupported generated missile variant: " .. tostring(args.variant))
        if isRocket then
            weapon.mountSizeClass = args.mountSizeClass or Enums.Weapon.MountSizeClass.M
            weapon.capacitorCost = args.capacitorCost or 1.4
        elseif isTorpedo then
            weapon.mountSizeClass = args.mountSizeClass or Enums.Weapon.MountSizeClass.L
            weapon.capacitorCost = args.capacitorCost or 2.8
        end
        weapon.launcherFamilyId = args.launcherFamilyId
            or Enums.Weapon.LauncherFamily.Missile
        -- Distinct exhaust look: missiles burn hot orange-white with a long
        -- smoky trail instead of inheriting the plasma bolt's blue glow.
        if effect.visual then
            effect.visual.bodyColor = Color(2.4, 1.1, 0.35, 1.0)
            effect.visual.lightColor = Color(3.0, 1.3, 0.45, 1.0)
            effect.visual.lightIntensity = 0.30
            if isTorpedo then
                -- Heavier, slower-burning plume.
                effect.visual.headSize = (effect.visual.headSize or 0.24) * 1.5
                effect.visual.tailWidth = (effect.visual.tailWidth or 0.12) * 1.6
                effect.visual.tailLength = (effect.visual.tailLength or 1.19) * 1.8
            else
                -- Rockets/guided: slim, fast exhaust.
                effect.visual.headSize = (effect.visual.headSize or 0.24) * 0.9
                effect.visual.tailWidth = (effect.visual.tailWidth or 0.12) * 0.7
                effect.visual.tailLength = (effect.visual.tailLength or 1.19) * 1.25
            end
        end
        local launchSpeedMultiplier = isRocket and 0.82
            or (isTorpedo and 0.48 or 0.70)
        local scaleMultiplier = isRocket and 1.18
            or (isTorpedo and 1.75 or 1.35)
        local damageMultiplier = isRocket and 2.2
            or (isTorpedo and 3.4 or 2.5)
        local cooldownMultiplier = isRocket and 2.1
            or (isTorpedo and 3.5 or 2.5)
        effect.guidance = {
            kind = "missile",
            thrust = args.thrust or (isRocket and 3.0
                or (isTorpedo and 1.9 or rng:getUniformRange(2.2, 3.0))),
            maximumAcceleration = args.maximumAcceleration
                or (isRocket and 3.4
                    or (isTorpedo and 2.0 or rng:getUniformRange(2.2, 3.0))),
            maximumTurnRate = args.maximumTurnRate
                or math.rad(isRocket and 62 or (isTorpedo and 38
                    or rng:getUniformRange(48, 62))),
            maximumSpeed = args.maximumSpeed
                or effect.speed * (isRocket and 1.85
                    or (isTorpedo and 1.35 or rng:getUniformRange(1.65, 1.95))),
            fuelLifetime = args.fuelLifetime or effect.lifetime,
            proximityRadius = args.proximityRadius or (isTorpedo and 0.22
                or rng:getUniformRange(0.10, 0.16)),
        }
        effect.speed = args.launchSpeed
            or effect.speed * launchSpeedMultiplier
        effect.scale = args.scale
            or effect.scale * scaleMultiplier
        weapon.damage = args.damage
            or baseWeapon.damage * damageMultiplier
        weapon.cooldown = args.cooldown
            or baseWeapon.cooldown * cooldownMultiplier
        weapon.interShotGap = args.interShotGap
            or baseWeapon.interShotGap * (isTorpedo and 2.8 or 2.0)
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
        tostring(weapon.mountSizeClass),
        tostring(weapon.combatRole),
        tostring(weapon.launcherFamilyId),
        tostring(weapon.capacitorCost),
        tostring(weapon.firePolicy and weapon.firePolicy.defaultMode),
        tostring(weapon.damage),
        tostring(effect.speed),
        tostring(effect.scale),
        tostring(effect.guidance and effect.guidance.maximumTurnRate),
    }, "|")
    return ProceduralCatalog:register(identity, weapon)
end

return WeaponGenerator()
