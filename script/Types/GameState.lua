-- For dynamic states
GameState = {
  state                 = Enums.GameStates.Splashscreen, -- previously gamemode
  paused                = false,
  panelActive           = false, -- indicates whether MasterControl panel is enabled or not
}

GameState.input = {
  invertPitch           = false,
}

GameState.debug = {
  metricsEnabled        = Config.debug.metricsEnabled,
  instantJobs           = Config.debug.instantJobs,
  jobSpeed              = Config.debug.jobSpeed
}

GameState.render = {
  fullscreen            = Config.render.fullscreen,
  gameWindow            = nil,
  resX                  = Config.render.defaultResX,
  resY                  = Config.render.defaultResY,
  vsync                 = Config.render.vsync,
  zNear                 = Config.render.zNear,
  zFar                  = Config.render.zFar,
  thrusterLights        = Config.render.thrusterLights,
  pulseLights           = Config.render.pulseLights,
}

GameState.audio = {
  enabled               = Config.audio.enabled,
  fxVolume              = Config.audio.fxVolume,
  musicVolume           = Config.audio.musicVolume,
}

GameState.ui = {
  showTrackers          = Config.ui.showTrackers,
  controlBarHeight      = Config.ui.controlBarHeight,
  hudStyle              = Config.ui.hudStyle,
  cursorStyle           = Config.ui.cursorStyle,
  cursorX               = Config.ui.cursorX,
  cursorY               = Config.ui.cursorY,
  displaySensors        = Config.ui.displaySensors
}

GameState.player = {
  humanPlayer           = nil,
  humanPlayerName       = "[Human Player Name]",
  humanPlayerShipName   = "[Human Player Ship Name]",

  currentControl        = Config.ui.defaultControl,
  playerMoving          = false,

  currentShip           = nil,
  weaponGroup           = 1,

  mapSystemPos          = Vec3f(0, 0, 0),
  mapSystemZoom         = 0.001,

  autonavTimestamp      = nil,
}

GameState.world = {
  -- world related states here later (system state, ai, economy etc)
  currentSystem         = nil,
}

GameState.gen = {
  nFields               = Config.gen.nStations,
  nAsteroids            = Config.gen.nAsteroids,
  nPlanets              = Config.gen.nPlanets,
  nStations             = Config.gen.nStations,
  nAIPlayers            = Config.gen.nAIPlayers,
  randomizeAIPlayers    = Config.gen.randomizeAIPlayers,
  nEconNPCs             = Config.gen.nEconNPCs,
  randomizeEconNPCs     = Config.gen.randomizeEconNPCs,
  nEscortNPCs           = Config.gen.nEscortNPCs,
  randomizeEscortNPCs   = Config.gen.randomizeEscortNPCs,
  uniqueShips           = Config.gen.uniqueShips
}

function GameState:SetState(state)
  self.state = state

  if self.state == Enums.GameStates.MainMenu or self.state == Enums.GameStates.Splashscreen then
    self.ui.currentControl = "Background" -- enable game startup mode
  else
    self.ui.currentControl = "Ship" -- enable flight mode
  end
end

function GameState:GetCurrentState()
  return self.state
end

function GameState:Pause()
  self.paused = true
end

function GameState:Unpause()
  self.paused = false
end