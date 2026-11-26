---Cost profile definitions: how a ship weighs the zones it could cross. A weight multiplies a
---zone's path cost; a zone tag that a profile doesn't name weighs one.
---Registers into CostProfileRegistry. Needs the tags of Shared/Definitions/ZoneTagDefs.lua.
---Loaded from Config/General/GameConfig.lua during config load.

local CostProfile = require("Shared.Types.CostProfile")

-- A weight that makes a zone ten times as dear to cross, and one that makes it free.
local DEARER = 10
local FREE   = 0

-- Takes every zone at its authored cost.
CostProfile {
    id      = Enums.CostProfile.Standard,
    name    = "Standard",
    weights = {},
}

-- Goes round belts and hostile space where it can.
CostProfile {
    id      = Enums.CostProfile.Cautious,
    name    = "Cautious",
    weights = { AsteroidField = DEARER, HostileSpace = DEARER },
}

-- Crosses belts and hostile space as if they weren't there.
CostProfile {
    id      = Enums.CostProfile.Bold,
    name    = "Bold",
    weights = { AsteroidField = FREE, HostileSpace = FREE },
}
