--** REQUIRES **--
local Player = require('GameObjects.Entities.Player')
local System = require('GameObjects.Entities.Test.System')
local DebugControl = require('Systems.Controls.Controls.DebugControl')
local Bindings = require('States.ApplicationBindings')
local ShipBindings = require('Systems.Controls.Bindings.ShipBindings')
local Actions = requireAll('GameObjects.Actions')
local Production = require('Systems.Economy.Production')
local Item = require('Systems.Economy.Item')
local SocketType = require('GameObjects.Entities.Ship.SocketType')

local LTheoryRedux = require('States.Application')

--** LOCAL VARIABLES **--
local newSound = nil
local newSeed = 0ULL
local newShip = nil
local menuMode = 0 -- 0 = splash screen, 1 = Main Menu, 2 = either Flight dialog or Seed dialog (New Game / Load Game menus TBD)
local bNewSSystem = false
local bSeedDialogDisplayed = false
local bBackgroundMode = false
local bShowSystemMap = false
local bSMapAdded = false
local smap = nil

local rng = RNG.FromTime()


local guiElements = {
  {
    name = "Choose Seed",
    elems = {
      { nil, 5022463494542550306ULL,  false },  -- KEEP black
      { nil, 5012768293123392491ULL,  false },  -- KEEP red
      { nil, 4933876146649964811ULL,  false },  -- KEEP blue and milky white
      { nil, 2008422628673393673ULL,  false },  -- MAYBE orange-ish
      { nil, 5712598467986491931ULL,  false },  -- KEEP gold-yellow
      { nil, 14169804077813660835ULL, false },  -- KEEP bluish-green with a bright gold star
      { nil, 9806676695553338612ULL,  false },  -- KEEP violet
      { nil, 14600758714913275339ULL, false },  -- KEEP blue
      { nil, 11589761683708427350ULL, false },  -- KEEP bright green
      { nil, 3432712644463072838ULL,  false },  -- KEEP blue-red-orange
      { nil, 10630444862697458122ULL, false },  -- MAYBE "Hubble palette"
      { nil, 5199604093543988311ULL,  false },  -- KEEP even bluish-white with a bright yellow star
      { nil, 9471911754066691691ULL,  false },  -- KEEP completely dark with one small blue star
    }
  }
}




--** MAIN CODE **--
function LTheoryRedux:onInit ()
  self.logo = Tex2D.Load("./res/images/LTR_logo2.png") -- load the LTR logo

  DebugControl.ltheory = self

  self.player = Entities.Player(Config.game.humanPlayerName)
  Config.game.humanPlayer = self.player
  self:generate()

  --* Value initializations *--


  --* Audio initializations *--
  Audio.Init()
  Audio.Set3DSettings(0.0, 10, 2);

--  Config.audio.pulseFire = Sound.Load(Config.paths.soundEffects .. Config.audio.pulseFireName, true, false)
  if Config.audio.pulseFire then Sound.SetVolume(Config.audio.pulseFire, Config.audio.soundMax) end

  -- Music courtesy of MesoTroniK
  newSound = Sound.Load(Config.paths.soundAmbiance .. Config.audio.backLoop1, true, false)
  if Config.audio.bSoundOn then
    Sound.SetVolume(newSound, Config.audio.soundMax)
  else
    Sound.SetVolume(newSound, Config.audio.soundMin)
  end
  Sound.Play(newSound)
end

function LTheoryRedux:toggleSound ()
  if Config.audio.bSoundOn then
    Sound.SetVolume(newSound, Config.audio.soundMin)
    Config.audio.bSoundOn = false
  else
    Sound.SetVolume(newSound, Config.audio.soundMax)
    Config.audio.bSoundOn = true
  end
end

function LTheoryRedux:onInput ()
  self.canvas:input()
end

function LTheoryRedux:onDraw ()
  -- Check to see whether to draw the System Map or the game world onto the canvas
  if bShowSystemMap then
    if not bSMapAdded then
      self.canvas:remove(self.gameView)
      self.canvas:add(smap)
      bSMapAdded = true
    end
  else
    if smap ~= nil then
      self.canvas:remove(smap)
      self.canvas:add(self.gameView)
      bSMapAdded = false
      smap = nil
    end
  end

  self.canvas:draw(self.resX, self.resY)

  HmGui.Draw() -- draw controls
end

