local ShipWeaponRegistry = require("Shared.Registries.ShipWeaponRegistry")

Config.weapons = Config.weapons or {}

ShipWeaponRegistry:new("debugPulseTurret", {
    -- The weapon testbed deliberately uses a four-times enlarged capital hull.
    projectileSpeed = 0.72,
    range = 6.5,
    damage = 25,
    projectileLifetime = 10.0,
    cooldown = 0.75,
    interShotGap = 0.12,
    traverseRate = math.rad(120),
    aimTolerance = math.rad(2.0),
    turretScale = 0.016,
    projectileScale = 0.006,
    pulseHeadSize = 0.040,
    pulseTailWidth = 0.015,
    pulseTailLength = 0.35,

    -- AI policy belongs to the weapon/loadout definition so a ship can
    -- select different behavior without hardcoding it in WeaponSystem.
    ai = {
        activeByDefault = false,
        targetRange = 6.5,
        defaultMode = "sequence",
        modeBySizeClass = {
            small = "sequence",
            medium = "sequence",
            large = "volley",
            capital = "volley",
        },
    },
})

-- Scenario/lifecycle settings are kept separate from weapon ballistics.
-- The weapon definition above remains the source of cooldown and projectile
-- behavior; this table controls the isolated testbed's target lifecycle.
Config.weapons.testbed = {
    targetMaxHealth = 10000,
    targetRespawnDelay = 3.0,
    targetSizeClass = "capital",
    targetScaleMultiplier = 1.0,
    aiActive = false,
}
