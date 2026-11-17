require("Shared.Definitions.WeaponDefs")
require("Shared.Definitions.WeaponDefs")

-- Target-priority tuning for weapon AI (X4-style threat scoring).
-- Keyed by weapon combatRole; values are per-contact-sizeClass multipliers.
-- Higher = more attractive. Consumed by WeaponSystem:chooseBestTarget.
Config.weapons.targetPriorityByRole = {
    [Enums.Weapon.CombatRole.PointDefense] = {
        small = 4.0,
        medium = 2.0,
        large = 0.5,
        capital = 0.25,
    },
    [Enums.Weapon.CombatRole.Line] = {
        small = 1.0,
        medium = 1.5,
        large = 2.0,
        capital = 2.5,
    },
    [Enums.Weapon.CombatRole.Heavy] = {
        small = 0.75,
        medium = 1.5,
        large = 2.0,
        capital = 2.5,
    },
    [Enums.Weapon.CombatRole.Missile] = {
        small = 1.0,
        medium = 1.5,
        large = 2.0,
        capital = 2.0,
    },
    [Enums.Weapon.CombatRole.CapitalHeavy] = {
        small = 0.5,
        medium = 1.0,
        large = 2.0,
        capital = 3.0,
    },
}

-- Volumetric death explosions, by the destroyed contact's size class (game
-- units; the impact effects are about 0.2-0.4). WeaponCombatSystem spawns
-- one where a Targetable contact dies.
Config.weapons.deathExplosion = {
    small   = { size = 0.30, duration = 1.6, lightIntensity = 5.0,  lightRadius = 0.35 },
    medium  = { size = 0.55, duration = 2.0, lightIntensity = 8.0,  lightRadius = 0.55 },
    large   = { size = 0.95, duration = 2.4, lightIntensity = 12.0, lightRadius = 0.80 },
    capital = { size = 1.60, duration = 3.0, lightIntensity = 18.0, lightRadius = 1.20 },
}