function LTheoryRedux:onUpdate (dt)
  self.player:getRoot():update(dt)
  self.canvas:update(dt)

  -- TODO: Confirm whether this is still needed
  local playerShip = self.player
  if playerShip ~= nil then
    playerShip = Config.game.currentShip
  end

  -- Take down splash text if pretty much any key is pressed
  if menuMode == 0 and Bindings.All:get() == 1 then
    bBackgroundMode = false
    menuMode = 1 -- show Main Menu
  end

  -- Add basic Game Control menu
  if menuMode ~= 0 and Input.GetPressed(Bindings.Escape) then
    bBackgroundMode = false
    if Config.getGameMode() == 1 then
      menuMode = 1 -- show Main Menu
    else
      -- First time here, menuMode should be 0 (just starting game), so don't pop up the Flight Mode dialog box
      -- After that, in active Flight Mode, do pop up the Flight Mode dialog box when the player presses ESC
      if menuMode == 0 then
        Config.game.flightModeButInactive = false
        menuMode = 2 -- show Flight Mode dialog
      elseif menuMode == 2 and not bSeedDialogDisplayed then
        Config.game.flightModeButInactive = true
      end
    end
  end

  -- If player pressed the "System Map" key in Flight Mode, toggle the system map's visibility
  if Input.GetPressed(Bindings.SystemMap) and menuMode == 2 then
    bShowSystemMap = not bShowSystemMap
    if smap == nil then
      smap = Systems.CommandView.SystemMap(self.system)
    end
  end

  -- Engage autopilot if we're in flight mode
  if Input.GetPressed(Bindings.AutoNav) and menuMode == 2 then
    if playerShip ~= nil then
      local target = playerShip:getTarget()
      if target == nil then target = self.focus end
      if not playerShip:isDestroyed() and playerShip:isShipDocked() == nil and target ~= nil and target ~= playerShip then
        if playerShip:getCurrentAction() == nil or not string.find(playerShip:getCurrentAction():getName(),"MoveTo") then
          -- Move undestroyed, undocked player ship to area of selected target
          local autodistance = Config.game.autonavRanges[target:getType()]
          Config.game.autonavTimestamp = Config.getCurrentTimestamp()
          Config.game.playerMoving = true -- must be set to true before pushing the MoveTo action
          playerShip:pushAction(Actions.MoveTo(target, autodistance))
        end
      end
    end
  end

  -- Disengage autopilot (require a 1-second delay, otherwise keypress turns autopilot on then off instantly)
  if Config.game.playerMoving then
    if Input.GetPressed(Bindings.AutoNav) and Config.getCurrentTimestamp() - Config.game.autonavTimestamp > 1 then
      Config.game.playerMoving = false
    end
  end

  -- If player pressed the "ToggleLights" key in Flight Mode, toggle dynamic lighting on/off
  -- NOTE: Performance is OK for just the player's ship, but adding many lit ships & pulses tanks performance
  if Input.GetPressed(Bindings.ToggleLights) and menuMode == 2 then
    Config.render.thrusterLights = not Config.render.thrusterLights
    Config.render.pulseLights    = not Config.render.pulseLights
  end

  -- Canvas overlays
  HmGui.Begin(self.resX, self.resY)
    if menuMode == 0 then
      LTheoryRedux:showGameLogo()
    elseif menuMode == 1 then
      if not bBackgroundMode then
        if bSeedDialogDisplayed then
          LTheoryRedux:showSeedDialog()
        else
          LTheoryRedux:showMainMenu()
        end
      end
    elseif menuMode == 2 then
      if Config.game.flightModeButInactive then
        Config.game.gamePaused = true
        LTheoryRedux:showFlightDialog()
      elseif bSeedDialogDisplayed then
        LTheoryRedux:showSeedDialog()
      end
    end
  HmGui.End()

  -- If player pressed the "new background" key and we're in startup mode, generate a new star system for a background
  if Input.GetPressed(Bindings.NewBackground) and menuMode == 1 then
    bNewSSystem = true
  end

  if bNewSSystem then
    bNewSSystem = false
    if newSeed ~= 0ULL then
      self.seed = newSeed
      newSeed = 0ULL
    else
      self.seed = rng:get64()
    end
    LTheoryRedux:createStarSystem()
  end

  -- If player pressed the "toggle audio" key, turn it off if it's on or on if it's off
  if Input.GetPressed(Bindings.ToggleSound) then
    LTheoryRedux:toggleSound()
  end
end

function LTheoryRedux:generate ()
  Config.setGameMode(1) -- start off in Startup Mode

  -- Use random seed for new background star system, and stay in "display game logo" startup mode
  LTheoryRedux:seedStarsystem(0)
