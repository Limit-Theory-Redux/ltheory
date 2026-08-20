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
