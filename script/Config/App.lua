Config.app = 'LTheoryRedux'

Config.gameTitle   = "Limit Theory Redux"
Config.gameVersion = "v0.008"

Config.render = {
  startingHorz   = 1600, -- 1600 (default), or 2400 (high DPI)
  startingVert   =  900, --  900 (default), or 2048 (high DPI)
  fullscreen     = false,
  vsync          = true,
  zNear          = 0.1, -- default: 0.1
  zFar           = 1e8, -- default: 1e6
  thrusterLights = false,
  pulseLights    = false,
}

Config.audio = {
  bSoundOn  = false,
  soundMin  = 0,
  soundMax  = 1, -- SetVolume range seems to go from 0 (min) to about 2 or 3 (max)
}

Config.paths = {
  soundAmbiance = "./res/sound/system/ambiance/",
}

Config.debug = {
  metrics         = true,
  window          = true, -- Debug window visible by default at launch?
  windowSection   = nil,  -- Set to the name of a debug window section to
                          -- collapse all others by default

  instantJobs     = false, -- set to true to speed up economic testing
  jobSpeed        = 10000, -- acceleration rate for instant jobs (in MineAt, DockAt)

  timeAccelFactor = 10, -- acceleration rate when holding "TimeAccel" input
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
  nStations  = 0,
  nPlanets   = 0,
  nAsteroids = 200, -- asteroids per asteroid field (smaller = less CPU hit)
  nBeltSize  = function (rng) return 0 end, -- asteroids per planetary belt

  nDustFlecks = 1024,
  nDustClouds = 1024,
  nStars      = function (rng) return 30000 * (1.0 + 0.5 * rng:getExp()) end,

  shipRes     = 8,
  nebulaRes   = 1024,

  nAIPlayers  = 0,  -- # of AI players (who manage Economic assets)
  nEconNPCs   = 0,  -- # of ships to be given Economic actions (managed by AI players)
  nEscortNPCs = 0,  -- # of ships to be given the Escort action

  playerShipSize = 4,
  nThrusters     = 1,
  nTurrets       = 2,

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
  scaleFieldAsteroid = 10000, -- overwritten in Local.lua
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
}

Config.game = {
  gameMode = 0, -- used by LTheoryRedux: 0 = undefined (splash screen), 1 = Startup Mode (Main Menu), 2 = Flight Mode
  flightModeButInactive = false, -- flag for being in Flight Mode but unable to fly (as when player ship is destroyed)

  gamePaused = false,

  humanPlayer         = nil,
  humanPlayerName     = "[Human Player Name]",
  humanPlayerShipName = "[Human Player Ship Name]",

  currentShip   = nil,
  currentSystem = nil,

  mapSystemPos  = Vec3f(0, 0, 0),
  mapSystemZoom = 0.0001,

  boostCost = 10,
  rateOfFire = 10,

  explosionSize          = 64,

  autoTarget             = false,
  pulseDamage            = 2,
  pulseSize              = 64,
  pulseSpeed             = 1e3, -- was 6e2
  pulseRange             = 1000,
  pulseSpread            = 0.01,
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
  shipHealthRegen        = 2,

  playerDamageResistance = 1.0,

  enemies                = 0,
  friendlies             = 0,
  squadSizeEnemy         = 8,
  squadSizeFriendly      = 8,
  spawnDistance          = 2000,
  friendlySpawnCount     = 10,
  timeScaleShipEditor    = 0.0,
  invertPitch            = false,

  aiFire                 = function (dt, rng) return rng:getExp() ^ 2 < dt end,

  playerMoving           = false,
  autonavTimestamp       = nil,
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
  pStartCredits = 10000,   -- player starting credits
  eStartCredits = 1000000, -- NPC player starting credits

  eInventory = 100, -- starting number of inventory slots

  jobIterations = 4000, -- how many randomly-chosen jobs an asset will consider before picking

  inputBacklog = 1, -- multiplier of number of units a factory can bid for on each input

  pickupDistWeightMine = 1.0, -- importance of pickup distance for a Mine job (smaller = more important)
  pickupDistWeightTran = 3.0, -- importance of pickup distance for a Transport job (smaller = more important)
  markup   = 1.2, -- change to base value when calculating ask price for selling an item
  markdown = 0.8, -- change to base value when calculating bid price for buying an item
}

Config.ui = {
  defaultControl   = 'Ship', -- enable flight mode as default so that LTheory.lua still works
  showTrackers     = true,
  controlBarHeight = 48,
  HUDdisplayed     = true,
  uniqueShips      = false,
}

