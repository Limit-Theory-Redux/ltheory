Config.app = 'LTheoryRedux'

Config.version = "v0.005"

Config.debug = {
  metrics         = true,
  window          = true, -- Debug window visible by default at launch?
  windowSection   = nil,  -- Set to the name of a debug window section to
                          -- collapse all others by default

  instantJobs     = true,

  timeAccelFactor = 10,
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
  nNPCs      = 0,
  nNPCsNew   = 0,
  nPlanets   = 0,
  nBeltSize  = function (rng) return 0 end, -- Asteroids per planetary belt
  nThrusters = 1,
  nTurrets   = 2,

  nDustFlecks = 1024,
  nDustClouds = 1024,
  nStars      = function (rng) return 30000 * (1.0 + 0.5 * rng:getExp()) end,

  shipRes     = 8,
  nebulaRes   = 1024,

  scalePlanet = 2000,
  playerShipSize = 4,
}

Config.game = {
  gameMode = 0,

  currentShip = nil,
  currentStation = nil,
  currentPlanet = nil,
  currentZone = nil,

  mapSystemPos  = Vec2f(0, 0),
  mapSystemZoom = 0.01,

  pStartCredits = 10000,
  eStartCredits = 100000,

  eInventory = 100,

  boostCost = 10,
  rateOfFire = 10,

  explosionSize          = 64,

  autoTarget             = false,
  pulseDamage            = 2,
  pulseSize              = 64,
  pulseSpeed             = 6e2,
  pulseRange             = 1000,
  pulseSpread            = 0.01,

  shipBuildTime          = 10,
  shipEnergy             = 100,
  shipEnergyRecharge     = 10,
  shipHealth             = 100,
  shipHealthRegen        = 2,
  shipDocked             = false,

  stationScale           = 20,

  playerDamageResistance = 1.0,

  enemies                = 0,
  friendlies             = 0,
  squadSizeEnemy         = 8,
  squadSizeFriendly      = 8,
  spawnDistance          = 2000,
  friendlySpawnCount     = 10,
  timeScaleShipEditor    = 0.0,
  invertPitch            = false,

  aiUsesBoost            = true,
  aiFire                 = function (dt, rng) return rng:getExp() ^ 2 < dt end,

  playerMoving           = false,
  autonavTimestamp       = nil,
  autonavRanges          = {  200,  -- Unknown
                                0,  -- Reserved
                                0,  -- Star Sector
                                0,  -- Star System
                             2000,  -- Zone
                            50000,  -- Planet
                              200,  -- Asteroid
                             1000,  -- Station
                              100}, -- Ship

  dockRange              = 50,
}

Config.render = {
  startingHorz = 1600,
  startingVert =  900,
  fullscreen   = false,
  vsync        = true,
}

Config.ui = {
  defaultControl   = 'Ship', -- enable flight mode as default so that LTheory.lua still works
  showTrackers     = true,
  controlBarHeight = 48,
  HUDdisplayed     = false,
}

Config.ui.color = {
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

  healthColor = {
    Color(0.8, 0.0, 0.6, 1.0), --  0% -   4% FULL PURPLE
    Color(0.9, 0.0, 0.3, 1.0), --  5% -   9%
    Color(1.0, 0.0, 0.1, 1.0), -- 10% -  14%
    Color(1.0, 0.0, 0.0, 1.0), -- 15% -  19% FULL RED
    Color(1.0, 0.2, 0.0, 1.0), -- 20% -  24%
    Color(1.0, 0.4, 0.0, 1.0), -- 25% -  29%
    Color(0.9, 0.6, 0.1, 1.0), -- 30% -  34%
    Color(0.8, 0.7, 0.2, 1.0), -- 35% -  39% FULL ORANGE
    Color(0.8, 0.7, 0.1, 1.0), -- 40% -  44%
    Color(0.7, 0.7, 0.0, 1.0), -- 45% -  49%
    Color(0.7, 0.8, 0.0, 1.0), -- 50% -  54% FULL YELLOW
    Color(0.6, 0.8, 0.0, 1.0), -- 55% -  59%
    Color(0.6, 0.8, 0.0, 1.0), -- 60% -  64%
    Color(0.5, 0.8, 0.0, 1.0), -- 65% -  69%
    Color(0.4, 0.8, 0.0, 1.0), -- 70% -  74%
    Color(0.3, 0.9, 0.0, 1.0), -- 75% -  79% OLIVE?
    Color(0.2, 0.9, 0.0, 1.0), -- 80% -  84%
    Color(0.1, 0.9, 0.0, 1.0), -- 85% -  89%
    Color(0.0, 1.0, 0.0, 1.0), -- 90% -  94%
    Color(0.0, 1.0, 0.0, 1.0), -- 95% - 100% FULL GREEN
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
      { 1, "Unknown"},
      { 2, "Reserved"},
      { 3, "Star Sector"},
      { 4, "Star System"},
      { 5, "Zone"},
      { 6, "Planet"},
      { 7, "Asteroid"},
      { 8, "Station"},
      { 9, "Ship"},
    }
  },
  {
    ID = "planet_types",
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
    ID = "zone_subtypes",
    name = "Zone Subtypes",
    elems = {
      { 1, "Unknown"},
      { 2, "Asteroid Field"},
      { 3, "Political Extent"},
      { 4, "Military Extent"},
      { 5, "Economic Extent"},
      { 6, "Cultural Extent"},
    }
  },
  {
    ID = "ship_types",
    name = "Ship Types",
    elems = {
      { 1, "Unknown"},
      { 2, "Fighter"},
      { 3, "Corvette"},
      { 4, "Frigate"},
      { 5, "Monitor"},
      { 6, "Destroyer"},
      { 7, "Cruiser"},
      { 8, "Battleship"},
      { 9, "Battlecruiser"},
      {10, "Carrier"},
      {11, "Yacht"},
      {12, "Liner"},
      {13, "Scout"},
      {14, "Laboratory"},
      {15, "Merchanter"},
      {16, "Miner"},
      {17, "Tanker"},
      {18, "Transport"},
      {19, "Ferry"},
      {20, "Tug"},
    }
  },
  {
    ID = "station_types",
    name = "Station Types",
    elems = {
      { 1, "Unknown"},
      { 2, "Solar Energy Array"},
      { 3, "Nuclear Reactor"},
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
  local objIDnum = 0 -- default is "not found"

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

function Config.getCurrentTimestamp()
  return os.time(os.date("!*t"))
end
