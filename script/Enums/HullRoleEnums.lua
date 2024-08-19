-- Hull/Role enumerations
Enums.ShipHulls = {
    Solo      = 1,
    Small     = 2,
    Compact   = 3,
    Medium    = 4,
    Large     = 5,
    VeryLarge = 6,
}

Enums.ShipRoles = {
    None        = 0,
    Combat      = 1,
    Trade       = 2,
    Mining      = 3,
    Exploration = 4,
    Civilian    = 5,
}

Enums.ShipHullRoles = {
    {
        Solo      = 1,
        Small     = 2,
        Compact   = 3,
        Medium    = 4,
        Large     = 5,
        VeryLarge = 6
    },
    {
        Fighter    = 1,
        Corvette   = 2,
        Frigate    = 3,
        Destroyer  = 4,
        Cruiser    = 5,
        Battleship = 6
    },
    {
        Courier       = 1,
        Trader        = 2,
        Merchanter    = 3,
        Freighter     = 4,
        BulkFreighter = 5,
        FreighterMax  = 6
    },
    {
        Miner      = 1,
        Prospector = 2,
        Digger     = 3,
        Driller    = 4,
        Dredger    = 5,
        Excavator  = 6
    },
    {
        Scout     = 1,
        Ranger    = 2,
        Seeker    = 3,
        Explorer  = 4,
        Wayfinder = 5,
        Surveyor  = 6
    },
    {
        Boat         = 1,
        Runabout     = 2,
        CabinCruiser = 3,
        Sloop        = 4,
        Yacht        = 5,
        Liner        = 6
    }
}

---@enum SpaceShipHullType
Enums.StationHulls = {
    Small  = 1,
    Medium = 2,
    Large  = 3,
}

Enums.StationRoles = {
    None     = 0,
    Economic = 1,
    Military = 2,
}

Enums.StationHullRoles = {
    {
        Small  = 1,
        Medium = 2,
        Large  = 3
    },
    {
        Trade  = 1,
        Market = 2,
        Depot  = 3
    },
    {
        Outpost = 1,
        Base    = 2,
        Citadel = 3
    }
}
