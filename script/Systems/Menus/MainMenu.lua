local MainMenu = class(function (self) end)

local MusicPlayer = require('Systems.SFX.MusicPlayer')
local Bindings = require('States.ApplicationBindings')

local mainMenuMusic = nil

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

function MainMenu:OnInit()
  self.enabled = true
  self.inBackgroundMode = false
  self.seedDialogDisplayed = false
  self.dt = 0
  self.lastActionDelta = 0
  self.returnToSplashDelta = 0

  if not self.keepState then
    self.currentMode = Enums.MenuMode.Splashscreen
    self.keepState = false
  end
  printf("Initialize MainMenu")
end

function MainMenu:ActionRegistered()
  self.lastActionDelta = self.dt
end

function MainMenu:OnUpdate(dt)
  if not self.dt or not dt then return end

  self.dt = self.dt + dt

  if self.enabled and self.currentMode == Enums.MenuMode.MainMenu and not MainMenu.inBackgroundMode then
    if self.lastActionDelta then
      self.returnToSplashDelta = self.lastActionDelta + Config.timeToResetToSplashscreen
    end

    if self.returnToSplashDelta ~= 0 and self.dt >= self.returnToSplashDelta then
      self:SetMenuMode(Enums.MenuMode.Splashscreen)
      self.lastActionDelta = 0
      self.returnToSplashDelta = 0
    end

    --printf("dt:".. self.dt)
    --printf("lastAction: " .. self.lastActionDelta)
    --printf("returnToSplashDelta: " .. self.returnToSplashDelta)
    --printf(Config.timeToResetToSplashscreen)
  else
    self.lastActionDelta = 0
    self.returnToSplashDelta = 0
  end
end

function MainMenu:Open()
  if not self.enabled then
    self:OnInit()
  end

  mainMenuMusic = MusicPlayer:QueueTrack(Config.audio.mainMenu, true)

  printf("Opening Main Menu.")
end

function MainMenu:Close(keepState)
  self.enabled = false
  self.keepState = keepState

  MusicPlayer:StopTrack(mainMenuMusic)

  printf("Closing Main Menu.")
end

function MainMenu:SetBackgroundMode(enabled)
  printf("Set Background Mode to: " .. tostring(enabled))
  self.inBackgroundMode = enabled
end

function MainMenu:SetMenuMode(mode)
  printf("Set Menu Mode to: " .. mode)
  self.currentMode = mode
end

function MainMenu:ShowGui()
  -- Add Main Menu dialog
  local scalefactor = (LTheoryRedux.resX / 22) / 72
  local scalefactorMenuX = 352.8 / LTheoryRedux.resX
  local scalefactorMenuY = 549   / LTheoryRedux.resY

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

  HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 12 * scalefactor), 'Resolution = '..LTheoryRedux.resX..' x '..LTheoryRedux.resY, 0.2, 0.2, 0.2, 1.0)
  HmGui.SetAlign(0.221, 0.971)
  HmGui.TextEx(Cache.Font('RajdhaniSemiBold', 12 * scalefactor), 'Resolution = '..LTheoryRedux.resX..' x '..LTheoryRedux.resY, 0.9, 0.9, 0.9, 1.0)
  HmGui.SetAlign(0.220, 0.970)

  self:ShowMainMenuInner()

  HmGui.SetStretch(0.18, 0.5)
  HmGui.SetAlign(0.0065, 0.8)
  HmGui.EndGroup()
end

function MainMenu:ShowMainMenuInner()
  -- Add Main Menu items
  local scalefactor = (LTheoryRedux.resX / 24) / 72

  HmGui.BeginGroupY()
  HmGui.PushTextColor(0.9, 0.9, 0.9, 1.0)
  HmGui.PushFont(Cache.Font('RajdhaniSemiBold', 36 * scalefactor))

  if HmGui.Button("NEW GAME") then
    self:ShowSeedDialog()
  end

  if HmGui.Button("LOAD GAME") then
    self:ShowSeedDialog()
  end

  if HmGui.Button("SETTINGS") then
  end

  if HmGui.Button("CREDITS") then
  end

  if HmGui.Button("BACKGROUND") then
    self:SetBackgroundMode(true)
  end

  if HmGui.Button("EXIT GAME") then
    LTheoryRedux:exitGame()
  end
  HmGui.PopStyle(2)
  HmGui.EndGroup()
end


function MainMenu:ShowSeedDialog()
  -- Add new star system seed selection dialog menu
  self.seedDialogDisplayed = true

  HmGui.BeginWindow(guiElements.name)
  HmGui.TextEx(Cache.Font('Iceland', 42), 'Choose Seed', 0.3, 0.4, 0.5, 1.0)
  HmGui.SetAlign(0.5, 0.5)
  HmGui.SetSpacing(16)
  self:ShowSeedDialogInner()
  HmGui.EndWindow()
  HmGui.SetAlign(0.5, 0.5)
end

