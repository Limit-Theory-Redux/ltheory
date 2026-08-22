-- Types --
local ProjectileDefinition = require("Shared.Types.ProjectileDefinition")
local ScaleConfig = require("Config.Gen.UniverseScaleConfig")

-- Speeds are authored in real m/s and compressed to engine game units once,
-- here (gameunit = meter scaled by globalScale downstream).
local function speed(metersPerSecond)
    return ScaleConfig:toGameUnits(metersPerSecond, "starSystem")
end

-- Impact effect presets, keyed by damage delivery style. Consumed by the
-- impact-effect system when a projectile or beam strikes a target.
--   hullImpact: visual on an unshielded hull surface
--   shieldImpact: visual on an active shield bubble (cooler, wider, shorter)
local function energyHullImpact(color)
    return {
        color = color,
        intensity = 6.0,
        radius = 0.22,
        duration = 0.45,
    }
end

local function kineticHullImpact(color)
    return {
        color = color,
        intensity = 8.0,
        radius = 0.16,
        duration = 0.28,
    }
end

local function shieldImpact()
    return {
        -- Cyan-white arc: reads as energy dispersing over a barrier.
        color = Color(0.55, 1.4, 2.2, 1.0),
        intensity = 9.0,
        radius = 0.38,
        duration = 0.55,
    }
end

ProjectileDefinition {
    id = Enums.Weapon.Projectile.Plasma,
    name = "Plasma Bolt",
    speed = speed(20000.0), -- 20 km/s
    lifetime = 4.0,
    scale = 0.005,
    visual = {
        kind = Enums.Weapon.Visual.Pulse,
        bodyColor = Color(0.3, 0.8, 2.0, 1.0),
        lightColor = Color(0.3, 0.9, 3.0, 1.0),
        lightRadius = 0.0,
        lightIntensity = 0.20,
        dissipationDuration = 0.35,
        firingLight = {
            color = Color(0.02, 0.18, 1.0, 1.0),
            radius = 0.0,
            intensity = 0.06,
            muzzleOffset = 0.12,
            duration = 0.12,
            fadeOutDuration = 0.12,
        },
        headSize = 0.06,
        tailWidth = 0.03,
        tailLength = 0.30,
        impact = {
            hull = energyHullImpact(Color(1.6, 0.9, 2.4, 1.0)),
            shield = shieldImpact(),
        },
    },
}

ProjectileDefinition {
    id = Enums.Weapon.Projectile.LaserBolt,
    name = "Laser Bolt",
    speed = speed(30000.0), -- 30 km/s
    lifetime = 3.0,
    scale = 0.00275,
    archetype = "laser-bolt",
    shaderKey = "laserbolt",
    visual = {
        kind = Enums.Weapon.Visual.Pulse,
        bodyColor = Color(1.75, 0.16, 0.035, 1.0),
        lightColor = Color(2.0, 0.24, 0.05, 1.0),
        lightRadius = 0.0,
        lightIntensity = 0.16,
        dissipationDuration = 0.22,
        firingLight = {
            color = Color(1.0, 0.08, 0.015, 1.0),
            radius = 0.0,
            intensity = 0.05,
            muzzleOffset = 0.10,
            duration = 0.09,
            fadeOutDuration = 0.10,
        },
        headSize = 0.0275,
        tailWidth = 0.01125,
        tailLength = 0.085,
        impact = {
            hull = energyHullImpact(Color(2.2, 0.5, 0.12, 1.0)),
            shield = shieldImpact(),
        },
    },
}

ProjectileDefinition {
    id = Enums.Weapon.Projectile.Flak,
    name = "Flak Round",
    speed = speed(16000.0), -- 16 km/s: fast enough to catch fighters
    lifetime = 3.0,
    scale = 0.0018,
    visual = {
        kind = Enums.Weapon.Visual.Pulse,
        bodyColor = Color(2.4, 1.6, 0.5, 1.0),
        lightColor = Color(3.0, 2.0, 0.7, 1.0),
        lightRadius = 0.0,
        lightIntensity = 0.14,
        dissipationDuration = 0.18,
        firingLight = {
            color = Color(1.0, 0.7, 0.25, 1.0),
            radius = 0.0,
            intensity = 0.05,
            muzzleOffset = 0.08,
            duration = 0.07,
            fadeOutDuration = 0.08,
        },
        headSize = 0.02,
        tailWidth = 0.009,
        tailLength = 0.06,
        impact = {
            hull = kineticHullImpact(Color(2.4, 1.6, 0.5, 1.0)),
            shield = shieldImpact(),
        },
    },
}

ProjectileDefinition {
    id = Enums.Weapon.Projectile.RailSlug,
    name = "Rail Slug",
    speed = speed(45000.0), -- 45 km/s hypervelocity slug
    lifetime = 3.0,
    scale = 0.004,
    archetype = "laser-bolt",
    shaderKey = "laserbolt",
    visual = {
        kind = Enums.Weapon.Visual.Pulse,
        bodyColor = Color(1.6, 1.9, 2.4, 1.0),
        lightColor = Color(1.9, 2.2, 2.8, 1.0),
        lightRadius = 0.0,
        lightIntensity = 0.18,
        dissipationDuration = 0.15,
        firingLight = {
            color = Color(0.8, 0.9, 1.0, 1.0),
            radius = 0.0,
            intensity = 0.06,
            muzzleOffset = 0.10,
            duration = 0.08,
            fadeOutDuration = 0.09,
        },
        headSize = 0.035,
        tailWidth = 0.014,
        tailLength = 0.11,
        impact = {
            hull = kineticHullImpact(Color(1.8, 2.2, 2.9, 1.0)),
            shield = shieldImpact(),
        },
    },
}
