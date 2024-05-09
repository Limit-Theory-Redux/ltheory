--! This should be broken down into their own configurations at some point

Config.gen = {
    seedGlobal                       = nil,            -- Set to force deterministic global RNG
    seedSystem                       = nil,            -- Set to force deterministic system generation

    origin                           = Vec3f(0, 0, 0), -- Set far from zero to test engine precision

    nFields                          = 1,
    nFieldsMax                       = 1,
    nFieldSize                       = function(rng) return 200 * (rng:getExp() + 1.0) end,
    nAsteroids                       = 150, -- asteroids per asteroid field (smaller = less CPU hit)
    nPlanets                         = 1,
    nPlanetsMax                      = 1,
    nStations                        = 1,
    nBeltSize                        = function(rng) return 0 end, -- asteroids per planetary belt

    nAIPlayers                       = 0,                          -- # of AI players (who manage Economic assets)
    randomizeAIPlayers               = false,
    nEconNPCs                        = 0,                          -- # of ships to be given Economic actions (managed by AI players)
    randomizeEconNPCs                = false,
    nEscortNPCs                      = 0,                          -- # of ships to be given the Escort action
    randomizeEscortNPCs              = false,
    nPirateNPCs                      = 0,
    randomizePirateNPCs              = false,

    nDustFlecks                      = 256,
    nDustClouds                      = 8,
    nStars                           = function(rng) return 30000 * (1.0 + 0.5 * rng:getExp()) end,
    nebulaBrightnessScale            = 4.0,

    uniqueShips                      = true,
    shipRes                          = 8,
    shipInventorySize                = 10,
    shipHullScale                    = { 4, -- Solo
        7,                                  -- Small
        10,                                 -- Compact
        14,                                 -- Medium
        19,                                 -- Large
        24 },                               -- Very Large
    shipHullRadius                   = { 12,
        15,
        19,
        24,
        30,
        38 },
    shipHullMass                     = { 12000,
        18000,
        23000,
        30000,
        42000,
        70000 },
    shipHullTranM                    = { 0.8, -- left/right/up/down translation speed based on hull size
        0.2,
        0.1,
        0.06,
        0.03,
        0.0 },
    shipHullManuM                    = { 0.3, -- pitch/roll/yaw speed based on hull size
        0.5,
        0.2,
        0.1,
        0.06,
        1.5 },                           -- radius is already slowing maneuvering

    shipComponents                   = { -- Sockets available for (ComponentEnums.lua):
        { 0, 0, 1, 3, 5,  8 },           -- armor plates
        { 0, 0, 0, 1, 2,  4 },           -- bay weapons
        { 1, 2, 3, 4, 6,  8 },           -- capacitors
        { 0, 0, 0, 0, 0,  0 },           -- cloak
        { 1, 1, 2, 3, 5,  8 },           -- communicators
        { 1, 2, 2, 3, 4,  6 },           -- computers
        { 1, 2, 4, 7, 12, 20 },          -- drone racks (* 2 drones each)
        { 1, 2, 4, 6, 10, 20 },          -- hull integrity
        { 1, 2, 3, 5, 7,  10 },          -- cargo pods (* shipInventorySize inventory each)
        { 1, 2, 3, 4, 5,  6 },           -- sensors
        { 0, 1, 2, 5, 8,  12 },          -- shield generators
        { 1, 2, 4, 5, 6,  8 },           -- thrusters * 2 (bilateral)
        { 2, 4, 6, 8, 10, 16 },          -- turret weapons * 2 (bilateral)
    },

    stationHullMass                  = { 112000,
        242000,
        575000 },
    stationInventorySize             = 100,
    stationComponents                = {
        -- trade --   --- mil ----    -- Sockets available for (ComponentEnums.lua):
        { 8,  16, 32,  24, 48,  96 },  -- armor plates
        { 4,  8,  16,  12, 32,  64 },  -- bay weapons
        { 8,  16, 24,  16, 24,  36 },  -- capacitors
        { 0,  0,  0,   0,  0,   0 },   -- cloak
        { 16, 24, 32,  12, 24,  40 },  -- communicators
        { 12, 16, 24,  20, 32,  48 },  -- computers
        { 0,  0,  0,   24, 48,  64 },  -- drone racks (* 2 drones each)
        { 30, 80, 130, 45, 100, 250 }, -- hull integrity
        { 24, 50, 100, 4,  10,  16 },  -- cargo pods (* stationInventorySize inventory each)
        { 8,  10, 16,  12, 16,  24 },  -- sensors
        { 16, 24, 32,  24, 40,  64 },  -- shield generators
        { 0,  0,  0,   0,  0,   0 },   -- thrusters (none on stations)
        { 16, 24, 32,  64, 128, 256 }, -- turret weapons
    },

    planetInventorySize              = 10000,
    planetComponents                 = {
        --  S    M    L    -- Sockets available for (ComponentEnums.lua):
        { 0,   0,   0 },   -- armor plates
        { 0,   0,   0 },   -- bay weapons
        { 0,   0,   0 },   -- capacitors
        { 0,   0,   0 },   -- cloak
        { 100, 160, 350 }, -- communicators
        { 100, 280, 400 }, -- computers
        { 0,   0,   0 },   -- drone racks (* 2 drones each)
        { 0,   0,   0 },   -- hull integrity
        { 240, 400, 800 }, -- cargo pods (* planetInventorySize inventory each)
        { 50,  120, 500 }, -- sensors
        { 100, 200, 400 }, -- shield generators
        { 0,   0,   0 },   -- thrusters (none on stations)
        { 0,   0,   0 },   -- turret weapons
    },

    compArmorStats                   = {
        name       = "Armor Plating",
        healthCurr = 1000,
        healthMax  = 1000,
    },
    compCapacitorStats               = {
        name       = "Capacitor",
        healthCurr = 100,
        healthMax  = 100,
        chargeCurr = 200,
        chargeMax  = 200,
        chargeRate = 12,
    },
    compCloakStats                   = {
        name       = "Cloak",
        healthCurr = 100,
        healthMax  = 100,
        rating     = 1,
        draw       = 0.05,
    },
    compComputerStats                = {
        name         = "Computer",
        healthCurr   = 100,
        healthMax    = 100,
        rating       = 1,
        lockCount    = 1,
        lockStrength = 1,
        mappingSpeed = 1,
    },
    compCommunicatorStats            = {
        name       = "Communicator",
        healthCurr = 100,
        healthMax  = 100,
        rating     = 1,
    },
    compDroneStats                   = {
        name         = "Drone Rack",
        healthCurr   = 100,
        healthMax    = 100,
        rateOfFire   = 10,
        droneType    = 1, -- 1 = mining (1 beam turret), 2 = combat (1 pulse turret)
        dronesCurr   = 2,
        dronesActive = 0,
        dronesMax    = 2,
        droneRange   = 8000,
        droneSpeed   = 40,
        reloadTime   = 5,
    },
    compHullStats                    = {
        name       = "Hull Structure",
        healthCurr = 100,
        healthMax  = 100,
    },
    compInventoryStats               = {
        name       = "Transport Pod",
        healthCurr = 100,
        healthMax  = 100,
        capacity   = 10,
        stateroom  = false,
    },
    compSensorStats                  = {
        name         = "Sensor",
        healthCurr   = 100,
        healthMax    = 100,
        rating       = 1,
        lockBreaking = 1,
        mappingRange = 2000,
        scanDetail   = 1,
        scanSpeed    = 10,
    },
    compShieldStats                  = {
        name         = "Shield Generator",
        healthCurr   = 100,
        healthMax    = 100,
        strengthCurr = 100,
        strengthMax  = 100,
        reviveRate   = 2,
        resistances  = { 85, 10, 5, 15 }, -- Energy, Kinetic, Explosive, Radiation (percentage scale)
        colorR       = 0.3,
        colorG       = 0.8,
        colorB       = 2.0,
    },
    compThrusterStats                = {
        name        = "Thruster",
        healthCurr  = 100,
        healthMax   = 100,
        speedMax    = 1000,
        maneuverMax = 100,
    },
    compTurretPulseStats             = {
        name               = "Pulse Turret",
        healthCurr         = 100,
        healthMax          = 100,
        autoTarget         = false,
        weaponRPM          = 700,
        weaponRPMDeviation = 0.12,
        damageType         = 1,
        damage             = 2,
        size               = 64,
        spread             = 0.01,
        range              = 2000,
        speed              = 800,
        charge             = 1.0,
        colorBodyR         = 0.3,
        colorBodyG         = 0.8,
        colorBodyB         = 2.0,
        colorLightR        = 0.3,
        colorLightG        = 0.9,
        colorLightB        = 3.0,
    },
    compTurretBeamStats              = {
        name               = "Beam Turret",
        healthCurr         = 100,
        healthMax          = 100,
        autoTarget         = false,
        weaponRPM          = 5000,
        weaponRPMDeviation = 0.05,
        heat               = 1,
        cooldown           = 1,
        damageType         = 1,
        damage             = 2,
        size               = 64,
        range              = 800,
        charge             = 1.0,
        colorR             = 0.3,
        colorG             = 0.8,
        colorB             = 2.0,
    },
    compTurretRailStats              = {
        name               = "Railgun Turret",
        healthCurr         = 100,
        healthMax          = 100,
        autoTarget         = false,
        weaponRPM          = 20,
        weaponRPMDeviation = 0.0,
        heat               = 1,
        cooldown           = 1,
        damageType         = 2,
        damage             = 4,
        size               = 64,
        range              = 1000,
        colorR             = 2.0,
        colorG             = 2.0,
        colorB             = 2.0,
    },
    compTurretProjStats              = {
        name               = "Launcher Turret",
        healthCurr         = 100,
        healthMax          = 100,
        weaponRPM          = 200,
        weaponRPMDeviation = 0.05,
        type               = 1, -- 1 = missile only
        guidanceType       = 1,
        damageType         = 3,
        damage             = 10,
        speed              = 100,
        range              = 20000,
    },
    compBayPulseStats                = {
        name               = "Pulse Bay",
        healthCurr         = 100,
        healthMax          = 100,
        autoTarget         = false,
        weaponRPM          = 700,
        weaponRPMDeviation = 0.12,
        heat               = 1,
        cooldown           = 1,
        damageType         = 1,
        damage             = 15,
        size               = 128,
        spread             = 0.02,
        range              = 2400,
        speed              = 600,
        charge             = 8.0,
        colorBodyR         = 1.5,
        colorBodyG         = 0.8,
        colorBodyB         = 0.4,
        colorLightR        = 2.0,
        colorLightG        = 0.9,
        colorLightB        = 0.6,
    },
    compBayBeamStats                 = {
        name               = "Beam Bay",
        healthCurr         = 100,
        healthMax          = 100,
        autoTarget         = false,
        weaponRPM          = 5000,
        weaponRPMDeviation = 0.05,
        heat               = 1,
        cooldown           = 1,
        damageType         = 1,
        damage             = 2,
        size               = 64,
        range              = 1000,
        charge             = 1.0,
        colorR             = 0.3,
        colorG             = 0.8,
        colorB             = 2.0,
    },
    compBayRailStats                 = {
        name               = "Railgun Bay",
        healthCurr         = 100,
        healthMax          = 100,
        autoTarget         = false,
        weaponRPM          = 20,
        weaponRPMDeviation = 0.0,
        heat               = 1,
        cooldown           = 1,
        damageType         = 2,
        damage             = 4,
        size               = 64,
        range              = 4000,
        colorR             = 2.0,
        colorG             = 2.0,
        colorB             = 2.0,
    },
    compBayCannonStats               = {
        name               = "Cannon Bay",
        healthCurr         = 100,
        healthMax          = 100,
        autoTarget         = false,
        weaponRPM          = 1500,
        weaponRPMDeviation = 0.25,
        heat               = 1,
        cooldown           = 1,
        damageType         = 1,
        damage             = 50,
        size               = 64,
        range              = 500,
        charge             = 50.0,
        colorR             = 0.3,
        colorG             = 0.8,
        colorB             = 2.0,
    },
    compBayProjStats                 = {
        name               = "Launcher Bay",
        healthCurr         = 100,
        healthMax          = 100,
        weaponRPM          = 200,
        weaponRPMDeviation = 0.05,
        type               = 1, -- 1 = missile, 2 = torpedo
        guidanceType       = 1,
        damageType         = 3,
        damage             = 55,
        speed              = 100,
        range              = 20000,
    },

    objectEmissionsDropoff           = { 1e8, -- star
        1e6,                                  -- planet
        20000,                                -- station
        5000,                                 -- ship
    },
    objectEmissions                  = {
        -- STAR  PLANET  STATION  SHIP  THRUSTER PULSE  BEAM  JUMP  CLOAK
        { 5,   21, 10, 16, 0,  0,  0,  9,  -5 },  --              |   1  Hz
        { 10,  13, 27, 10, 0,  0,  0,  37, -5 },  --              |   5  Hz
        { 13,  7,  30, 8,  0,  0,  4,  11, 0 },   --              |  10  Hz > Audio
        { 36,  1,  11, 3,  6,  0,  22, 1,  0 },   --              |  50  Hz > Audio
        { 11,  0,  24, 7,  17, 0,  67, 0,  0 },   --              | 100  Hz > Audio
        { 8,   0,  11, 15, 4,  3,  19, 0,  -10 }, --              | 500  Hz > Audio
        { 3,   0,  19, 28, 1,  17, 2,  0,  -10 }, --              |   1 kHz > Audio
        { 14,  0,  25, 35, 0,  61, 0,  0,  -20 }, --  Low-band    |   5 kHz > Audio
        { 3,   0,  32, 43, 0,  11, 0,  0,  -30 }, --              |  10 kHz > Audio
        { 18,  5,  45, 50, 0,  0,  0,  0,  -40 }, --              |  50 kHz
        { 19,  32, 54, 62, 0,  0,  0,  0,  -40 }, --              | 100 khz
        { 26,  73, 71, 21, 0,  0,  0,  0,  -10 }, --              | 500 khz > AM radio
        { 20,  84, 59, 5,  0,  0,  0,  0,  0 },   --              |   1 MHz > AM radio     > Ultrasound
        { 11,  60, 46, 18, 0,  0,  0,  0,  -5 },  --              |   5 MHz > AM radio     > Ultrasound
        { 9,   31, 30, 27, 0,  0,  0,  2,  -10 }, --              |  10 MHz                > Ultrasound
        { 3,   57, 18, 22, 0,  0,  0,  13, -10 }, --             ||  50 Mhz > TV
        { 12,  22, 6,  10, 4,  0,  0,  41, -5 },  --             || 100 Mhz > TV           > FM radio
        { 5,   36, 18, 13, 23, 0,  0,  7,  -5 },  --             || 500 Mhz > Microwaves
        { 9,   39, 32, 16, 13, 0,  0,  0,  -10 }, --             ||   1 GHz > Microwaves   > Mobile phones
        { 19,  47, 37, 21, 2,  0,  0,  0,  -15 }, --             ||   5 GHz > Microwaves   > WiFi
        { 7,   51, 43, 33, 0,  0,  0,  0,  -20 }, --             ||  10 GHz > Microwaves   > Radar
        { 1,   42, 40, 17, 0,  0,  0,  0,  -10 }, --             ||  50 GHz > Microwaves   > Radar
        { 4,   26, 21, 16, 0,  0,  0,  0,  -10 }, --             || 100 GHz > Microwaves   > Radar
        { 9,   54, 31, 35, 0,  0,  0,  0,  -20 }, --  Mid-band   || 500 GHz > Bio-imaging
        { 19,  71, 70, 12, 0,  0,  0,  0,  -10 }, --             ||   1 THz > Bio-imaging  > Infrared
        { 29,  79, 87, 8,  1,  0,  0,  3,  -5 },  --             ||   5 THz > Bio-imaging  > Infrared
        { 57,  68, 61, 15, 5,  0,  8,  14, -5 },  --             ||  10 THz > Bio-imaging  > Infrared
        { 66,  28, 46, 22, 12, 10, 17, 26, -10 }, --             ||  50 THz > Bio-imaging  > Infrared
        { 78,  20, 33, 51, 51, 72, 54, 70, -30 }, --             || 100 THz                > Infrared
        { 100, 12, 29, 25, 90, 82, 89, 79, -15 }, --             || 500 THz > Visible light
        { 81,  3,  29, 32, 72, 41, 37, 8,  -15 }, --             ||   1 PHz > Ultraviolet
        { 66,  6,  35, 19, 59, 90, 6,  0,  -5 },  --             ||   5 PHz > Ultraviolet
        { 41,  1,  24, 6,  21, 21, 8,  0,  0 },   --             ||  10 PHz > Ultraviolet
        { 51,  0,  8,  1,  5,  3,  26, 0,  0 },   --            |||  50 PHz > Soft X-ray
        { 45,  0,  0,  0,  0,  0,  74, 0,  0 },   --            ||| 100 PHz > Soft X-ray
        { 36,  0,  0,  0,  2,  0,  61, 0,  0 },   --            ||| 500 PHz > Soft X-ray
        { 28,  0,  0,  0,  8,  0,  14, 0,  0 },   --            |||   1 EHz > Soft X-ray
        { 23,  0,  1,  0,  17, 0,  2,  0,  0 },   --            |||   5 EHz > Soft X-ray
        { 17,  1,  3,  0,  31, 0,  0,  0,  0 },   --            |||  10 EHz > Soft X-ray
        { 25,  0,  3,  0,  11, 0,  0,  0,  0 },   --  High-band |||  50 EHz > Hard X-ray
        { 18,  0,  2,  0,  3,  0,  0,  0,  0 },   --            ||| 100 EHz > Hard X-ray
        { 16,  0,  1,  0,  1,  0,  0,  0,  0 },   --            ||| 500 EHz > Gamma
        { 24,  0,  0,  0,  0,  0,  0,  0,  0 },   --            |||   1 ZHz > Gamma
        { 17,  1,  0,  0,  0,  0,  0,  0,  0 },   --            |||   5 ZHz > Gamma
        { 9,   7,  2,  0,  0,  0,  0,  0,  0 },   --            |||  10 ZHz
        { 2,   2,  1,  0,  0,  0,  0,  7,  0 },   --            |||  50 ZHz
        { 1,   0,  0,  0,  0,  0,  0,  88, 0 },   --            ||| 100 ZHz # Jump wave
        { 0,   0,  0,  0,  0,  0,  0,  1,  0 },   --            ||| 500 ZHz
    },

    nebulaRes                        = 2048,

    zNearBack                        = 0.1,
    zNearReal                        = 0.1, -- 0.1
    zFarBack                         = 1e6,
    zFarReal                         = 1e4, -- 1e6

    scaleSystemBack                  = 2e5,
    scaleSystemReal                  = 2e4,  -- 2e9 maximum, but anything bigger than 5e4 currently introduces a horrible "wobble"
    scalePlanetBack                  = 120000,
    scalePlanetReal                  = 8000, -- 15000
    scalePlanetModBack               = 7e4,
    scalePlanetModReal               = 1,    -- 4e5

    scaleSystem                      = 1e6,  -- this needs to be extended massively; see also zFar and zNear
    scaleStar                        = 1e6,
    scalePlanet                      = 5e3,
    scalePlanetMod                   = 7e4, -- 7e4
    scaleFieldAsteroid               = 40000,
    scaleAsteroid                    = 7.0,
    scaleStation                     = 70,

    sizePlanet                       = 2,         -- 1 = small, 2 = medium, 3 = large

    radiusStarTrue                   = 695700000, -- nominal radius of Sun is 695,700 km; VY Canis Majoris is ~1,420 x Solar radius
    radiusPlanetTrue                 = 6371000,   -- average radius of Earth is 6,371 km; Ceres = 470 km; Jupiter = 70,000 km
    radiusAsteroidTrue               = 50000,     -- 0.005 km to 450 km
    massStarTrue                     = 2e30,      -- 1.98 x 10^30 is the Sun's mass in kg; Westerhout 49-2 is ~250 x Solar mass
    massPlanetTrue                   = 6e24,      -- 5.97e24 is Earth's mass in kg (1e10 as a test value)
    massAsteroidTrue                 = 5e18,      -- typical mass for a 50 km asteroid; 50m = ~1,000,000,000 kg

    massAsteroidExp                  = { 4.1,     -- Carbonaceous
        5.9,                                      -- Metallic
        3.2 },                                    -- Silicaceous

    stationMinimumDistance           = 2000,      -- minimum distance between stations
    minimumDistancePlacementMaxTries = 100
}
