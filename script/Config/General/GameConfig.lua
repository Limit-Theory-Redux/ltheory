--! this can be split up better
require("Shared.Definitions.FlightProfileDefs")

Config.game = {
    boostCost              = 20,

    explosionSize          = 64,

    autoTarget             = false,

    droneType              = 1, -- 1 = mining drone, 2 = combat drone
    droneDamage            = 10,
    droneTarget            = nil,
    droneSize              = 75,
    droneSpeed             = 50,
    droneRange             = 5000,

    missileDamage          = 80,
    missileTarget          = nil,
    missileSize            = 100,
    missileSpeed           = 100,
    missileRange           = 10000,

    torpedoDamage          = 250,
    torpedoTarget          = nil,
    torpedoSize            = 64,
    torpedoSpeed           = 30,
    torpedoRange           = 14000,

    weaponGroup            = 1,

    shieldColor            = Color(0.2, 0.5, 1.0, 1.0),

    enemies                = 0,
    friendlies             = 0,
    squadSizeEnemy         = 8,
    squadSizeFriendly      = 8,

    spawnDistance          = 2000,
    friendlySpawnCount     = 10,
    timeScaleShipEditor    = 0.0,

    aiFire                 = function(dt, rng) return rng:getExp() ^ 2 < dt end,

    autonavRanges          = { 200, -- Unknown
        0,                          -- Reserved
        0,                          -- Star Sector
        0,                          -- Star System
        2000,                       -- Zone
        1e7,                        -- Star (TODO: radius + offset)
        10000,                      -- Planet (TODO: radius + offset)
        300,                        -- Asteroid
        500,                        -- Jumpgate
        2000,                       -- Station
        100 },                      -- Ship

    dockRange              = 50,

    -- Ship flight limits are per-ship data: Shared/Definitions/FlightProfileDefs.lua.
    -- The player's input settings:
    playerFlight = {
        invertYaw   = false,
        invertPitch = false,
    },

    -- Physics tuning for ship-scale contacts, set by apps with ship flight.
    physics = (function()
        local ScaleConfig = require("Config.Gen.UniverseScaleConfig")

        -- m: the physics engine scales its tolerances by this; about a ship's size
        local lengthUnitReal = 100
        -- s: a ship's soft-CCD distance covers this much travel at its speed cap
        local ccdFrameTime = 1 / 60

        return {
            lengthUnit              = ScaleConfig:toGameUnits(lengthUnitReal, "starSystem"),
            contactNaturalFrequency = 1920, -- Hz

            ---Soft-CCD prediction distance for a body whose speed is capped at `speedCap`.
            ---@param speedCap number gameunits/s
            ---@return number gameunits
            softCcdDistance = function(speedCap) return speedCap * ccdFrameTime end,
        }
    end)(),

    -- Ship hull geometry (from legacy, indexed by Enums.ShipHulls)
    -- Solo=1, Small=2, Compact=3, Medium=4, Large=5, VeryLarge=6
    shipHulls = {
        -- Scaled for globalScale 0.0001 (1 game unit = 10km)
        -- Solo ~50m, Small ~100m, Compact ~150m, Medium ~200m, Large ~300m, VeryLarge ~500m
        scale  = { 0.005, 0.01, 0.015, 0.02, 0.03, 0.05 },
        radius = { 0.003, 0.005, 0.008, 0.012, 0.018, 0.025 },
    },

    travelDrive = {
        chargeRequired  = 3.0,     -- seconds holding T to activate
        speedMultiplier = 40000,   -- adjusted for higher base thrust
        rampUpSpeed     = 0.3,
        rampDownSpeed   = 1.5,

        -- Max speed multiplier per zone type
        maxSpeedByZone = {
            moon      = 200,
            planet    = 2000,
            star      = 8000,
            openSpace = 40000,
        },
    },

    autoPilot = {
        arrivalRange      = 500,    -- distance to target to consider arrived
        travelDriveDelay  = 5.0,    -- seconds before auto-engaging travel drive
        enableTravelDrive = true,   -- auto-engage travel drive during autopilot
    },

    chaseCamera = {
        minRadius      = 0.5,    -- Min zoom (multiplier on ship radius)
        maxRadius      = 8.0,    -- Max zoom
        defaultRadius  = 1.0,    -- Default zoom
        zoomSpeed      = 0.15,   -- Zoom per scroll tick
    },

    -- Orbit camera (chase view of the player ship)
    orbitCamera = {
        distance     = 0.015,    -- Default follow distance
        minDistance  = 0.005,
        maxDistance  = 5.0,
        smoothing    = 0.1,
        zoomSpeed    = 0.01,
    },

    -- Free camera (unattached fly-around view)
    freeCamera = {
        moveSpeed      = 1.0,
        fastMultiplier = 100.0,
        mouseSensitivity = 0.003,
    },

    -- Texture quality (cube map resolution for planet/moon surfaces)
    planetTexRes       = 1024,
    moonTexRes         = 1024,

    dispoMin               = -1.0,
    dispoNeutral           = 0.0,
    dispoMax               = 1.0,
    dispoHostileThreshold  = -0.3333333,
    dispoFriendlyThreshold = 0.3333333,
    dispoName              = { "hostile",
        "neutral",
        "friendly" },
}
