Config.debug.window = false
Config.debug.metrics = false
Config.ui.showTrackers = true

Config.render.vsync = false

Config.audio.bSoundOn = false

Config.gen.nFields    =   5
Config.gen.nAsteroids = 100
Config.gen.nPlanets   =   1
Config.gen.nStations  =  40

Config.gen.scaleFieldAsteroid = Config.gen.scaleAsteroid * 9500
Config.gen.nBeltSize = function (rng) return 200 end

Config.ui.uniqueShips = false -- true = generate each ship as a unique mesh (very slow!)

Config.gen.nTurrets   = 1
Config.gen.nThrusters = 2

Config.gen.nAIPlayers  =   3 -- # of AI players (who manage Economic assets)
Config.gen.nEconNPCs   =  45 -- total # of ships to be given Economic actions (to be split among AI players)
Config.gen.nEscortNPCs =   0 -- # of ships to be given the Escort action

Config.debug.instantJobs = false
Config.debug.jobSpeed    = 10000

if false then
  Config.jit.loom = true
  Config.jit.profile = false
  Config.jit.verbose = false
end