end

function LTheoryRedux:seedStarsystem (changeMenuMode)
  self.seed = rng:get64()

  LTheoryRedux:createStarSystem()

  menuMode = changeMenuMode
end

function LTheoryRedux:createStarSystem ()
  if self.system then self.system:delete() end

  print("------------------------")
  if Config.getGameMode() == 1 then
    -- Use custom system generation sizes for a nice background star system
    Config.gen.scaleSystem    = Config.gen.scaleSystemBack
    Config.gen.scalePlanet    = Config.gen.scalePlanetBack
    Config.gen.scalePlanetMod = Config.gen.scalePlanetModBack
    Config.render.zNear       = Config.gen.zNearBack
    Config.render.zFar        = Config.gen.zFarBack
  else
    -- Use the "real" system generation sizes for a gameplay star system
    Config.gen.scaleSystem    = Config.gen.scaleSystemReal
    Config.gen.scalePlanet    = Config.gen.scalePlanetReal
    Config.gen.scalePlanetMod = Config.gen.scalePlanetModReal
    Config.render.zNear       = Config.gen.zNearReal
    Config.render.zFar        = Config.gen.zFarReal
  end

  -- Spawn a new star system
  self.system = System(self.seed)
  Config.game.currentSystem = self.system -- remember the player's current star system

  do
    if Config.getGameMode() == 1 then
      -- Background Mode
      -- Generate a new star system with nebulae/dust, a planet, an asteroid field,
      --   a space station, and an invisible rotating ship
      newShip = self.system:spawnBackground() -- spawn an invisible ship
      LTheoryRedux:insertShip(newShip)

      -- Add a planet
      for i = 1, 1 do
        local planet = self.system:spawnPlanet(false) -- no planetary asteroid belt
        local ppos = planet:getPos()
        ppos.x = ppos.x * 2
        ppos.y = ppos.y * 2
        planet:setPos(ppos) -- move planet away from origin for background
      end

      -- Add an asteroid field
      -- Must add BEFORE space stations
      for i = 1, rng:getInt(0, 1) do -- 50/50 chance of having asteroids
        self.system:spawnAsteroidField(-1, true) -- -1 is a special case meaning background
      end

      -- Add a space station
      local station = self.system:spawnStation(Config.game.humanPlayer, nil)
    else
      -- Flight Mode
      -- Generate a new star system with nebulae/dust, a planet, an asteroid field,
      --   a space station, a visible pilotable ship, and possibly some NPC ships
      local afield = nil

      -- Add system-wide AI director
      self.tradeAI = Entities.Player("AI Trade Player")
      self.tradeAI:addCredits(1e10)

      -- Add a generic ship-like entity to serve as the imaginary player ship
      self.tradeShip = Entity()
      self.tradeShip:setOwner(self.tradeAI)

      -- Add planets
      local planet = nil -- remember the last planet created (TODO: remember ALL the planets)
      for i = 1, Config.gen.nPlanets do
        planet = self.system:spawnPlanet(false)
      end

      -- Add asteroid fields
      -- Must add BEFORE space stations
      for i = 1, Config.gen.nFields do
        afield = self.system:spawnAsteroidField(Config.gen.nAsteroids, false)
        printf("Added %s asteroids to %s", Config.gen.nAsteroids, afield:getName())
      end

      -- Add space stations with random factories
      -- Every system gets one "free" solar plant
      local newStation = self.system:spawnStation(self.tradeAI, Production.EnergySolar)
      self.system:place(newStation)

      if Config.gen.nAIPlayers > 0 and Config.gen.nEconNPCs > 0 then
        -- Add the "extra" stations only if there are economic ships to use them
        -- Add a free Waste Recycler station
        newStation = self.system:spawnStation(self.tradeAI, Production.Recycler)
        self.system:place(newStation)
      end

      for i = 3, Config.gen.nStations do
        -- Create Stations within randomly selected AsteroidField Zones
        self.system:spawnStation(self.tradeAI, nil)
      end

      -- Possibly add some additional factory stations based on which ones were randomly created and their inputs
      self.system:addExtraFactories(self.system, Config.gen.nPlanets, self.tradeAI)

      -- Add the player's ship
      newShip = self.system:spawnShip(Config.game.humanPlayer)
      newShip:setName(Config.game.humanPlayerShipName)
      newShip:setHealth(500, 500, 10) -- make the player's ship healthier than the default NPC ship

      LTheoryRedux:insertShip(newShip)

      Config.game.currentShip = newShip

      -- Set our ship's starting location within the extent of a random asteroid field
      self.system:place(newShip)
