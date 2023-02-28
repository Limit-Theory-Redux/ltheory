--** REQUIRES **--
local Player = require('GameObjects.Entities.Player')
local System = require('GameObjects.Entities.Test.System')
local DebugControl = require('Systems.Controls.Controls.DebugControl')
local Bindings = require('States.ApplicationBindings')
local Actions = requireAll('GameObjects.Actions')
local Item = require('Systems.Economy.Item')
local SocketType = require('GameObjects.Entities.Ship.SocketType')

local LTheoryRedux = require('States.Application')

--** LOCAL VARIABLES **--
local newSound = nil
local newSeed = 0ULL
local newShip = nil
local menuMode = 0 -- initially show game logo
local scalefactor = 0.0
local bNewSSystem = false
local bFlightModePaused = false
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
      { nil, 7035008865122330386ULL,  false },
      { nil, 15054808765102574876ULL, false },
      { nil, 1777258448479734603ULL,  false },
      { nil, 9770135211012317023ULL,  false },
      { nil, 13415752391947803947ULL, false },
      { nil, 18346913580697132292ULL, false },
      { nil, 8788869510796381519ULL,  false },
      { nil, 8668067427585514558ULL,  false },
      { nil, 3806448947569663889ULL,  false },
      { nil, 2509601882259751919ULL,  false },
      { nil, 12145308173506787001ULL, false },
      { nil, 7450823138892184048ULL,  false }
    }
  }
}


--** MAIN CODE **--
function LTheoryRedux:onInit ()
  self.logo = Tex2D.Load("./res/images/LTR_logo1d.png") -- load the LTR logo

  DebugControl.ltheory = self

  self.player = Entities.Player()
  self:generate()

  -- Audio initialization moved here from GameView.lua
  Audio.Init()
  Audio.Set3DSettings(0.0, 10, 2);

  -- Music courtesy of MesoTroniK
  newSound = Sound.Load("./res/sound/system/ambiance/LTR_Surpassing_The_Limit_Redux_Ambient_Long_Fade.ogg", true, false)
  Sound.SetVolume(newSound, 1) -- SetVolume range seems to go from 0 (min) to about 2 or 3 (max)
  Sound.Play(newSound)
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

  -- Add basic Game Control menu
  if Input.GetPressed(Bindings.Escape) then
    bBackgroundMode = false
    if Config.getGameMode() == 1 then
      menuMode = 1 -- show Main Menu
    else
      -- The first time we get here, menuMode should be 0 to show we're just starting the game,
      --   so don't pop up the Flight Mode dialog box
      -- After that, when we're in Flight Mode, do pop up the Flight Mode dialog box when the player presses ESC
      if menuMode == 0 then
        bFlightModePaused = false
      else
        bFlightModePaused = true
      end
      menuMode = 2 -- show Flight Mode dialog
    end
  end

  -- If player pressed the "System Map" key in Flight Mode, toggle the system map's visibility
  if Input.GetPressed(Bindings.SystemMap) and menuMode == 2 then
    bShowSystemMap = not bShowSystemMap
    if smap == nil then
      smap = Systems.CommandView.SystemMap(self.system)
    end
  end

  -- Disengage autopilot (require a 1-second delay, otherwise keypress turns autopilot on then off instantly)
  if Config.game.playerMoving then
    if Input.GetPressed(Bindings.MoveTo) and Config.getCurrentTimestamp() - Config.game.autonavTimestamp > 1 then
      Config.game.playerMoving = false
    end
  end

  -- Canvas overlays
  HmGui.Begin(self.resX, self.resY)
    if menuMode == 0 then
      LTheoryRedux:showGameLogo()
    elseif menuMode == 1 then
      if not bBackgroundMode then -- why can't I say "~bBackgroundMode"? Why isn't "~" a normal unary operator? &^%$ Lua designers!
        LTheoryRedux:showMainMenu()
      end
    elseif menuMode == 2 then
      if bFlightModePaused then
        LTheoryRedux:showFlightDialog()
      else
        if bSeedDialogDisplayed then
          LTheoryRedux:showSeedDialog()
        end
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
  self.system = System(self.seed)
printf("Spawning new star system '%s' using seed = %s", self.system:getName(), self.seed)

  do
    if Config.getGameMode() == 1 then
      -- Generate a new star system with nebulae/dust, a planet, an asteroid field,
      --   a space station, and an invisible rotating ship
      newShip = self.system:spawnBackground() -- spawn an invisible ship
      LTheoryRedux:insertShip(newShip)

      -- Add a planet
      for i = 1, 1 do
        self.system:spawnPlanet()
      end

      -- Add a space station
      for i = 1, 1 do
        self.system:spawnStation()
      end

      -- Add an asteroid field
      for i = 1, 1 do
        self.system:spawnAsteroidField(500)
      end
    else
      -- Generate a new star system with nebulae/dust, a planet, an asteroid field,
      --   a space station, a visible pilotable ship, and 100 "escort" ships
      local asteroidCount = 500
      local escortCount   = 100
      local aField = nil

      -- Add the player's ship
      newShip = self.system:spawnShip()
      newShip:setName("NSS Titonicus")