Config.ui.color = {
                      --     R     G     B     A
  accent            = Color(1.00, 0.00, 0.30, 1.0),
  focused           = Color(1.00, 0.00, 0.30, 1.0),
  active            = Color(0.70, 0.00, 0.21, 1.0),
  background        = Color(0.15, 0.15, 0.15, 1.0),
  backgroundInvert  = Color(0.85, 0.85, 0.85, 1.0),
  border            = Color(0.12, 0.12, 0.12, 1.0),
  fill              = Color(0.60, 0.60, 0.60, 1.0),
  textNormal        = Color(0.75, 0.75, 0.75, 1.0),
  textNormalFocused = Color(0.00, 0.00, 0.00, 1.0),
  textInvert        = Color(0.25, 0.25, 0.25, 1.0),
  textInvertFocused = Color(0.00, 0.00, 0.00, 1.0),
  textTitle         = Color(0.60, 0.60, 0.60, 1.0),
  debugRect         = Color(0.50, 1.00, 0.50, 0.05),
  selection         = Color(1.00, 0.50, 0.10, 1.0),
  control           = Color(0.20, 0.60, 1.00, 0.3),
  controlFocused    = Color(0.20, 1.00, 0.20, 0.4),
  controlActive     = Color(0.14, 0.70, 0.14, 0.4),
  hologram          = Color(0.30, 0.40, 1.00, 0.8),
  borderBright      = Color(1.00, 1.00, 1.00, 0.6),

  healthColor = {
    --     R    G    B    A
    Color(0.0, 0.0, 0.0, 0.8), --  0% -   1% BLACK
    Color(0.1, 0.0, 0.0, 0.6), --  2% -   3%
    Color(0.3, 0.0, 0.1, 0.4), --  4% -   5%
    Color(0.5, 0.0, 0.3, 0.3), --  6% -   7%
    Color(0.6, 0.0, 0.5, 0.2), --  8% -   9%
    Color(0.7, 0.0, 0.7, 0.2), -- 10% -  11%
    Color(0.8, 0.0, 0.8, 0.2), -- 12% -  13% PURPLE
    Color(0.9, 0.0, 0.7, 0.2), -- 14% -  15%
    Color(1.0, 0.0, 0.5, 0.2), -- 16% -  17%
    Color(1.0, 0.0, 0.3, 0.2), -- 18% -  19%
    Color(1.0, 0.1, 0.2, 0.2), -- 20% -  21%
    Color(1.0, 0.1, 0.1, 0.2), -- 22% -  23%
    Color(1.0, 0.1, 0.0, 0.2), -- 24% -  25% RED
    Color(0.9, 0.2, 0.0, 0.2), -- 26% -  27%
    Color(0.9, 0.2, 0.0, 0.2), -- 28% -  29%
    Color(0.8, 0.3, 0.0, 0.2), -- 30% -  31%
    Color(0.8, 0.3, 0.0, 0.2), -- 32% -  33%
    Color(0.7, 0.3, 0.1, 0.2), -- 34% -  35%
    Color(0.7, 0.4, 0.1, 0.2), -- 36% -  37%
    Color(0.6, 0.4, 0.1, 0.2), -- 38% -  39% ORANGE
    Color(0.6, 0.5, 0.1, 0.2), -- 40% -  41%
    Color(0.6, 0.5, 0.1, 0.2), -- 42% -  43%
    Color(0.7, 0.5, 0.1, 0.2), -- 44% -  45%
    Color(0.7, 0.5, 0.2, 0.2), -- 46% -  47%
    Color(0.7, 0.6, 0.2, 0.2), -- 48% -  49%
    Color(0.7, 0.6, 0.2, 0.2), -- 50% -  51%
    Color(0.7, 0.7, 0.3, 0.2), -- 52% -  53%
    Color(0.8, 0.7, 0.3, 0.2), -- 54% -  55%
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

function Config.setGameMode(gm)
  Config.game.gameMode = gm

  if Config.game.gameMode == 1 then
    Config.ui.defaultControl = 'Background' -- enable game startup mode
  else
    Config.ui.defaultControl = 'Ship' -- enable flight mode
  end
end

function Config.getGameMode()
  return Config.game.gameMode
end

Config.objectInfo = {
  {
    ID = "object_types",
    name = "Object Types",
    elems = {
      -- NOTE: If you change these, you must also change autonavRanges!
      { 1, "Unknown", ""},
      { 2, "Reserved", ""},
      { 3, "Star Sector", ""},
      { 4, "Star System", ""},
      { 5, "Zone", "zone_subtypes"},
      { 6, "Star", "star_subtypes"},
      { 7, "Planet", "planet_subtypes"},
      { 8, "Asteroid", "asteroid_subtypes"},
      { 9, "Jumpgate", "jumpgate_subtypes"},
      {10, "Station", "station_subtypes"},
      {11, "Ship", "ship_subtypes"},
      {12, "Colony", "colony_subtypes"},
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
    }
  },
  {
    ID = "ship_subtypes",
    name = "Ship Types",
    elems = {
      { 1, "Unknown"},
      { 2, "Reserved"},
      { 3, "Fighter"},
      { 4, "Corvette"},
      { 5, "Frigate"},
      { 6, "Monitor"},
      { 7, "Destroyer"},
      { 8, "Cruiser"},
      { 9, "Battleship"},
      {10, "Battlecruiser"},
      {11, "Carrier"},
      {12, "Yacht"},
      {13, "Liner"},
      {14, "Scout"},
      {15, "Laboratory"},
      {16, "Merchanter"},
      {17, "Miner"},
      {18, "Tanker"},
      {19, "Transport"},
      {20, "Ferry"},
      {21, "Tug"},
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
  return Config.objectInfo[Config:getObjectTypeIndex(objIDname)]["elems"][objtypenum][2]
end

function Config:getObjectSubInfo(objIDname, objtypenum, objsubtypenum)
  local subtypename = Config.objectInfo[Config:getObjectTypeIndex(objIDname)]["elems"][objtypenum][3]
  return Config.objectInfo[Config:getObjectTypeIndex(subtypename)]["elems"][objsubtypenum][2]
end

function Config.getCurrentTimestamp()
  return os.time(os.date("!*t"))
end