printf("Added our ship, the '%s', at pos %s", newShip:getName(), newShip:getPos())

      -- TESTING: ADD SHIPS WITH ESCORT BEHAVIOR ENABLED
      local ships = {}
      for i = 1, Config.gen.nEscortNPCs do
        local escort = self.system:spawnShip(nil)
        local offset = self.system.rng:getSphere():scale(100)
        escort:setPos(newShip:getPos() + offset)

        escort:pushAction(Actions.Escort(newShip, offset))

        -- TEMP: a few NPC escort ships get to be "aces" with extra health and maneuverability
        --       These will be dogfighting challenges!
        if rng:getInt(0, 100) < 20 then
          escort:setHealth(100, 100, 0.2)
          escort.usesBoost = true
        end

        insert(ships, escort)
      end
if Config.gen.nEscortNPCs > 0 then
  printf("Added %d escort ships", Config.gen.nEscortNPCs)
end

      -- TESTING: MAKE SHIPS CHASE EACH OTHER!
      for i = 1, #ships - 1 do
        ships[i]:pushAction(Actions.Attack(ships[i+1]))
      end

      -- TESTING: ADD SHIPS WITH ECONOMIC BEHAVIOR ENABLED
      -- Add AI Players and give each one some assets
      if Config.gen.nAIPlayers > 0 and Config.gen.nEconNPCs > 0 then
        local econShipsPerAI = math.floor(Config.gen.nEconNPCs / Config.gen.nAIPlayers)
        local econShipsAdded = econShipsPerAI * Config.gen.nAIPlayers
        for i = 1, Config.gen.nAIPlayers do
          local tradePlayerName = format("AI Trade Player %d", i)
          local tradePlayer = Entities.Player(tradePlayerName)
          insert(self.system.players, tradePlayer)

          -- Give AI Player some starting money
          tradePlayer:addCredits(Config.econ.eStartCredits)

          -- Create multiple assets (ships) assigned to this AI Player
          self.system:spawnAI(econShipsPerAI, Actions.Wait(1), tradePlayer)
printf("%d assets added to %s", econShipsPerAI, tradePlayerName)
        end
printf("Added %d economic ships to %d AI players", econShipsAdded, Config.gen.nAIPlayers)

        for _, tradePlayer in ipairs(self.system.players) do
          -- Tell each AI player to start using the Think action
          tradePlayer:pushAction(Actions.Think())
        end
      end

    end
  end

  -- Insert the game view into the application canvas to make it visible
  self.gameView = Systems.Overlay.GameView(self.player)
  self.canvas = UI.Canvas()
  self.canvas
    :add(self.gameView
      :add(Systems.Controls.Controls.MasterControl(self.gameView, self.player))
    )
end

function LTheoryRedux:insertShip(ourShip)
  -- Insert ship into this star system
  ourShip:setPos(Config.gen.origin)
  ourShip:setFriction(0)
  ourShip:setSleepThreshold(0, 0)
  ourShip:setOwner(self.player)
  self.system:addChild(ourShip)
  self.player:setControlling(ourShip)
end

function LTheoryRedux:showGameLogo ()
  -- Draw the LTR game logo on top of the background star system
  local scaleFactor = ((self.resX * self.resY) / (1600 * 900)) ^ 0.5
  local scaleFactorX = self.resX / 1600
  local scaleFactorY = self.resY /  900
  HmGui.Image(self.logo) -- draw the LTR logo on top of the canvas
  HmGui.SetStretch(0.76 * scaleFactor / scaleFactorX, 0.243 * scaleFactor / scaleFactorY) -- scale logo (width, height)
  HmGui.SetAlign(0.5, 0.5) -- align logo
end

