---The mining laser: the beam a miner plays on an asteroid's ore. A beam effect for `BeamEntity`'s
---drawing only; it does no damage.
local MiningLaser = {
    effect = {
        kind = Enums.Weapon.Effect.Beam,
        visual = {
            bodyColor = Color(1.3, 0.6, 0.12, 1.0),
            lightColor = Color(1.6, 0.8, 0.2, 1.0),
            beamWidth = 0.0012,
            lightRadius = 0.004,
            lightIntensity = 0.15,
        },
    },
}

return MiningLaser