--      newShip:setHealth(1000, 1000, 50) -- make the player's ship extra-healthy for now
      newShip:setHealth(500, 500, 20) -- make the player's ship extra-healthy for now
      Config.game.currentShip = newShip
      LTheoryRedux:insertShip(newShip)
      printf("Added our ship, the '%s'", newShip:getName())

      -- Add a planet
      for i = 1, 1 do
        self.system:spawnPlanet()
      end

      -- Add a space station
      for i = 1, 1 do
        self.system:spawnStation()
      end

      -- Add an asteroid field
      for i = 1, 1 do
        aField = self.system:spawnAsteroidField(asteroidCount)
      end
      printf("Added %s asteroids to %s", asteroidCount, aField:getName())
      --printf("Object type is '%s'", Config.objectInfo[1]["elems"][aField:getType()][2])

      -- Add escort ships
      local ships = {}

      for i = 1, escortCount do
        local escort = self.system:spawnShip()
        local offset = rng:getSphere():scale(100)
        escort:setPos(newShip:getPos() + offset)
        escort:setOwner(self.player)
        escort:addItem(Item.Credit, Config.game.eStartCredits)
--        escort:pushAction(Actions.Think()) -- (currently generates an error)
--        escort:pushAction(Actions.Attack(newShip)) -- (currently doesn't break, but doesn't work)
        escort:pushAction(Actions.Escort(newShip, offset))
        insert(ships, escort)
      end

      -- Make ships chase each other!
--      for i = 1, #ships - 1 do
--        ships[i]:pushAction(Actions.Attack(ships[i+1]))
--      end

      printf("Added %s escort ships", escortCount)
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
  HmGui.SetStretch(0.74947917 * scaleFactor / scaleFactorX, 0.28 * scaleFactor / scaleFactorY) -- scale logo (width, height)
  HmGui.SetAlign(0.5, 0.5) -- align logo
end

function LTheoryRedux:showMainMenu ()
  -- Add Main Menu dialog
  local scalefactor = (self.resX / 25) / 72
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

    HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 18 * scalefactor), Config.version, 0.2, 0.2, 0.2, 1.0)
    HmGui.SetAlign(0.011, 0.971)

    HmGui.SetAlign(0.01, 0.97)

    HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 18 * scalefactor), 'Resolution = '..self.resX..' x '..self.resY, 0.2, 0.2, 0.2, 1.0)
    HmGui.SetAlign(0.161, 0.971)
    HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 18 * scalefactor), 'Resolution = '..self.resX..' x '..self.resY, 0.9, 0.9, 0.9, 1.0)
    HmGui.SetAlign(0.16, 0.97)

    self:showMainMenuInner()

    HmGui.SetStretch(0.194, 0.6)
    HmGui.SetAlign(0.0065, 0.72)
  HmGui.EndGroup()
end

function LTheoryRedux:showMainMenuInner ()
  -- Add Main Menu items
  local scalefactor = (self.resX / 25) / 72

  HmGui.BeginGroupY()
    HmGui.PushTextColor(0.9, 0.9, 0.9, 1.0)
    HmGui.PushFont(Cache.Font('RajdhaniSemiBold', 36 * scalefactor))
    if HmGui.Button("NEW GAME") then
      LTheoryRedux:showSeedDialog()
      menuMode = 2
    end
    if HmGui.Button("LOAD GAME") then
      LTheoryRedux:showSeedDialog()
      menuMode = 2
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
    HmGui.TextEx(Cache.Font('Iceland', 20), 'Flight Mode Controls', 0.3, 0.4, 0.5, 1.0)
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
    HmGui.PushFont(Cache.Font('Exo2Bold', 18))
    if HmGui.Button("Return to Game") then
      bFlightModePaused = false
    end
    HmGui.SetSpacing(8)
    if HmGui.Button("Save Game") then
      bFlightModePaused = false
    end
    HmGui.SetSpacing(8)
    if HmGui.Button("Load Game") then
      bFlightModePaused = false
      LTheoryRedux:showSeedDialog()
    end
    HmGui.SetSpacing(8)
    if HmGui.Button("Game Settings") then
      bFlightModePaused = true
    end
    HmGui.SetSpacing(8)
    if HmGui.Button("Exit to Main Menu") then
      bFlightModePaused = false
      Config.setGameMode(1) -- switch to Startup Mode
      LTheoryRedux:seedStarsystem(1) -- use random seed for new background star system and display it in Main Menu mode
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
    HmGui.TextEx(Cache.Font('Iceland', 24), 'Choose Seed', 0.3, 0.4, 0.5, 1.0)
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
    HmGui.PushFont(Cache.Font('Exo2', 16))

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
      HmGui.PushFont(Cache.Font('Exo2Bold', 18))
      if HmGui.Button("Cancel") then
        bSeedDialogDisplayed = false
        bNewSSystem = false
        menuMode = Config.getGameMode()
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
function LTheoryRedux:sleep (sec)
  os.execute(package.config:sub(1,1) == "/" and "sleep " .. sec or "timeout " .. sec )
end

return LTheoryRedux
