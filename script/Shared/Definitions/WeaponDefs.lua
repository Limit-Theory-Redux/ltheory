-- Types --
local WeaponDefinition = require("Shared.Types.WeaponDefinition")
local LaserProfileRegistry = require("Shared.Registries.LaserProfileRegistry")
local ProjectileRegistry = require("Shared.Registries.ProjectileRegistry")
local BeamRegistry = require("Shared.Registries.BeamRegistry")
local ScaleConfig = require("Config.Gen.UniverseScaleConfig")

-- Ranges are authored in real meters and compressed to engine game units
-- once, here (gameunit = meter scaled by globalScale downstream).
local function range(meters)
    return ScaleConfig:toGameUnits(meters, "starSystem")
end

-- Effects and laser profiles --
require("Shared.Definitions.LaserProfileDefs")
require("Shared.Definitions.ProjectileDefs")
require("Shared.Definitions.BeamDefs")

local function laserDefinition(id, name, profileId)
    local profile = LaserProfileRegistry:get(profileId)
    assert(profile, "missing laser profile: " .. tostring(profileId))
    return {
        id = id,
        name = name,
        category = Enums.Weapon.Category.Turret,
        range = range(36000), -- 36 km
        damage = 0,
        damagePerSecond = profile.damagePerSecond,
        cooldown = 0.08,
        interShotGap = 0,
        capacitorCost = 0.18,
        capacitorGroup = Enums.Weapon.CapacitorGroup.Laser,
        turretScale = 0.0005,
        mountSizeClass = Enums.Weapon.MountSizeClass.M,
        combatRole = Enums.Weapon.CombatRole.Line,
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
            targetRange = range(36000),
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
    range = range(30000), -- 30 km
    damage = 28,
    cooldown = 0.90,
    interShotGap = 0.12,
    capacitorCost = 1.0,
    capacitorGroup = Enums.Weapon.CapacitorGroup.Plasma,
    turretScale = 0.00075,
    mountSizeClass = Enums.Weapon.MountSizeClass.L,
    combatRole = Enums.Weapon.CombatRole.Heavy,
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
        targetRange = range(30000),
    },
}

WeaponDefinition {
    id = Enums.Weapon.Type.LaserBolt,
    name = "Laser Bolt Turret",
    category = Enums.Weapon.Category.Turret,
    range = range(35000), -- 35 km
    damage = 9,
    -- Point defense: rapid, near-continuous cycling.
    cooldown = 0.12,
    interShotGap = 0.02,
    capacitorCost = 0.22,
    capacitorGroup = Enums.Weapon.CapacitorGroup.Laser,
    turretScale = 0.00045,
    mountSizeClass = Enums.Weapon.MountSizeClass.SM,
    combatRole = Enums.Weapon.CombatRole.PointDefense,
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
        targetRange = range(35000),
    },
}

WeaponDefinition {
    id = Enums.Weapon.Type.XLLongChargeLaser,
    name = "XL Long-Charge Laser",
    category = Enums.Weapon.Category.Turret,
    range = range(60000), -- 60 km
    damage = 0,
    damagePerSecond = 240,
    cooldown = 2.8,
    interShotGap = 0,
    capacitorCost = 3.5,
    capacitorGroup = Enums.Weapon.CapacitorGroup.Laser,
    turretScale = 0.00125,
    mountSizeClass = Enums.Weapon.MountSizeClass.XL,
    combatRole = Enums.Weapon.CombatRole.CapitalHeavy,
    effect = BeamRegistry:get(Enums.Weapon.Beam.Laser),
    laserProfileId = Enums.Weapon.LaserProfile.Red,
    laserProfile = LaserProfileRegistry:get(Enums.Weapon.LaserProfile.Red),
    accuracy = {
        spread = math.rad(0.08),
        trackingJitter = math.rad(0.03),
    },
    tracking = {
        traverseRate = math.rad(75),
        aimTolerance = math.rad(0.8),
    },
    firePolicy = {
        defaultMode = Enums.Weapon.FireMode.Volley,
    },
    capacityPolicy = {
        [Enums.Weapon.FireMode.Volley] = {
            id = Enums.Weapon.CapacityPolicy.Burst,
        },
    },
    ai = {
        activeByDefault = false,
        targetRange = range(60000),
    },
}

