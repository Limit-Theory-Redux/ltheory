Config.debug.instantJobs = true
Config.debug.jobSpeed = 10000

Config.debug.window = false
Config.debug.metrics = true
Config.ui.showTrackers = true

Config.render.vsync = false

Config.audio.bSoundOn = true

-- Config.gen.nBeltSize = function (rng) return 10000 end
Config.gen.scalePlanet = 5e3
Config.gen.nNPCs = 100
Config.gen.nFields = 1
Config.gen.nPlanets = 1
Config.gen.nStations = 2
Config.gen.nTurrets = 1
Config.gen.nThrusters = 2
-- Config.gen.nDustClouds = 0
-- Config.gen.nDustFlecks = 2048

Config.ui.uniqueShips = false -- generate each ship as a unique mesh? (very slow!)

if false then
  Config.jit.loom = true
  Config.jit.profile = false
  Config.jit.verbose = false
end
