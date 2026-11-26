---Approach spec definitions: where a ship stops at a target of each kind.
---Registers into ApproachSpecRegistry. The kind each is the default of is in Shared/Helpers/ApproachSpecs.lua.
---Loaded from Config/General/GameConfig.lua during config load.

local ApproachSpec = require("Shared.Types.ApproachSpec")
local NavConfig    = require("Config.Navigation.NavConfig")

local Kind = Enums.ApproachKind

-- A station is approached at a point this many of its radii in front of its nose.
local STATION_APPROACH_RADII = 1.5

ApproachSpec {
    id    = Enums.ApproachSpec.PlanetStandOff,
    name  = "PlanetStandOff",
    kind  = Kind.StandOff,
    radii = NavConfig.standOffRadii,
}

ApproachSpec {
    id    = Enums.ApproachSpec.MoonStandOff,
    name  = "MoonStandOff",
    kind  = Kind.StandOff,
    radii = NavConfig.standOffRadii,
}

ApproachSpec {
    id    = Enums.ApproachSpec.AsteroidStandOff,
    name  = "AsteroidStandOff",
    kind  = Kind.StandOff,
    radii = NavConfig.standOffRadii,
}

ApproachSpec {
    id          = Enums.ApproachSpec.StationApproach,
    name        = "StationApproach",
    kind        = Kind.Point,
    offsetRadii = { right = 0, up = 0, forward = STATION_APPROACH_RADII },
}
