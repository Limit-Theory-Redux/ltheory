Config.app = 'LTheoryRedux'

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
  currentVersion = "v0.003",

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

  autoTarget             = false,
  pulseDamage            = 5,
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
  playerMoving           = false,
  autonavTimestamp       = nil,

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

  autonavRanges          = {  200,  -- Unknown
                              100,  -- Ship
                              200,  -- Asteroid
                             1000,  -- Station
                             2000,  -- Zone
                            50000}, -- Planet

  dockRange              = 50,
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

Config.render = {
  startingHorz = 1600,
  startingVert =  900,
  fullscreen   = false,
  vsync        = true,
}

Config.ui = {
  defaultControl   = 'Ship', -- enable flight mode as default so that LTheory.lua still works
  showTrackers     = true,
  controlBarHeight = 48
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
}

Config.ui.font = {
  normal     = Cache.Font('Share', 14),
  normalSize = 14,
  title      = Cache.Font('Exo2Bold', 10),
  titleSize  = 10,
}

Config.objectInfo = {
  {
    ID = "object_types",
    name = "Object Types",
    elems = {
      { 1, "Unknown"},
      { 2, "Ship"},
      { 3, "Asteroid"},
      { 4, "Station"},
      { 5, "Zone"},
      { 6, "Planet"},
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
}

function Config:getObjectType(objname)
  local objtype = 1

  -- Scan object types table for match against provided object's type
  -- Return number of object type if found
  for i = 1, #Config.objectInfo[1]["elems"] do
    if string.match(objname, Config.objectInfo[1]["elems"][i][2]) then
      objtype = Config.objectInfo[1]["elems"][i][1]
      break
    end
  end

  return objtype
end

function Config.getCurrentTimestamp()
  return os.time(os.date("!*t"))
end
