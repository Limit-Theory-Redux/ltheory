-- Types --
local WeaponDefinition = require("Shared.Types.WeaponDefinition")

-- Effects and laser profiles --
require("Shared.Definitions.LaserProfileDefs")
require("Shared.Definitions.ProjectileDefs")
require("Shared.Definitions.BeamDefs")

local LaserProfileRegistry = require("Shared.Registries.LaserProfileRegistry")
local ProjectileRegistry = require("Shared.Registries.ProjectileRegistry")
local BeamRegistry = require("Shared.Registries.BeamRegistry")

local function laserDefinition(id, name, profileId)
    local profile = LaserProfileRegistry:get(profileId)
    assert(profile, "missing laser profile: " .. tostring(profileId))
    return {
        id = id,
        name = name,
        category = Enums.Weapon.Category.Turret,
        range = 7.2,
        damage = 0,
        damagePerSecond = profile.damagePerSecond,
        cooldown = 0.08,
        interShotGap = 0,
        capacitorCost = 0.18,
        capacitorGroup = Enums.Weapon.CapacitorGroup.Laser,
        turretScale = 0.010,
        effect = BeamRegistry:get(Enums.Weapon.Beam.Laser),
        laserProfileId = profileId,
        laserProfile = profile,
        accuracy = {
            spread = math.rad(0.12),
            trackingJitter = math.rad(0.06),
        },
        tracking = {
            traverseRate = math.rad(180),
            aimTolerance = math.rad(1.2),
        },
        firePolicy = {
            defaultMode = Enums.Weapon.FireMode.Sequence,
            modeBySizeClass = {
                small = Enums.Weapon.FireMode.Sequence,
                medium = Enums.Weapon.FireMode.Sequence,
                large = Enums.Weapon.FireMode.Volley,
                capital = Enums.Weapon.FireMode.Volley,
            },
        },
        capacityPolicy = {
            [Enums.Weapon.FireMode.Volley] = {
                id = Enums.Weapon.CapacityPolicy.Burst,
            },
            [Enums.Weapon.FireMode.Sequence] = {
                id = Enums.Weapon.CapacityPolicy.Sustain,
            },
        },
        ai = {
            activeByDefault = false,
            targetRange = 7.2,
        },
    }
end

WeaponDefinition(laserDefinition(
    Enums.Weapon.Type.Laser,
    "Laser Turret (650 nm Red)",
    Enums.Weapon.LaserProfile.Red))

WeaponDefinition(laserDefinition(
    Enums.Weapon.Type.LaserGreen,
    "Laser Turret (532 nm Green)",
    Enums.Weapon.LaserProfile.Green))

WeaponDefinition(laserDefinition(
    Enums.Weapon.Type.LaserBlue,
    "Laser Turret (450 nm Blue)",
    Enums.Weapon.LaserProfile.Blue))

WeaponDefinition(laserDefinition(
    Enums.Weapon.Type.LaserViolet,
    "Laser Turret (405 nm Violet)",
    Enums.Weapon.LaserProfile.Violet))

WeaponDefinition {
    id = Enums.Weapon.Type.Plasma,
    name = "Plasma Turret",
    category = Enums.Weapon.Category.Turret,
    range = 6.5,
    damage = 28,
    cooldown = 0.90,
    interShotGap = 0.12,
    capacitorCost = 1.0,
    capacitorGroup = Enums.Weapon.CapacitorGroup.Plasma,
    turretScale = 0.016,
    effect = ProjectileRegistry:get(Enums.Weapon.Projectile.Plasma),
    accuracy = {
        spread = math.rad(0.65),
        trackingJitter = math.rad(0.20),
    },

    tracking = {
        traverseRate = math.rad(120),
        aimTolerance = math.rad(2.0),
    },
    firePolicy = {
        defaultMode = Enums.Weapon.FireMode.Sequence,
        modeBySizeClass = {
            small = Enums.Weapon.FireMode.Sequence,
            medium = Enums.Weapon.FireMode.Sequence,
            large = Enums.Weapon.FireMode.Volley,
            capital = Enums.Weapon.FireMode.Volley,
        },
    },
    capacityPolicy = {
        [Enums.Weapon.FireMode.Volley] = {
            id = Enums.Weapon.CapacityPolicy.Burst,
        },
        [Enums.Weapon.FireMode.Sequence] = {
            id = Enums.Weapon.CapacityPolicy.Sustain,
        },
    },
    ai = {
        activeByDefault = false,
        targetRange = 6.5,
    },
}

WeaponDefinition {
    id = Enums.Weapon.Type.LaserBolt,
    name = "Laser Bolt Turret",
    category = Enums.Weapon.Category.Turret,
    range = 7.4,
    damage = 9,
    cooldown = 0.24,
    interShotGap = 0.05,
    capacitorCost = 0.22,
    capacitorGroup = Enums.Weapon.CapacitorGroup.Laser,
    turretScale = 0.009,
    effect = ProjectileRegistry:get(Enums.Weapon.Projectile.LaserBolt),
    accuracy = {
        spread = math.rad(0.35),
        trackingJitter = math.rad(0.10),
    },
    tracking = {
        traverseRate = math.rad(170),
        aimTolerance = math.rad(1.5),
    },
    firePolicy = {
        defaultMode = Enums.Weapon.FireMode.Sequence,
        modeBySizeClass = {
            small = Enums.Weapon.FireMode.Sequence,
            medium = Enums.Weapon.FireMode.Sequence,
            large = Enums.Weapon.FireMode.Volley,
            capital = Enums.Weapon.FireMode.Volley,
        },
    },
    capacityPolicy = {
        [Enums.Weapon.FireMode.Volley] = {
            id = Enums.Weapon.CapacityPolicy.Burst,
        },
        [Enums.Weapon.FireMode.Sequence] = {
            id = Enums.Weapon.CapacityPolicy.Sustain,
        },
    },
    ai = {
        activeByDefault = false,
        targetRange = 7.4,
    },
}
