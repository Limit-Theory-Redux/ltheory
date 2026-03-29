--! this can be split up better
Config.game = {
    boostCost              = 20,

    explosionSize          = 64,

    autoTarget             = false,

    pulseColorBodyR        = 0.3,
    pulseColorBodyG        = 0.8,
    pulseColorBodyB        = 2.0,
    pulseColorLightR       = 0.3,
    pulseColorLightG       = 0.9,
    pulseColorLightB       = 3.0,

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

    shipFlight = (function()
        local ScaleConfig = require("Config.Gen.UniverseScaleConfig")

        -- Define desired real-world values
        local maxSpeedReal    = 8800     -- m/s max speed (~8.8 km/s for fighter)
        local accelTimeReal   = 8.0      -- seconds to reach max speed
        local lateralRatio    = 0.5      -- lateral/vertical thrust vs forward
        local verticalRatio   = 0.25     -- vertical thrust vs forward

        -- Convert to game units using ScaleConfig
        local maxSpeedGame = ScaleConfig:toGameUnits(maxSpeedReal, "starSystem")
        local accelGame    = maxSpeedGame / accelTimeReal

        -- Derive physics values
        local mass = 12000
        -- Higher drag = faster deceleration, lower top speed without thrust
        -- thrust computed from desired max speed and drag: thrust = maxSpeed * mass * drag
        local drag = 2.0
        local thrust = maxSpeedGame * mass * drag

        return {
            -- Auto-computed from desired speed/acceleration
            thrustForward  = thrust,
            thrustRight    = thrust * lateralRatio,
            thrustUp       = thrust * verticalRatio,

            -- Rotational torques (independent of speed formula)
            torquePitch    = 5000000,
            torqueYaw      = 5000000,
            torqueRoll     = 4000000,

            -- Auto-computed physics
            linearDrag     = drag,
            angularDrag    = 5.0,

            -- Input inversion
            invertYaw      = false,
            invertPitch    = false,

            -- Reference values for other systems
            maxSpeedGame   = maxSpeedGame,
            maxSpeedReal   = maxSpeedReal,
        }
    end)(),

    -- Ship hull classes (from legacy, indexed by Enums.ShipHulls)
    -- Solo=1, Small=2, Compact=3, Medium=4, Large=5, VeryLarge=6
    shipHulls = {
        scale  = { 4, 7, 10, 14, 19, 24 },
        radius = { 12, 15, 19, 24, 30, 38 },
        mass   = { 12000, 18000, 23000, 30000, 42000, 70000 },
        drag   = { 0.75, 0.75, 0.75, 0.75, 0.75, 0.75 },
        -- Translation speed multiplier (smaller = less lateral/vertical for big ships)
        tranM  = { 0.8, 0.2, 0.1, 0.06, 0.03, 0.0 },
        -- Maneuverability multiplier (pitch/yaw/roll scaling)
        manuM  = { 0.3, 0.5, 0.2, 0.1, 0.06, 1.5 },
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

    solarSystemPlayable = {
        defaultSeed        = 12345,

        -- Ship spawn
        shipScale          = 1.5,
        shipSpawnOffset    = 3.0,     -- Multiplier of planet radius for spawn distance

        -- Station spawn
        maxStations        = 2,       -- Max stations to spawn (limited by planet count)
        stationScale       = 2.0,
        stationOrbitMult   = 2.5,     -- Multiplier of planet radius for station orbit distance

        -- Orbit camera defaults
        orbitDistance       = 30,
        orbitMinDistance    = 5,
        orbitMaxDistance    = 500,
        orbitSmoothing     = 0.1,
        orbitZoomSpeed     = 8.0,

        -- Free camera defaults
        freeMoveSpeed      = 100.0,
        freeFastMult       = 10.0,
        freeMouseSens      = 0.003,

        -- Texture quality (cube map resolution for planet/moon surfaces)
        planetTexRes       = 1024,
        moonTexRes         = 1024,
    },

    dispoMin               = -1.0,
    dispoNeutral           = 0.0,
    dispoMax               = 1.0,
    dispoHostileThreshold  = -0.3333333,
    dispoFriendlyThreshold = 0.3333333,
    dispoName              = { "hostile",
        "neutral",
        "friendly" },
}
