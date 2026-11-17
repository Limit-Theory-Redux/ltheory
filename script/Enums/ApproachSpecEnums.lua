---@alias ApproachSpecId integer An `Enums.ApproachSpec` value
---@alias ApproachKindId integer An `Enums.ApproachKind` value

Enums.ApproachSpec = {
    PlanetStandOff   = 1,
    MoonStandOff     = 2,
    AsteroidStandOff = 3,
    StationApproach  = 4,
}

-- How a ship stops at a target: at a distance from its centre, or at a point fixed in its own axes.
Enums.ApproachKind = {
    StandOff = 1,
    Point    = 2,
}
