--** REQUIRES **--
local Player = require('GameObjects.Entities.Player')
local System = require('GameObjects.Entities.Test.System')
local DebugControl = require('Systems.Controls.Controls.DebugControl')
local Actions = requireAll('GameObjects.Actions')

local LTheoryRedux = require('States.Application')

--** LOCAL VARIABLES **--
local currentVersion = "v0.002"

local newSeed = 0ULL
local newShip = nil
local rng = RNG.FromTime()
local menuMode = 0 -- initially show game logo
local scalefactor = 0.0
local bNewUniverse = false
local bFlightModePaused = false
local bSeedDialogDisplayed = false
local bBackgroundMode = false

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
      { nil, 12118942710891801364ULL, false }
    }
  }
}

--** MAIN CODE **--
function LTheoryRedux:onInit ()
  self.logo   = Tex2D.Load('./res/images/LTR_logo1d.png') -- load the LTR logo
  self.mmback = Tex2D.Load('./res/images/LTR-MM-background.png') -- load the Main Menu background

  DebugControl.ltheory = self

  self.player = Entities.Player()
  self:generate()

  -- Audio initialization moved here from GameView.lua
  Audio.Init()
  Audio.Set3DSettings(0.0, 10, 2);
end

function LTheoryRedux:onInput ()
  self.canvas:input()
end

function LTheoryRedux:onDraw ()
  self.canvas:draw(self.resX, self.resY)
  HmGui.Draw() -- draw controls
end

function LTheoryRedux:onUpdate (dt)
  self.player:getRoot():update(dt)
  self.canvas:update(dt)

  -- Add basic Game Control menu
  if Input.GetPressed(Button.Keyboard.Escape) then
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

  if bNewUniverse then
    bNewUniverse = false
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

  LTheoryRedux:seedUniverse(0) -- use random seed for new background star system, and stay in "display game logo" startup mode
end

function LTheoryRedux:seedUniverse (changeMenuMode)
  self.seed = rng:get64()

  LTheoryRedux:createStarSystem()

  menuMode = changeMenuMode
end

function LTheoryRedux:createStarSystem ()
  if self.system then self.system:delete() end
printf("Spawning new star system using seed = %s", self.seed)
  self.system = System(self.seed)

  do
    if Config.getGameMode() == 1 then
      newShip = self.system:spawnBackground() -- spawn an invisible ship

      for i = 1, 1 do
        self.system:spawnStation()
      end

      for i = 1, 1 do
        self.system:spawnAsteroidField(500, 10)
      end

      for i = 1, 1 do
        self.system:spawnPlanet()
      end

      LTheoryRedux:insertShip(newShip)
    else
      newShip = self.system:spawnShip()

      -- World generation temporarily copied from background generation
      -- until actual world generation is written
      for i = 1, 1 do
        self.system:spawnStation()
      end

      for i = 1, 1 do
        self.system:spawnAsteroidField(500, 10)
      end

      for i = 1, 1 do
        self.system:spawnPlanet()
      end

      LTheoryRedux:insertShip(newShip)

      -- player escorts
      local ships = {}
      for i = 1, 100 do
        local escort = self.system:spawnShip()
        local offset = rng:getSphere():scale(100)
        escort:setPos(newShip:getPos() + offset)
        escort:setOwner(self.player)
        escort:pushAction(Actions.Escort(newShip, offset))
        insert(ships, escort)
      end

      for i = 1, #ships do
        local j = rng:getInt(1, #ships)
        if i ~= j then
          -- ships[i]:pushAction(Actions.Attack(ships[j]))
        end
      end

      for i = 1, 0 do
        self.system:spawnAI(100)
      end
    end
  end

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
    HmGui.Image(self.mmback) -- draw the Main Menu image background on top of the canvas
    HmGui.SetStretch(0.3, 1.0) -- scale menu image background (width, height)
    HmGui.SetAlign(0.0, 0.0)
  HmGui.EndGroup()

  HmGui.BeginGroupStack()
    HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 72 * scalefactor), 'LIMIT THEORY', 0.77, 0.77, 0.77, 1.0)
    HmGui.SetAlign(0.03, 0.04)
    HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 58 * scalefactor), 'REDUX', 0.77, 0.77, 0.77, 1.0)
    HmGui.SetAlign(0.18, 0.13)

    HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 18 * scalefactor), currentVersion, 0.77, 0.77, 0.77, 1.0)
    HmGui.SetAlign(0.01, 0.97)

    HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 18 * scalefactor), 'Resolution = '..self.resX..' x '..self.resY, 0.77, 0.77, 0.77, 1.0)
    HmGui.SetAlign(0.21, 0.97)

    self:showMainMenuInner()

    HmGui.SetStretch(0.194, 0.6) -- 0.245, 0.6
    HmGui.SetAlign(0.0065, 0.72) -- 0.0065, 0.74
  HmGui.EndGroup()
end

function LTheoryRedux:showMainMenuInner ()
  -- Add Main Menu items
  HmGui.BeginGroupY()
    HmGui.PushTextColor(1.0, 1.0, 1.0, 1.0)
    HmGui.PushFont(Cache.Font('RajdhaniSemiBold', 32))
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
      LTheoryRedux:quit()
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
      LTheoryRedux:seedUniverse(1) -- use random seed for new background star system and display it in Main Menu mode
    end
    HmGui.SetSpacing(8)
    if HmGui.Button("Exit Game") then
      LTheoryRedux:quit()
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
      -- Create the new checkbox
      guiElements[1]["elems"][i][3] = HmGui.Checkbox(tostring(guiElements[1]["elems"][i][2]), guiElements[1]["elems"][i][3])
      if guiElements[1]["elems"][i][3] then
        -- Checkbox was selected
        -- Reset all other checkboxes (so these work like radio buttons, where only one can be active)
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
        bNewUniverse = false
        menuMode = Config.getGameMode()
      end
      HmGui.SetSpacing(16)
      if HmGui.Button("Random Seed") then
        newSeed = rng:get64() -- get a random seed value
        bSeedDialogDisplayed = false
        for i = 1, #guiElements[1]["elems"] do -- reset all seed selection checkboxes
          guiElements[1]["elems"][i][3] = false
        end
        bNewUniverse = true
        Config.setGameMode(2) -- switch to Flight Mode
        menuMode = 2
      end
      HmGui.SetSpacing(16)
      if HmGui.Button("Use Seed") then
        bSeedDialogDisplayed = false
        for i = 1, #guiElements[1]["elems"] do -- reset all seed selection checkboxes
          guiElements[1]["elems"][i][3] = false
        end
        bNewUniverse = true
        Config.setGameMode(2) -- switch to Flight Mode
        menuMode = 2
      end
      HmGui.PopStyle(2)
    HmGui.EndGroup()
    HmGui.SetAlign(0.5, 0.5)
    HmGui.PopStyle(2)
  HmGui.EndGroup()
end

--** SUPPORT FUNCTIONS **--
function LTheoryRedux:sleep (sec)
  os.execute(package.config:sub(1,1) == "/" and "sleep " .. sec or "timeout " .. sec )
end

return LTheoryRedux