function MainMenu:ShowSeedDialogInner()
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
      LTheoryRedux.seed = guiElements[1]["elems"][i][2]
    end
    HmGui.SetSpacing(8)
  end
  HmGui.SetSpacing(16)

  HmGui.BeginGroupX()
  HmGui.PushTextColor(1.0, 1.0, 1.0, 1.0)
  HmGui.PushFont(Cache.Font('Exo2Bold', 28))

  if HmGui.Button("Cancel") then
    self.seedDialogDisplayed = false
    self:SetMenuMode(Config.getGameMode())
    LTheoryRedux:freezeTurrets()
    Config.game.gamePaused = false

    if MainMenu.currentMode == Enums.MenuMode.Dialog then
      Input.SetMouseVisible(false)
    end
  end
  HmGui.SetSpacing(16)

  if HmGui.Button("Random Seed") then
    LTheoryRedux:generateNewSeed()
    self.seedDialogDisplayed = false

    for i = 1, #guiElements[1]["elems"] do -- reset all seed selection checkboxes
      guiElements[1]["elems"][i][3] = false
    end
    Config.setGameMode(2) -- switch to Flight Mode
    self:SetMenuMode(Enums.MenuMode.Dialog)
    Config.game.flightModeButInactive = false
    Config.game.gamePaused = false
    Input.SetMouseVisible(false)
    LTheoryRedux:createStarSystem()
  end
  HmGui.SetSpacing(16)

  if HmGui.Button("Use Seed") then
    self.seedDialogDisplayed = false

    for i = 1, #guiElements[1]["elems"] do -- reset all seed selection checkboxes
      guiElements[1]["elems"][i][3] = false
    end
    Config.setGameMode(2) -- switch to Flight Mode
    self:SetMenuMode(Enums.MenuMode.Dialog)
    Config.game.flightModeButInactive = false
    Config.game.gamePaused = false
    Input.SetMouseVisible(false)
    LTheoryRedux:createStarSystem()
  end
  HmGui.PopStyle(2)
  HmGui.EndGroup()
  HmGui.SetAlign(0.5, 0.5)
  HmGui.PopStyle(2)
  HmGui.EndGroup()
end

function MainMenu:ShowFlightDialog()
  -- Add Flight Mode dialog menu
  HmGui.BeginWindow("Flight Mode")
  HmGui.TextEx(Cache.Font('Iceland', 36), 'Flight Mode Controls', 0.3, 0.4, 0.5, 1.0)
  HmGui.SetAlign(0.5, 0.5)
  HmGui.SetSpacing(16)
  self:ShowFlightDialogInner()
  HmGui.EndWindow()
  HmGui.SetAlign(0.5, 0.5)
end

function MainMenu:ShowFlightDialogInner()
  -- Add Flight Mode dialog menu items
  HmGui.BeginGroupY()
  HmGui.PushTextColor(1.0, 1.0, 1.0, 1.0)
  HmGui.PushFont(Cache.Font('Exo2Bold', 26))

  if Config.game.currentShip ~= nil and not Config.game.currentShip:isDestroyed() then
    if HmGui.Button("Return to Game") then
      --printf("panelActive = %s, defaultControl = %s", Config.game.panelActive, Config.ui.defaultControl)
      LTheoryRedux:freezeTurrets()
      Config.game.flightModeButInactive = false
      Config.game.gamePaused = false
      Config.game.panelActive = false

      if Config.ui.defaultControl == "Ship" then
        Input.SetMouseVisible(false)
      end
    end
  end

  if Config.game.currentShip ~= nil and not Config.game.currentShip:isDestroyed() then
    HmGui.SetSpacing(8)

    if HmGui.Button("Save Game") then
      -- TODO: Save game state here
      LTheoryRedux:freezeTurrets()
      Config.game.flightModeButInactive = false
      Config.game.gamePaused = false
      Input.SetMouseVisible(false)
    end
  end
  HmGui.SetSpacing(8)

  if HmGui.Button("Load Game") then
    -- TODO: Show Load Game menu once that's been implemented
    -- NOTE: For now, just pop up a Seed Menu dialog for creating a new star system
    self:ShowSeedDialog()
    Config.game.flightModeButInactive = false
  end
  HmGui.SetSpacing(8)

  if HmGui.Button("Game Settings") then
    -- TODO: Show Game Settings menu once that's been implemented
    LTheoryRedux:freezeTurrets()
    Config.game.flightModeButInactive = false
    Config.game.gamePaused = false
    Input.SetMouseVisible(false)
  end
  HmGui.SetSpacing(8)

  if HmGui.Button("Exit to Main Menu") then
    Config.game.flightModeButInactive = true
    Config.setGameMode(1) -- switch to Startup Mode
    LTheoryRedux:seedStarsystem(Enums.MenuMode.MainMenu) -- use random seed for new background star system and display it in Main Menu mode
    Config.game.gamePaused = false
  end
  HmGui.SetSpacing(8)

  if HmGui.Button("Exit Game") then
    LTheoryRedux:exitGame()
  end
  HmGui.PopStyle(2)
  HmGui.EndGroup()
end

return MainMenu