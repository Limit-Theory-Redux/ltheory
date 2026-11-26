---The mining laser: the beam a miner plays on an asteroid's ore. A beam effect for `BeamEntity`'s
---drawing only; it does no damage.
local MiningLaser = {
    effect = {
        kind = Enums.Weapon.Effect.Beam,
        visual = {
            bodyColor = Color(2.4, 1.1, 0.15, 1.0),
            lightColor = Color(4.0, 1.8, 0.3, 1.0),
            beamWidth = 0.003,
            lightRadius = 0.05,
            lightIntensity = 0.6,
        },
    },
}

return MiningLaser
