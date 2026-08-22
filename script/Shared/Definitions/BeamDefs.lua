-- Types --
local BeamDefinition = require("Shared.Types.BeamDefinition")
local LaserProfileRegistry = require("Shared.Registries.LaserProfileRegistry")

local redProfile = LaserProfileRegistry:get(Enums.Weapon.LaserProfile.Red)
assert(redProfile, "red laser profile must be registered before the base beam")

BeamDefinition {
    id = Enums.Weapon.Beam.Laser,
    name = "Laser Beam",
    tickInterval = 0.08,
    sway = {
        -- Angular offsets are applied in the beam's target-relative basis.
        amplitude = math.rad(0.08),
        frequency = 0.8,
        secondaryFrequency = 0.53,
        secondaryAmplitude = 0.45,
    },
    visual = redProfile.presentation,
    -- Beam impact flash at the surface point, tinted by the laser profile.
    impact = {
        hull = {
            color = Color(2.2, 0.4, 0.3, 1.0),
            intensity = 5.0,
            radius = 0.18,
            duration = 0.20,
        },
        shield = {
            color = Color(0.55, 1.4, 2.2, 1.0),
            intensity = 7.0,
            radius = 0.32,
            duration = 0.26,
        },
    },
}
