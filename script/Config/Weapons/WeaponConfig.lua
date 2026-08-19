require("Shared.Definitions.WeaponDefs")

-- Scenario and installation data only. Weapon identity and behavior live in
-- Shared.Definitions.WeaponDefs.
Config.weapons = Config.weapons or {}
Config.weapons.testbed = {
    loadout = {
        { mountId = "fore_outer_port", weaponId = Enums.Weapon.Type.Laser },
        { mountId = "fore_outer_starboard", weaponId = Enums.Weapon.Type.Plasma },
        { mountId = "fore_inner_port", weaponId = Enums.Weapon.Type.Plasma },
        { mountId = "fore_inner_starboard", weaponId = Enums.Weapon.Type.LaserGreen },
        { mountId = "mid_port", weaponId = Enums.Weapon.Type.Plasma },
        { mountId = "mid_starboard", weaponId = Enums.Weapon.Type.LaserBlue },
        { mountId = "aft_inner_port", weaponId = Enums.Weapon.Type.LaserViolet },
        { mountId = "aft_inner_starboard", weaponId = Enums.Weapon.Type.Plasma },
        { mountId = "aft_outer_port", weaponId = Enums.Weapon.Type.Plasma },
        { mountId = "aft_outer_starboard", weaponId = Enums.Weapon.Type.Laser },
    },
    targetMaxHealth = 10000,
    targetRespawnDelay = 3.0,
    targetSizeClass = "capital",
    targetScaleMultiplier = 1.0,
    deferredLighting = true,
    aiActive = true,
    targetMotion = {
        enabled = true,
        mode = "orbit",
        startMode = 1,
        center = { x = 0, y = 0, z = 0 },
        orbitModes = {
            {
                name = "horizontal-clockwise",
                plane = "xz",
                angularSpeed = 0.52,
                direction = 1,
                phase = 0,
            },
            {
                name = "horizontal-counterclockwise",
                plane = "xz",
                angularSpeed = 0.52,
                direction = -1,
                phase = 0,
            },
            {
                name = "vertical-clockwise",
                plane = "xy",
                angularSpeed = 0.42,
                direction = 1,
                phase = 0,
            },
            {
                name = "tilted-counterclockwise",
                plane = "tilted",
                tilt = math.rad(35),
                angularSpeed = 0.46,
                direction = -1,
                phase = 0,
            },
            {
                name = "fore-aft-vertical",
                plane = "yz",
                angularSpeed = 0.38,
                direction = 1,
                phase = 0,
            },
        },
    },
    targetPoint = {
        motionAmplitude = 0.04,
        motionFrequency = 0.30,
        minFacingDot = 0.15,
    },
    capacitors = {
        banks = {
            {
                groupId = Enums.Weapon.CapacitorGroup.Laser,
                maxCharge = 8.0,
                chargeRate = 2.5,
            },
            {
                groupId = Enums.Weapon.CapacitorGroup.Plasma,
                maxCharge = 5.0,
                chargeRate = 1.5,
            },
        },
        policies = {
            [Enums.Weapon.FireMode.Volley] = {
                id = Enums.Weapon.CapacityPolicy.Burst,
            },
            [Enums.Weapon.FireMode.Sequence] = {
                id = Enums.Weapon.CapacityPolicy.Sustain,
            },
        },
    },
}
