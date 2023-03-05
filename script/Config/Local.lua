Config.debug.instantJobs = true
Config.debug.jobSpeed = 10000

Config.debug.window = false
Config.debug.metrics = false
Config.ui.showTrackers = true

Config.render.vsync = false

Config.audio.bSoundOn = false

Config.gen.scaleFieldAsteroid = Config.gen.scaleAsteroid * 9500
Config.gen.nBeltSize = function (rng) return 200 end
Config.gen.nNPCs = 100
Config.gen.nFields = 3
Config.gen.nPlanets = 1
Config.gen.nStations = 6
Config.gen.nTurrets = 1
Config.gen.nThrusters = 2
-- Config.gen.nDustClouds = 0
-- Config.gen.nDustFlecks = 2048

Config.ui.uniqueShips = false -- true = generate each ship as a unique mesh (very slow!)

if false then
  Config.jit.loom = true
  Config.jit.profile = false
  Config.jit.verbose = false
end
