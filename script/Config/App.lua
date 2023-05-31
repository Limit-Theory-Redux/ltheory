Config.org = 'LTheoryRedux'
Config.app = 'LTheoryRedux'

Config.orgInfo = {
  repository = "https://github.com/Limit-Theory-Redux/ltheory",
  discord = "https://discord.gg/MrfRR5ytJF",
}

Config.gameTitle   = "Limit Theory Redux"
Config.gameVersion = "v0.008"

Config.userInitFilename = "user.ini"

Config.timeToResetToSplashscreen = 60

Config.render = {
  defaultResX     = 1600,
  defaultResY     = 900,
  fullscreen      = false,
  vsync           = true,
  zNear           = 0.1, -- default: 0.1
  zFar            = 1e6, -- default: 1e6
  thrusterLights  = false,
  pulseLights     = false,
}

Config.audio = {
  soundEnabled = true,
  supportedFormats = {".ogg"},
  mainMenuMusicEnabled = true,
  bSoundOn    = false,
  soundMin    = 0,
  soundMax    = 1, -- SetVolume range seems to go from 0 (min) to about 2 or 3 (max)
  musicVolume = 0.75, -- current volume
  mainMenu    = "LTR_Main_Menu.ogg",

  pulseFireName      = "",
  pulseFire          = nil,
  pulseHitName       = "",
  pulseHit           = nil,
  explodeShipName    = "",
  explodeShip        = nil,
  explodeStationName = "",
  explodeStation     = nil,
}

Config.paths = {
  files         = Directory.GetPrefPath(Config.org, Config.app), -- base directory using environment-agnostic path
  soundAmbiance = "./res/sound/system/audio/music/",
  soundEffects  = "./res/sound/system/audio/fx/",
  enums         = "./script/Enums/",
  types         = "./script/Types/"
}

Config.debug = {
  metricsEnabled  = false,
  window          = true, -- Debug window visible by default at launch?
  windowSection   = nil,  -- Set to the name of a debug window section to
                          -- collapse all others by default

  instantJobs     = false, -- set to true to speed up economic testing
  jobSpeed        = 10000, -- acceleration rate for instant jobs (in MineAt, DockAt)

  timeAccelFactor = 10, -- acceleration rate when holding "TimeAccel" input

  printConfig = false
}

Config.debug.physics = {
  drawWireframes         = false,
  drawBoundingBoxesLocal = false,
  drawBoundingBoxesworld = false,
}

local goodSeeds = {
  14589938814258111262ULL,
  15297218883250103974ULL,
  1842258441393851360ULL,
  1305797465843153519ULL,
  5421862249219039751ULL,
  638780708004697442ULL,
}