local function missileDefinition(id, name, mountSizeClass, capacitorCost, damage, cooldown)
    return {
        id = id,
        name = name,
        category = Enums.Weapon.Category.Turret,
        range = range(45000), -- 45 km
        damage = damage,
        cooldown = cooldown,
        interShotGap = 0.12,
        capacitorCost = capacitorCost,
        capacitorGroup = Enums.Weapon.CapacitorGroup.Plasma,
        launcherFamilyId = Enums.Weapon.LauncherFamily.Missile,
        turretScale = 0.0009,
        mountSizeClass = mountSizeClass,
        combatRole = Enums.Weapon.CombatRole.Missile,
        effect = ProjectileRegistry:get(Enums.Weapon.Projectile.Plasma),
        accuracy = {
            spread = math.rad(0.45),
            trackingJitter = math.rad(0.12),
        },
        tracking = {
            traverseRate = math.rad(100),
            aimTolerance = math.rad(2.5),
        },
        firePolicy = {
            defaultMode = Enums.Weapon.FireMode.Volley,
        },
        capacityPolicy = {
            [Enums.Weapon.FireMode.Volley] = {
                id = Enums.Weapon.CapacityPolicy.Burst,
            },
        },
        ai = {
            activeByDefault = false,
            targetRange = range(45000),
        },
    }
end

WeaponDefinition(missileDefinition(
    Enums.Weapon.Type.MissileRocket,
    "M Rocket Launcher",
    Enums.Weapon.MountSizeClass.M,
    1.4,
    42,
    1.8))

WeaponDefinition(missileDefinition(
    Enums.Weapon.Type.MissileTorpedo,
    "L Torpedo Launcher",
    Enums.Weapon.MountSizeClass.L,
    2.8,
    88,
    3.4))

WeaponDefinition {
    id = Enums.Weapon.Type.FlakCannon,
    name = "Flak Cannon",
    category = Enums.Weapon.Category.Turret,
    -- Anti-fighter screen: fast cycling + high per-shot damage so the
    -- fighter's small health pool evaporates on contact.
    range = range(28000), -- 28 km
    damage = 14,
    cooldown = 0.20,
    interShotGap = 0.03,
    capacitorCost = 0.30,
    capacitorGroup = Enums.Weapon.CapacitorGroup.Plasma,
    turretScale = 0.0005,
    mountSizeClass = Enums.Weapon.MountSizeClass.SM,
    combatRole = Enums.Weapon.CombatRole.PointDefense,
    effect = ProjectileRegistry:get(Enums.Weapon.Projectile.Flak),
    accuracy = {
        spread = math.rad(0.6),
        trackingJitter = math.rad(0.15),
    },
    tracking = {
        traverseRate = math.rad(220),
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
        targetRange = range(28000),
    },
}

WeaponDefinition {
    id = Enums.Weapon.Type.Railgun,
    name = "Assault Railgun",
    category = Enums.Weapon.Category.Turret,
    -- Anti-medium/hull: heavy hypervelocity slug, moderate cycle.
    range = range(40000), -- 40 km
    damage = 55,
    cooldown = 1.6,
    interShotGap = 0.10,
    capacitorCost = 2.2,
    capacitorGroup = Enums.Weapon.CapacitorGroup.Plasma,
    turretScale = 0.0007,
    mountSizeClass = Enums.Weapon.MountSizeClass.M,
    combatRole = Enums.Weapon.CombatRole.Line,
    effect = ProjectileRegistry:get(Enums.Weapon.Projectile.RailSlug),
    accuracy = {
        spread = math.rad(0.25),
        trackingJitter = math.rad(0.08),
    },
    tracking = {
        traverseRate = math.rad(140),
        aimTolerance = math.rad(1.4),
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
        targetRange = range(40000),
    },
}