function LTheoryRedux:showMainMenu ()
  -- Add Main Menu dialog
  local scalefactor = (self.resX / 22) / 72
  local scalefactorMenuX = 352.8 / self.resX
  local scalefactorMenuY = 549   / self.resY

  HmGui.BeginGroupStack()
    HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 72 * scalefactor), 'LIMIT THEORY', 0.2, 0.2, 0.2, 1.0)
    HmGui.SetAlign(0.031, 0.042)
    HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 72 * scalefactor), 'LIMIT THEORY', 0.9, 0.9, 0.9, 1.0)
    HmGui.SetAlign(0.03, 0.04)
    HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 58 * scalefactor), 'REDUX', 0.2, 0.2, 0.2, 1.0)
    HmGui.SetAlign(0.181, 0.132)
    HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 58 * scalefactor), 'REDUX', 0.9, 0.9, 0.9, 1.0)
    HmGui.SetAlign(0.18, 0.13)

    HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 12 * scalefactor), Config.gameVersion, 0.2, 0.2, 0.2, 1.0)
    HmGui.SetAlign(0.012, 0.971)
    HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 12 * scalefactor), Config.gameVersion, 0.9, 0.9, 0.9, 1.0)
    HmGui.SetAlign(0.011, 0.970)

    HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 12 * scalefactor), 'Resolution = '..self.resX..' x '..self.resY, 0.2, 0.2, 0.2, 1.0)
    HmGui.SetAlign(0.221, 0.971)
    HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 12 * scalefactor), 'Resolution = '..self.resX..' x '..self.resY, 0.9, 0.9, 0.9, 1.0)
    HmGui.SetAlign(0.220, 0.970)

    self:showMainMenuInner()

    HmGui.SetStretch(0.18, 0.5)
    HmGui.SetAlign(0.0065, 0.8)
  HmGui.EndGroup()
end

function LTheoryRedux:showMainMenuInner ()
  -- Add Main Menu items
  local scalefactor = (self.resX / 24) / 72

  HmGui.BeginGroupY()
    HmGui.PushTextColor(0.9, 0.9, 0.9, 1.0)
    HmGui.PushFont(Cache.Font('RajdhaniSemiBold', 36 * scalefactor))
    if HmGui.Button("NEW GAME") then
      LTheoryRedux:showSeedDialog()
    end
    if HmGui.Button("LOAD GAME") then
      LTheoryRedux:showSeedDialog()
    end
    if HmGui.Button("SETTINGS") then
    end
    if HmGui.Button("CREDITS") then
    end
    if HmGui.Button("BACKGROUND") then
      bBackgroundMode = true
    end
    if HmGui.Button("EXIT GAME") then
      LTheoryRedux:exitGame()
    end
    HmGui.PopStyle(2)
  HmGui.EndGroup()
end

function LTheoryRedux:showFlightDialog ()
  -- Add Flight Mode dialog menu
  HmGui.BeginWindow("Flight Mode")
    HmGui.TextEx(Cache.Font('Iceland', 36), 'Flight Mode Controls', 0.3, 0.4, 0.5, 1.0)
    HmGui.SetAlign(0.5, 0.5)
    HmGui.SetSpacing(16)
    self:showFlightDialogInner()
  HmGui.EndWindow()
  HmGui.SetAlign(0.5, 0.5)
end

function LTheoryRedux:showFlightDialogInner ()
  -- Add Flight Mode dialog menu items
  HmGui.BeginGroupY()
    HmGui.PushTextColor(1.0, 1.0, 1.0, 1.0)
    HmGui.PushFont(Cache.Font('Exo2Bold', 26))
    if Config.game.currentShip ~= nil and not Config.game.currentShip:isDestroyed() then
      if HmGui.Button("Return to Game") then
        LTheoryRedux:freezeTurrets()
        Config.game.flightModeButInactive = false
        Config.game.gamePaused = false
      end
    end
    if Config.game.currentShip ~= nil and not Config.game.currentShip:isDestroyed() then
      HmGui.SetSpacing(8)
      if HmGui.Button("Save Game") then
        -- TODO: Save game state here
        LTheoryRedux:freezeTurrets()
        Config.game.flightModeButInactive = false
        Config.game.gamePaused = false
      end
    end
    HmGui.SetSpacing(8)
    if HmGui.Button("Load Game") then
      -- TODO: Show Load Game menu once that's been implemented
      -- NOTE: For now, just pop up a Seed Menu dialog for creating a new star system
      LTheoryRedux:showSeedDialog()
      Config.game.flightModeButInactive = false
    end
    HmGui.SetSpacing(8)
    if HmGui.Button("Game Settings") then
      -- TODO: Show Game Settings menu once that's been implemented
      LTheoryRedux:freezeTurrets()
      Config.game.flightModeButInactive = false
      Config.game.gamePaused = false
    end
    HmGui.SetSpacing(8)
    if HmGui.Button("Exit to Main Menu") then
      Config.game.flightModeButInactive = true
      Config.setGameMode(1) -- switch to Startup Mode
      LTheoryRedux:seedStarsystem(1) -- use random seed for new background star system and display it in Main Menu mode
      Config.game.gamePaused = false
    end
    HmGui.SetSpacing(8)
    if HmGui.Button("Exit Game") then
      LTheoryRedux:exitGame()
    end
    HmGui.PopStyle(2)
  HmGui.EndGroup()