Config.gen = {
  seedGlobal = nil, -- Set to force deterministic global RNG
  seedSystem = nil, -- Set to force deterministic system generation

  origin     = Vec3f(0, 0, 0), -- Set far from zero to test engine precision

  nFields    = 20,
  nFieldSize = function (rng) return 200 * (rng:getExp() + 1.0) end,
  nAsteroids = 200, -- asteroids per asteroid field (smaller = less CPU hit)
  nPlanets   = 0,
  nStations  = 0,
  nBeltSize  = function (rng) return 0 end, -- asteroids per planetary belt

  nDustFlecks = 256,
  nDustClouds = 8,
  nStars      = function (rng) return 30000 * (1.0 + 0.5 * rng:getExp()) end,
  nebulaBrightnessScale = 1.0,

  uniqueShips   = true,
  shipRes       = 8,
  shipHullScale = { 4,  -- Solo
                    7,  -- Small
                   10,  -- Compact
                   14,  -- Medium
                   19,  -- Large
                   24}, -- Very Large
  shipHullRadius = { 12,
                     15,
                     19,
                     24,
                     30,
                     38},
  shipHullMass   = { 12000,
                     18000,
                     23000,
                     30000,
                     42000,
                     70000},
  shipHullTranM  = { 0.8, -- left/right/up/down translation speed based on hull size
                     0.2,
                     0.1,
                     0.06,
                     0.03,
                     0.0},
  shipHullManuM  = { 0.8, -- pitch/roll/yaw speed based on hull size
                     0.5,
                     0.35,
                     0.25,
                     0.16,
                     4.0}, -- radius is already slowing maneuvering
  shipComponents = {                           -- Sockets available for (ComponentEnums.lua):
                     { 1,  2,  4,  6, 10, 20}, -- hull integrity
                     { 1,  2,  2,  3,  4,  6}, -- computers
                     { 1,  2,  3,  4,  5,  6}, -- sensors
                     { 1,  1,  2,  3,  3,  5}, -- life support
                     { 1,  2,  3,  4,  6,  8}, -- capacitors
                     { 1,  2,  4,  5,  6,  8}, -- thrusters * 2 (bilateral)
                     { 2,  4,  6,  8, 10, 16}, -- turret weapons * 2 (bilateral)
                     { 0,  0,  0,  1,  2,  4}, -- bay weapons
                     { 1,  2,  3,  5,  7, 10}, -- cargo pods (* shipInventorySize inventory each)
                     { 1,  2,  4,  7, 12, 20}, -- drone racks (* 2 drones each)
                     { 0,  1,  2,  5,  8, 12}, -- shield generators
                     { 0,  0,  1,  3,  5,  8}, -- armor plates
                   },
  shipInventorySize    =  10,
  stationInventorySize = 100,
  stationHullMass   = { 112000,
                        242000,
                        575000},
  stationComponents = {    -- trade ---   --- mil ----   -- Sockets available for (ComponentEnums.lua):
                        {  30,  80, 130,  45, 100, 250}, -- hull integrity
                        {  12,  16,  24,  20,  32,  48}, -- computers
                        {   8,  10,  16,  12,  16,  24}, -- sensors
                        {   8,  12,  16,  12,  16,  20}, -- life support
                        {   8,  16,  24,  16,  24,  36}, -- capacitors
                        {   0,   0,   0,   0,   0,   0}, -- thrusters (none on stations)
                        {  16,  24,  32,  64, 128, 256}, -- turret weapons
                        {   4,   8,  16,  12,  32,  64}, -- bay weapons
                        {  24,  50, 100,   4,  10,  16}, -- cargo pods (* stationInventorySize inventory each)
                        {   0,   0,   0,  24,  48,  64}, -- drone racks (* 2 drones each)
                        {  16,  24,  32,  24,  40,  64}, -- shield generators
                        {   8,  16,  32,  24,  48,  96}, -- armor plates
                      },

  compHullStats      = {
                         name          = "Hull Structure",
                         healthCurr    = 100,
                         healthMax     = 100,
                       },
  compComputerStats  = {
                         name          = "Computer",
                         healthCurr    = 100,
                         healthMax     = 100,
                         cpuCount      = 1,
                         mappingSpeed  = 1,
                         lockCount     = 1,
                         lockStrength  = 1,
                       },
  compSensorStats    = {
                         name          = "Sensor",
                         healthCurr    = 100,
                         healthMax     = 100,
                         mappingRange  = 2000,
                         scanSpeed     = 10,
                         scanDetail    = 1,
                         lockBreaking  = 1,
                       },
  compLifeSupportStats = {
                         name          = "Life Support",
                         healthCurr    = 100,
                         healthMax     = 100,
                         pods          = 2, -- number of cargo pods given life support
                       },
  compCapacitorStats = {
                         name          = "Capacitor",
                         healthCurr    = 100,
                         healthMax     = 100,
                         chargeCurr    = 200,
                         chargeMax     = 200,
                         chargeRate    = 12,
                       },
  compThrusterStats  = {
                         name          = "Thruster",
                         healthCurr    = 100,
                         healthMax     = 100,
                         speedMax      = 1000,
                         maneuverMax   = 100,
                       },
  compInventoryStats = {
                         name          = "Transport Pod",
                         healthCurr    = 100,
                         healthMax     = 100,
                         capacity      = 10,
                         stateroom     = false,
                       },
  compDroneStats     = {
                         name          = "Drone Rack",
                         healthCurr    = 100,
                         healthMax     = 100,
                         rateOfFire    = 10,
                         droneType     = 1, -- 1 = mining (1 beam turret), 2 = combat (1 pulse turret)
                         dronesCurr    = 2,
                         dronesActive  = 0,
                         dronesMax     = 2,
                         droneRange    = 8000,
                         droneSpeed    = 40,
                         reloadTime    = 5,
                       },
  compShieldStats    = {
                         name          = "Shield Generator",
                         healthCurr    = 100,
                         healthMax     = 100,
                         strengthCurr  = 100,
                         strengthMax   = 100,
                         reviveRate    = 2,
                         resistances   = {85, 10, 5, 15}, -- Energy, Kinetic, Explosive, Radiation (percentage scale)
                         colorR        = 0.3,
                         colorG        = 0.8,
                         colorB        = 2.0,
                       },
  compArmorStats     = {
                         name          = "Armor Plating",
                         healthCurr    = 1000,
                         healthMax     = 1000,
                       },

  compTurretPulseStats = {
                           name               = "Pulse Turret",
                           healthCurr         = 100,
                           healthMax          = 100,
                           autoTarget         = false,
                           roundsPerMinute    = 450,
                           rpmDeviation       = 0.02,
                           damageType         = 1,
                           damage             = 2,
                           size               = 64,
                           spread             = 0.01,
                           range              = 1000,
                           speed              = 1000,
                           charge             = 1.0,
                           colorBodyR         = 0.3,
                           colorBodyG         = 0.8,
                           colorBodyB         = 2.0,
                           colorLightR        = 0.3,
                           colorLightG        = 0.9,
                           colorLightB        = 3.0,
                         },
  compTurretBeamStats  = {
                           name               = "Beam Turret",
                           healthCurr         = 100,
                           healthMax          = 100,
                           autoTarget         = false,
                           roundsPerMinute    = 450,
                           rpmDeviation       = 0.02,
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
  compTurretRailStats  = {
                           name               = "Railgun Turret",
                           healthCurr         = 100,
                           healthMax          = 100,
                           autoTarget         = false,
                           roundsPerMinute    = 450,
                           rpmDeviation       = 0.02,
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
  compTurretProjStats  = {
                           name               = "Launcher Turret",
                           healthCurr         = 100,
                           healthMax          = 100,
                           roundsPerMinute    = 450,
                           rpmDeviation       = 0.02,
                           type               = 1, -- 1 = missile only
                           guidanceType       = 1,
                           damageType         = 3,
                           damage             = 10,
                           speed              = 100,
                           range              = 20000,
                         },
  compBayPulseStats    = {
                           name               = "Pulse Bay",
                           healthCurr         = 100,
                           healthMax          = 100,
                           autoTarget         = false,
                           roundsPerMinute    = 450,
                           rpmDeviation       = 0.02,
                           heat               = 1,
                           cooldown           = 1,
                           damageType         = 1,
                           damage             = 15,
                           size               = 128,
                           spread             = 0.02,
                           range              = 1200,
                           speed              = 800,
                           charge             = 8.0,
                           colorBodyR         = 1.5,
                           colorBodyG         = 0.8,
                           colorBodyB         = 0.4,
                           colorLightR        = 2.0,
                           colorLightG        = 0.9,
                           colorLightB        = 0.6,
                         },
  compBayBeamStats     = {
                           name               = "Beam Bay",
                           healthCurr         = 100,
                           healthMax          = 100,
                           autoTarget         = false,
                           roundsPerMinute    = 450,
                           rpmDeviation       = 0.02,
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
  compBayRailStats     = {
                           name               = "Railgun Bay",
                           healthCurr         = 100,
                           healthMax          = 100,
                           autoTarget         = false,
                           roundsPerMinute    = 450,
                           rpmDeviation       = 0.02,
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
  compBayCannonStats   = {
                           name               = "Cannon Bay",
                           healthCurr         = 100,
                           healthMax          = 100,
                           autoTarget         = false,
                           roundsPerMinute    = 450,
                           rpmDeviation       = 0.02,
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
  compBayProjStats     = {
                           name               = "Launcher Bay",
                           healthCurr         = 100,
                           healthMax          = 100,
                           roundsPerMinute    = 450,
                           rpmDeviation       = 0.02,
                           type               = 1, -- 1 = missile, 2 = torpedo
                           guidanceType       = 1,
                           damageType         = 3,
                           damage             = 55,
                           speed              = 100,
                           range              = 20000,
                         },

  nebulaRes   = 1024, -- 2048 sometimes creates nebulae with stright-line edges

  nAIPlayers          = 0,  -- # of AI players (who manage Economic assets)
  randomizeAIPlayers    = false,
  nEconNPCs           = 0,  -- # of ships to be given Economic actions (managed by AI players)
  randomizeEconNPCs   = false,
  nEscortNPCs         = 0,  -- # of ships to be given the Escort action
  randomizeEscortNPCs = false,

  zNearBack          = 0.1,
  zNearReal          = 0.1, -- 0.1
  zFarBack           = 1e6,
  zFarReal           = 1e4, -- 1e6

  scaleSystemBack    = 2e5,
  scaleSystemReal    = 2e4, -- 2e9 maximum, but anything bigger than 5e4 currently introduces a horrible "wobble"
  scalePlanetBack    = 120000,
  scalePlanetReal    = 8000, -- 15000
  scalePlanetModBack = 7e4,
  scalePlanetModReal = 1, -- 4e5

  scaleSystem        = 1e6,   -- this needs to be extended massively; see also zFar and zNear
  scaleStar          = 1e6,
  scalePlanet        = 5e3,
  scalePlanetMod     = 7e4,  -- 7e4
  scaleFieldAsteroid = 50000,
  scaleAsteroid      = 7.0,
  scaleStation       = 70,

  radiusStarTrue      = 695700000, -- nominal radius of Sun is 695,700 km; VY Canis Majoris is ~1,420 x Solar radius
  radiusPlanetTrue    =   6371000, -- average radius of Earth is 6,371 km; Ceres = 470 km; Jupiter = 70,000 km
  radiusAsteroidTrue  =     50000, -- 0.005 km to 450 km
  massStarTrue        = 2e30,  -- 1.98 x 10^30 is the Sun's mass in kg; Westerhout 49-2 is ~250 x Solar mass
  massPlanetTrue      = 6e24,  -- 5.97e24 is Earth's mass in kg (1e10 as a test value)
  massAsteroidTrue    = 5e18,  -- typical mass for a 50 km asteroid; 50m = ~1,000,000,000 kg

  massAsteroidExp = {4.1,  -- Carbonaceous
                     5.9,  -- Metallic
                     3.2}, -- Silicaceous

  stationMinimumDistance = 5000, -- minimum distance between stations
  minimumDistancePlacementMaxTries = 100
}

Config.game = {
  boostCost              = 20,
  explosionSize          = 64,

  autoTarget             = true,
  pulseDamage            = 2,
  pulseSize              = 64,
  pulseSpeed             = 4e3, -- was 6e2
  pulseRange             = 4000,
  pulseSpread            = 0.01,
  pulseCharge            = 1.0, -- default amount of capacitor charge used by each shot
  pulseColorBodyR        = 0.3,
  pulseColorBodyG        = 0.8,
  pulseColorBodyB        = 2.0,
  pulseColorLightR       = 0.3,
  pulseColorLightG       = 0.9,
  pulseColorLightB       = 3.0,

  shipBuildTime          = 10,
  shipEnergy             = 100,
  shipEnergyRecharge     = 10,
  shipHealth             = 100,
  shipHealthRegen        = 0.1,

  playerDamageResistance = 1.0,
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

  aiFire                 = function (dt, rng) return rng:getExp() ^ 2 < dt end,

  autonavRanges          = {  200,  -- Unknown
                                0,  -- Reserved
                                0,  -- Star Sector
                                0,  -- Star System
                             2000,  -- Zone
                              1e7,  -- Star (TODO: radius + offset)
                            10000,  -- Planet (TODO: radius + offset)
                              300,  -- Asteroid
                              500,  -- Jumpgate
                             2000,  -- Station
                              100}, -- Ship

  dockRange              = 50,

  dispoMin               = -1.0,
  dispoNeutral           =  0.0,
  dispoMax               =  1.0,
  dispoHostileThreshold  = -0.3333333,
  dispoFriendlyThreshold =  0.3333333,
  dispoName              = {"hostile",
                            "neutral",
                            "friendly"},
}

Config.econ = {
  pStartCredits           = 10000,   -- player starting credits
  eStartCredits           = 1000000, -- NPC player starting credits

  eInventory              = 100, -- starting number of inventory slots

  jobIterations           = 4000, -- how many randomly-chosen jobs an asset will consider before picking

  inputBacklog            = 1, -- multiplier of number of units a factory can bid for on each input

  pickupDistWeightMine    = 1.0, -- importance of pickup distance for a Mine job (smaller = more important)
  pickupDistWeightTran    = 3.0, -- importance of pickup distance for a Transport job (smaller = more important)
  markup                  = 1.2, -- change to base value when calculating ask price for selling an item
  markdown                = 0.8, -- change to base value when calculating bid price for buying an item

  lowAttentionUpdateRate = 5,
}

Config.ui = {
  defaultControl        = "Background",
  controlBarHeight      = 48,
  hudStyle              = 1,
  sensorsDisplayed      = true,
  cursorSmooth          = "cursor/cursor1-small",
  cursorSimple          = "cursor/simple_cursor",
  cursor                = "cursor/simple_cursor",
  cursorStyle           = 1,
  cursorX               = 1,
  cursorY               = 1,
  mapSystemZoomSpeed    = 0.1,
  mapSystemPanSpeed     = 1,

  -- Trackers
  showTrackers     = true,
  maxTrackingRange = 150000,
  trackerBracketingRenderDistances = {
    Planet    = math.huge,
    Asteroid  = 25000,
    Jumpgate  = 50000,
    Station   = math.huge,
    Ship      = 25000,
    Colony    = 200000,
  },
  trackerObjectOcclusion  = 0.1
}

Config.ui.color = {
                      --     R     G     B     A
  accent            = Color(1.00, 0.00, 0.30, 1.0),
  focused           = Color(1.00, 0.00, 0.30, 1.0),
  active            = Color(0.70, 0.00, 0.21, 1.0),
  background        = Color(0.15, 0.15, 0.15, 1.0),
  backgroundInvert  = Color(0.85, 0.85, 0.85, 1.0),
  border            = Color(0.00, 0.40, 1.00, 0.3),
  borderBright      = Color(1.00, 1.00, 1.00, 0.6),
  borderOverlay     = Color(0.20, 0.60, 1.00, 1.0),
  borderDim         = Color(0.50, 0.50, 0.50, 0.4),
  fill              = Color(0.60, 0.60, 0.60, 1.0),
  textNormal        = Color(0.75, 0.75, 0.75, 1.0),
  textNormalFocused = Color(0.00, 0.00, 0.00, 1.0),
  textInvert        = Color(0.25, 0.25, 0.25, 1.0),
  textInvertFocused = Color(0.00, 0.00, 0.00, 1.0),
  textTitle         = Color(0.80, 0.80, 0.80, 0.8),
  debugRect         = Color(0.50, 1.00, 0.50, 0.1),
  selection         = Color(1.00, 0.50, 0.10, 1.0),
  control           = Color(0.20, 0.90, 1.00, 1.0),
  controlFocused    = Color(0.20, 1.00, 0.20, 0.6),
  controlActive     = Color(0.14, 0.70, 0.14, 0.7),
  hologram          = Color(0.30, 0.40, 1.00, 0.8),
  ctrlCursor        = Color(0.20, 0.50, 1.00, 0.7),
  reticle           = Color(0.10, 0.30, 1.00, 3.0),
  windowBackground  = Color(0.00, 0.40, 1.00, 0.2),
  clientBackground  = Color(0.30, 0.30, 0.30, 0.0),
  meterBar          = Color(0.10, 0.60, 1.00, 0.7),
  meterBarDark      = Color(0.00, 0.30, 0.70, 0.1),
  meterBarOver      = Color(1.00, 0.30, 0.00, 0.6),
  hullIntegrity     = Color(0.20, 0.25, 0.30, 0.9),
  armorIntegrity    = Color(0.80, 0.75, 0.30, 0.6),
  shieldStrength    = Color(0.20, 0.50, 1.00, 0.7),
  capacitorEnergy   = Color(0.50, 0.00, 1.00, 0.7),


  healthColor = {
    --     R    G    B    A
    Color(0.0, 0.0, 0.0, 0.9), --  0% -   1% BLACK
    Color(0.1, 0.0, 0.0, 0.7), --  2% -   3%
    Color(0.2, 0.0, 0.1, 0.5), --  4% -   5%
    Color(0.3, 0.0, 0.3, 0.4), --  6% -   7%
    Color(0.4, 0.0, 0.5, 0.3), --  8% -   9%
    Color(0.5, 0.0, 0.7, 0.3), -- 10% -  11%
    Color(0.6, 0.0, 0.8, 0.2), -- 12% -  13% PURPLE
    Color(0.7, 0.0, 0.8, 0.2), -- 14% -  15%
    Color(0.8, 0.0, 0.7, 0.2), -- 16% -  17%
    Color(0.9, 0.0, 0.4, 0.2), -- 18% -  19%
    Color(1.0, 0.1, 0.2, 0.2), -- 20% -  21%
    Color(1.0, 0.1, 0.2, 0.2), -- 22% -  23%
    Color(1.0, 0.1, 0.1, 0.2), -- 24% -  25% RED
    Color(1.0, 0.2, 0.0, 0.2), -- 26% -  27%
    Color(1.0, 0.2, 0.0, 0.3), -- 28% -  29%
    Color(1.0, 0.3, 0.0, 0.3), -- 30% -  31%
    Color(0.9, 0.3, 0.0, 0.3), -- 32% -  33%
    Color(0.8, 0.4, 0.1, 0.3), -- 34% -  35%
    Color(0.8, 0.4, 0.1, 0.4), -- 36% -  37%
    Color(0.8, 0.5, 0.1, 0.5), -- 38% -  39% ORANGE
    Color(0.7, 0.5, 0.1, 0.5), -- 40% -  41%
    Color(0.7, 0.5, 0.2, 0.4), -- 42% -  43%
    Color(0.6, 0.5, 0.2, 0.3), -- 44% -  45%
    Color(0.6, 0.5, 0.2, 0.3), -- 46% -  47%
    Color(0.7, 0.6, 0.3, 0.2), -- 48% -  49%
    Color(0.7, 0.6, 0.3, 0.2), -- 50% -  51%
    Color(0.7, 0.6, 0.3, 0.2), -- 52% -  53%
    Color(0.8, 0.7, 0.4, 0.2), -- 54% -  55%
    Color(0.8, 0.7, 0.3, 0.2), -- 56% -  57%
    Color(0.8, 0.7, 0.3, 0.2), -- 58% -  59% YELLOW
    Color(0.7, 0.7, 0.2, 0.2), -- 60% -  61%
    Color(0.7, 0.7, 0.2, 0.2), -- 62% -  63%
    Color(0.6, 0.7, 0.1, 0.2), -- 64% -  65%
    Color(0.6, 0.7, 0.1, 0.2), -- 66% -  67%
    Color(0.5, 0.7, 0.0, 0.2), -- 68% -  69%
    Color(0.5, 0.8, 0.0, 0.2), -- 70% -  71%
    Color(0.4, 0.8, 0.0, 0.2), -- 72% -  73%
    Color(0.4, 0.8, 0.1, 0.2), -- 74% -  75%
    Color(0.3, 0.8, 0.1, 0.2), -- 76% -  77%
    Color(0.3, 0.8, 0.2, 0.2), -- 78% -  79% OLIVE?
    Color(0.2, 0.8, 0.2, 0.2), -- 80% -  81%
    Color(0.2, 0.9, 0.2, 0.2), -- 82% -  83%
    Color(0.2, 0.9, 0.3, 0.2), -- 84% -  85%
    Color(0.1, 0.9, 0.3, 0.2), -- 86% -  87%
    Color(0.1, 0.9, 0.2, 0.2), -- 88% -  89%
    Color(0.1, 1.0, 0.2, 0.2), -- 90% -  91%
    Color(0.1, 1.0, 0.1, 0.2), -- 92% -  93%
    Color(0.0, 1.0, 0.1, 0.2), -- 94% -  95%
    Color(0.0, 1.0, 0.0, 0.2), -- 96% -  97%
    Color(0.0, 1.0, 0.0, 0.2), -- 98% - 100% GREEN
  },
}

Config.ui.font = {
  normal     = Cache.Font('Share', 14),
  normalSize = 14,
  title      = Cache.Font('Exo2Bold', 10),
  titleSize  = 10,
}

-- Static object type names and data
Config.objectInfo = {
  {
    ID = "object_types",
    name = "Object Types",
    elems = {
      -- NOTE: If you change these, you must also change autonavRanges!
      { 1, "Unknown",     ""},
      { 2, "Reserved",    ""},
      { 3, "Star Sector", ""},
      { 4, "Star System", ""},
      { 5, "Zone",        "zone_subtypes"},
      { 6, "Star",        "star_subtypes"},
      { 7, "Planet",      "planet_subtypes"},
      { 8, "Asteroid",    "asteroid_subtypes"},
      { 9, "Jumpgate",    "jumpgate_subtypes"},
      {10, "Station",     "station_subtypes"},
      {11, "Ship",        "ship_subtypes"},
      {12, "Colony",      "colony_subtypes"},
    }
  },
  {
    ID = "zone_subtypes",
    name = "Zone Subtypes",
    elems = {
      { 1, "Unknown"},
      { 2, "Reserved"},
      { 3, "Asteroid Field"},
      { 4, "Political Extent"},
      { 5, "Military Extent"},
      { 6, "Economic Extent"},
      { 7, "Cultural Extent"},
    }
  },
  {
    ID = "planet_subtypes",
    name = "Planet Types",
    elems = {
      { 1, "Unknown"},
      { 2, "Brown Dwarf"},
      { 3, "Gas giant"},
      { 4, "Rocky"},
    }
  },
  {
    ID = "planet_subtypes_size",
    name = "Planet Subtypes - Size",
    elems = {
      { 1, "(none)"},
      { 2, "Unknown"},
      { 3, "Large"},
      { 4, "Small"},
    }
  },
  {
    ID = "planet_subtypes_atm",
    name = "Planet Subtypes - Atmosphere",
    elems = {
      { 1, "Unknown"},
      { 2, "None (vacuum)"},
      { 3, "Thin"},
      { 4, "Thin, tainted"},
      { 5, "Thin, exotic"},
      { 6, "Normal"},
      { 7, "Normal, tainted"},
      { 8, "Dense"},
      { 9, "Dense, tainted"},
      {10, "Dense, exotic"},
    }
  },
  {
    ID = "planet_subtypes_hyd",
    name = "Planet Subtypes - Hydrosphere",
    elems = {
      { 1, "Unknown"},
      { 2, "None (vacuum)"},
      { 3, "Desert (1% - 9% water)"},
      { 4, "Dry (10% - 29% water)"},
      { 5, "Wet (30% - 69% water)"},
      { 6, "Water (70% - 89% water)"},
      { 7, "Ocean (90% - 100% water)"},
    }
  },
  {
    ID = "asteroid_subtypes",     -- if you change these, also change massAsteroidExp
    name = "Asteroid Types",      -- until reference functions access the values from here
    elems = {
      { 1, "Unknown",      0.0},
      { 2, "Reserved",     0.0},
      { 3, "Carbonaceous", 5.5},
      { 4, "Metallic",     6.0},
      { 5, "Silicaceous",  5.0},
    }
  },
  {
    ID = "jumpgate_subtypes",
    name = "Jumpgate Types",
    elems = {
      { 1, "Unknown"},
      { 2, "Reserved"},
      { 3, "Neighbor"},
      { 4, "Wild"},
    }
  },
  {
    ID = "station_subtypes",
    name = "Station Types",
    elems = {
      { 1, "Unknown"},
      { 2, "Reserved"},
      { 3, "Solar Energy Array"},
      { 4, "Nuclear Reactor"},
      { 5, "Pirate"},
    }
  },
  {
    ID = "station_hulls",
    name = "Station Hulls",
    elems = {
      { 1, "Unknown"},
      { 2, "Reserved"},
      { 3, "Small"},
      { 4, "Medium"},
      { 5, "Large"},
      { 6, "Trade"},
      { 7, "Market"},
      { 8, "Depot"},
      { 9, "Outpost"},
      {10, "Base"},
      {11, "Citadel"},
    }
  },
  {
    ID = "ship_subtypes",
    name = "Ship Types",
    elems = {
      { 1, "Unknown"},
      { 2, "Reserved"},
      { 3, "Solo"},
      { 4, "Small"},
      { 5, "Compact"},
      { 6, "Medium"},
      { 7, "Large"},
      { 8, "Very Large"},
      { 9, "Fighter"},
      {10, "Corvette"},
      {11, "Frigate"},
      {12, "Destroyer"},
      {13, "Cruiser"},
      {14, "Battleship"},
      {15, "Courier"},
      {16, "Trader"},
      {17, "Merchanter"},
      {18, "Freighter"},
      {19, "Bulk Freighter"},
      {20, "FreighterMax"},
      {21, "Miner"},
      {22, "Prospector"},
      {23, "Digger"},
      {24, "Driller"},
      {25, "Dredger"},
      {26, "Excavator"},
      {27, "Scout"},
      {28, "Ranger"},
      {29, "Seeker"},
      {30, "Explorer"},
      {31, "Wayfinder"},
      {32, "Surveyor"},
      {33, "Boat"},
      {34, "Runabout"},
      {35, "Cabin Cruiser"},
      {36, "Sloop"},
      {37, "Yacht"},
      {38, "Liner"},
      {33, "Marauder"},
      {34, "Smuggler"},
    }
  },
  {
    ID = "colony_subtypes",
    name = "Colony Types",
    elems = {
      { 1, "Unknown"},
      { 2, "Reserved"},
      { 3, "Outpost"},
      { 4, "Military Base"},
      { 5, "Manufacturing"},
      { 6, "Trading"},
      { 7, "Research"},
      { 8, "Breeding"},
      { 9, "Consulate"},
    }
  },
  {
    ID = "reserved_subtypes",
    name = "Reserved Types",
    elems = {
      { 1, "Unknown"},
      { 2, "Reserved"},
      { 3, "?"},
      { 4, "!"},
    }
  },
  {
    ID = "1_subtypes",
    name = "1 Subtypes",
    elems = {
      { 1, "Unknown"},
      { 2, ""},
      { 3, ""},
      { 4, ""},
      { 5, ""},
      { 6, ""},
    }
  },
}

function Config:getObjectTypeByName(objIDname, objtypename)
  -- For a given kind of object (by ID), find the index of the object type provided
  local objIDnum = Config:getObjectTypeIndex(objIDname)

  return Config:getObjectTypeByIDVal(objIDnum, objtypename)
end

function Config:getObjectTypeIndex(objIDname)
  -- For a given kind of object (by ID name), find the index of the object type provided
  local objIDnum = 1 -- default is "Unknown"

  -- Find index number of given object ID in the object types table
  for i = 1, #Config.objectInfo do
    if string.match(objIDname, Config.objectInfo[i].ID) then
      objIDnum = i
      break
    end
  end

  return objIDnum
end

function Config:getObjectTypeByIDVal(objIDnum, objtypename)
  -- For a given kind of object (by ID number), find the index of the object type provided
  local objtype = 1 -- default is "Unknown"

  if objIDnum > 0 then
    -- Scan object types table for match against provided object's type
    -- Return number of object type if found
    for i = 1, #Config.objectInfo[objIDnum]["elems"] do
      if string.match(objtypename, Config.objectInfo[objIDnum]["elems"][i][2]) then
        objtype = Config.objectInfo[objIDnum]["elems"][i][1]
        break
      end
    end
  end

  return objtype
end

function Config:getObjectInfo(objIDname, objtypenum)
  if Config.objectInfo[Config:getObjectTypeIndex(objIDname)] then
    if Config.objectInfo[Config:getObjectTypeIndex(objIDname)]["elems"][objtypenum] then
      return Config.objectInfo[Config:getObjectTypeIndex(objIDname)]["elems"][objtypenum][2]
    end
  end
end

function Config:getObjectSubInfo(objIDname, objtypenum, objsubtypenum)
  if Config.objectInfo[Config:getObjectTypeIndex(objIDname)] then
    if Config.objectInfo[Config:getObjectTypeIndex(objIDname)]["elems"][objtypenum] then
      local subtypename = Config.objectInfo[Config:getObjectTypeIndex(objIDname)]["elems"][objtypenum][3]
      return Config.objectInfo[Config:getObjectTypeIndex(subtypename)]["elems"][objsubtypenum][2]
    end
  end
end

function Config.getCurrentTimestamp()
  return os.time(os.date("!*t"))
end
