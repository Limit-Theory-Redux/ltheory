-- Types --
local LaserProfileDefinition = require("Shared.Types.LaserProfileDefinition")

-- Wavelengths are visible-light bands. Photon energies use E[eV] =
-- 1239.841984 / wavelength[nm]. Strength is a data-defined optical power
-- multiplier relative to the red profile; it also scales beam DPS.
LaserProfileDefinition {
    id = Enums.Weapon.LaserProfile.Red,
    name = "Visible Red 650 nm",
    family = Enums.Weapon.LaserFamily.Red,
    wavelengthNm = 650,
    photonEnergyEv = 1.90745,
    powerWatts = 1800,
    strength = 1.00,
    baseDamagePerSecond = 18,
    presentation = {
        kind = Enums.Weapon.Visual.Beam,
        bodyColor = Color(2.2, 0.04, 0.02, 1.0),
        lightColor = Color(4.0, 0.08, 0.04, 1.0),
        beamWidth = 0.008,
        lightRadius = 0.0,
        lightIntensity = 0.18,
        firingLight = {
            color = Color(1.0, 0.03, 0.02, 1.0),
            radius = 0.0,
            intensity = 0.06,
            muzzleOffset = 0.12,
            duration = 0.10,
            fadeOutDuration = 0.10,
        },
    },
}

LaserProfileDefinition {
    id = Enums.Weapon.LaserProfile.Green,
    name = "Visible Green 532 nm",
    family = Enums.Weapon.LaserFamily.Green,
    wavelengthNm = 532,
    photonEnergyEv = 2.33053,
    powerWatts = 1944,
    strength = 1.08,
    baseDamagePerSecond = 18,
    presentation = {
        kind = Enums.Weapon.Visual.Beam,
        bodyColor = Color(0.03, 2.2, 0.04, 1.0),
        lightColor = Color(0.06, 4.2, 0.08, 1.0),
        beamWidth = 0.0075,
        lightRadius = 0.0,
        lightIntensity = 0.19,
        firingLight = {
            color = Color(0.03, 1.0, 0.04, 1.0),
            radius = 0.0,
            intensity = 0.06,
            muzzleOffset = 0.12,
            duration = 0.10,
            fadeOutDuration = 0.10,
        },
    },
}

LaserProfileDefinition {
    id = Enums.Weapon.LaserProfile.Blue,
    name = "Visible Blue 450 nm",
    family = Enums.Weapon.LaserFamily.Blue,
    wavelengthNm = 450,
    photonEnergyEv = 2.75520,
    powerWatts = 2070,
    strength = 1.15,
    baseDamagePerSecond = 18,
    presentation = {
        kind = Enums.Weapon.Visual.Beam,
        bodyColor = Color(0.02, 0.20, 2.3, 1.0),
        lightColor = Color(0.04, 0.40, 4.4, 1.0),
        beamWidth = 0.007,
        lightRadius = 0.0,
        lightIntensity = 0.20,
        firingLight = {
            color = Color(0.02, 0.15, 1.0, 1.0),
            radius = 0.0,
            intensity = 0.07,
            muzzleOffset = 0.12,
            duration = 0.10,
            fadeOutDuration = 0.10,
        },
    },
}

LaserProfileDefinition {
    id = Enums.Weapon.LaserProfile.Violet,
    name = "Visible Violet 405 nm",
    family = Enums.Weapon.LaserFamily.Violet,
    wavelengthNm = 405,
    photonEnergyEv = 3.06134,
    powerWatts = 2160,
    strength = 1.20,
    baseDamagePerSecond = 18,
    presentation = {
        kind = Enums.Weapon.Visual.Beam,
        bodyColor = Color(0.45, 0.04, 2.5, 1.0),
        lightColor = Color(0.90, 0.08, 4.8, 1.0),
        beamWidth = 0.0065,
        lightRadius = 0.0,
        lightIntensity = 0.21,
        firingLight = {
            color = Color(0.20, 0.03, 1.0, 1.0),
            radius = 0.0,
            intensity = 0.07,
            muzzleOffset = 0.12,
            duration = 0.10,
            fadeOutDuration = 0.10,
        },
    },
}
