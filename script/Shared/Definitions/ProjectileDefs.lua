-- Types --
local ProjectileDefinition = require("Shared.Types.ProjectileDefinition")

ProjectileDefinition {
    id = Enums.Weapon.Projectile.Plasma,
    name = "Plasma Bolt",
    speed = 1.80,
    lifetime = 10.0,
    scale = 0.020,
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
        headSize = 0.24,
        tailWidth = 0.12,
        tailLength = 1.19,
    },
}

ProjectileDefinition {
    id = Enums.Weapon.Projectile.LaserBolt,
    name = "Laser Bolt",
    speed = 3.80,
    lifetime = 5.5,
    scale = 0.011,
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
        headSize = 0.11,
        tailWidth = 0.045,
        tailLength = 0.34,
    },
}