end

function LTheoryRedux:showSeedDialog ()
  -- Add new star system seed selection dialog menu
  bSeedDialogDisplayed = true
  HmGui.BeginWindow(guiElements.name)
    HmGui.TextEx(Cache.Font('Iceland', 42), 'Choose Seed', 0.3, 0.4, 0.5, 1.0)
    HmGui.SetAlign(0.5, 0.5)
    HmGui.SetSpacing(16)
    self:showSeedDialogInner()
  HmGui.EndWindow()
  HmGui.SetAlign(0.5, 0.5)
end

function LTheoryRedux:showSeedDialogInner ()
  -- Add new star system seed selection dialog menu items
  HmGui.BeginGroupY()
    HmGui.PushTextColor(1.0, 1.0, 1.0, 1.0)
    HmGui.PushFont(Cache.Font('Exo2', 26))

    -- Loop through saved seeds (hardcoded for now) and display as checkboxes
    for i = 1, #guiElements[1]["elems"] do
      -- Create the new checkbox and save a reference to its current state (T/F)
      guiElements[1]["elems"][i][3] = HmGui.Checkbox(tostring(guiElements[1]["elems"][i][2]), guiElements[1]["elems"][i][3])
      if guiElements[1]["elems"][i][3] then
        -- Checkbox was selected
        -- Reset all other checkboxes (so that these checkboxes will work like radio buttons, where only one can be active)
        for j = 1, #guiElements[1]["elems"] do
          if j ~= i then
            guiElements[1]["elems"][j][3] = false
          end
        end
        -- Save the star system seed associated with it
        newSeed = guiElements[1]["elems"][i][2]
      end

      HmGui.SetSpacing(8)
    end

    HmGui.SetSpacing(16)

    HmGui.BeginGroupX()
      HmGui.PushTextColor(1.0, 1.0, 1.0, 1.0)
      HmGui.PushFont(Cache.Font('Exo2Bold', 28))
      if HmGui.Button("Cancel") then
        bSeedDialogDisplayed = false
        bNewSSystem = false
        menuMode = Config.getGameMode()
        LTheoryRedux:freezeTurrets()
        Config.game.gamePaused = false
      end
      HmGui.SetSpacing(16)
      if HmGui.Button("Random Seed") then
        newSeed = rng:get64() -- get a random seed value
        bSeedDialogDisplayed = false
        for i = 1, #guiElements[1]["elems"] do -- reset all seed selection checkboxes
          guiElements[1]["elems"][i][3] = false
        end
        bNewSSystem = true
        Config.setGameMode(2) -- switch to Flight Mode
        menuMode = 2
        Config.game.flightModeButInactive = false
        Config.game.gamePaused = false
      end
      HmGui.SetSpacing(16)
      if HmGui.Button("Use Seed") then
        bSeedDialogDisplayed = false
        for i = 1, #guiElements[1]["elems"] do -- reset all seed selection checkboxes
          guiElements[1]["elems"][i][3] = false
        end
        bNewSSystem = true
        Config.setGameMode(2) -- switch to Flight Mode
        menuMode = 2
        Config.game.flightModeButInactive = false
        Config.game.gamePaused = false
      end
      HmGui.PopStyle(2)
    HmGui.EndGroup()
    HmGui.SetAlign(0.5, 0.5)
    HmGui.PopStyle(2)
  HmGui.EndGroup()
end

function LTheoryRedux:exitGame ()
  -- Shut down game and exit
  Sound.SetVolume(newSound, 0.0)

  LTheoryRedux:quit()
end

--** SUPPORT FUNCTIONS **--
function LTheoryRedux:freezeTurrets ()
  -- When taking down a dialog, Turret:updateTurret sees the button click input and thinks it means "Fire"
  -- So this routine adds a very brief cooldown to the player ship's turrets
  if Config.game.currentShip then
    for turret in Config.game.currentShip:iterSocketsByType(SocketType.Turret) do
      turret:addCooldown(2.0)
    end
  end
end

function LTheoryRedux:sleep (sec)
  os.execute(package.config:sub(1,1) == "/" and "sleep " .. sec or "timeout " .. sec )
end

return LTheoryRedux
