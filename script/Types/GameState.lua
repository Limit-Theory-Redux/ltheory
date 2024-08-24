-- For dynamic states
GameState = {
    state           = Enums.GameStates.Splashscreen, -- previously gamemode
    paused          = false,
    panelActive     = false,                         -- indicates whether MasterControl panel is enabled or not
    skipTitleScreen = false,
}

GameState.input = {
    invertPitch = false,
}

GameState.debug = {
    metricsEnabled     = Config.debug.metricsEnabled,
    instantJobs        = Config.debug.instantJobs,
    jobSpeed           = Config.debug.jobSpeed,
    timeAccelFactor    = Config.debug.timeAccelFactor,
    printConfig        = Config.debug.printConfig,
    showMapActionLines = Config.debug.showMapActionLines,

    physics            = {
        drawWireframes         = Config.debug.physics.drawWireframes,
        drawBoundingBoxesLocal = Config.debug.physics.drawBoundingBoxesLocal,
        drawBoundingBoxesworld = Config.debug.physics.drawBoundingBoxesworld,
    }
}

GameState.render = {
    fullscreen          = Config.render.general.fullscreen,
    fullscreenExclusive = Config.render.general.fullscreenExclusive,
    gameWindow          = nil,
    gameView            = nil,
    uiCanvas            = nil,
    resX                = Config.render.window.defaultResX,
    resY                = Config.render.window.defaultResY,
    presentMode         = Config.render.general.presentMode,
    fov                 = Config.render.camera.fov,
    zNear               = Config.render.camera.zNear,
    zFar                = Config.render.camera.zFar,
    thrusterLights      = Config.render.general.thrusterLights,
    pulseLights         = Config.render.general.pulseLights,
    renderDistances     = {
        Ship = 10000,
        Station = 100000
    }
}

GameState.audio = {
    manager      = nil,
    soundEnabled = Config.audio.general.soundEnabled,
    fxVolume     = Config.audio.general.fxVolume,
    musicVolume  = Config.audio.general.musicVolume,
    menuTheme    = Config.audio.general.mainMenu
}

GameState.ui = {
    controlBarHeight                 = Config.ui.general.controlBarHeight,
    hudStyle                         = Config.ui.general.hudStyle,
    cursorStyle                      = Config.ui.general.cursorStyle,
    cursorX                          = Config.ui.general.cursorX,
    cursorY                          = Config.ui.general.cursorY,
    sensorsDisplayed                 = Config.ui.general.sensorsDisplayed,

    scaleFactor                      = Config.ui.general.scaleFactor,

    -- Trackers
    showTrackers                     = Config.ui.general.showTrackers,
    maxTrackingRange                 = Config.ui.general.maxTrackingRange,
    trackerBracketingRenderDistances = {
        Planet   = Config.ui.general.trackerBracketingRenderDistances.Planet,
        Asteroid = Config.ui.general.trackerBracketingRenderDistances.Asteroid,
        Jumpgate = Config.ui.general.trackerBracketingRenderDistances.Jumpgate,
        Station  = Config.ui.general.trackerBracketingRenderDistances.Station,
        Ship     = Config.ui.general.trackerBracketingRenderDistances.Ship,
        Colony   = Config.ui.general.trackerBracketingRenderDistances.Colony,
    },
    trackerObjectOcclusion           = Config.ui.general.trackerObjectOcclusion,

    tacMapRange                      = Enums.HudTacMapRanges.Normal,

    mapSystemPanSpeed                = 0.5,
    mapSystemZoomSpeed               = 0.1,

    backgroundClockEnabled           = false
}

GameState.player = {
    humanPlayer          = nil,
    humanPlayerName      = "[Human Player Name]",
    humanPlayerShipName  = "[Human Player Ship Name]",

    playerFactionName    = "[Human Player Faction]",

    currentControl       = Config.ui.general.defaultControl,

    currentShip          = nil,
    shipHull             = Enums.ShipHulls.Solo,
    weaponGroup          = 1,

    currentCamera        = Enums.CameraMode.FirstPerson,
    lastCamera           = nil,
    startupCamera        = Enums.CameraMode.FirstPerson,

    currentMapSystemPos  = Vec3f(0, 0, 0),
    currentMapSystemZoom = 0.001,
    currentMapSystemPan  = 40.0,

    autonavActive        = false,
}

GameState.world = {
    -- TODO: World related states here later (system state, ai, economy etc)
    currentUniverse = nil,
    currentSystem = nil,
}

GameState.gen = {
    debug                 = Config.gen.debug,
    nFields               = Config.gen.nFields,
    nFieldsMax            = Config.gen.nFieldsMax,
    nAsteroids            = Config.gen.nAsteroids,
    nPlanets              = Config.gen.nPlanets,
    nPlanetsMax           = Config.gen.nPlanetsMax,
    nStations             = Config.gen.nStations,
    nAIPlayers            = Config.gen.nAIPlayers,
    randomizeAIPlayers    = Config.gen.randomizeAIPlayers,
    nEconNPCs             = Config.gen.nEconNPCs,
    randomizeEconNPCs     = Config.gen.randomizeEconNPCs,
    nEscortNPCs           = Config.gen.nEscortNPCs,
    randomizeEscortNPCs   = Config.gen.randomizeEscortNPCs,
    nPirateNPCs           = Config.gen.nPirateNPCs,
    randomizePirateNPCs   = Config.gen.randomizePirateNPCs,
    uniqueShips           = Config.gen.uniqueShips,
    nebulaBrightnessScale = Config.gen.nebulaBrightnessScale
}

function GameState:SetState(state)
    self.state = state

    if self.state == Enums.GameStates.MainMenu or self.state == Enums.GameStates.Splashscreen then
        self.player.currentControl = Enums.ControlModes.Background -- enable game startup mode
    else
        self.player.currentControl = Enums.ControlModes.Ship       -- enable flight mode
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
