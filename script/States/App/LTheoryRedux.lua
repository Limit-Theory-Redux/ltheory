--** REQUIRES **--
local Player = require('GameObjects.Entities.Player')
local System = require('GameObjects.Entities.Test.System')
local DebugControl = require('Systems.Controls.Controls.DebugControl')
local Bindings = require('States.ApplicationBindings')
local ShipBindings = require('Systems.Controls.Bindings.ShipBindings')
local Actions = requireAll('GameObjects.Actions')
local SocketType = require('GameObjects.Entities.Ship.SocketType')
local InitFiles = require('Systems.Files.InitFiles')
local MainMenu = require('Systems.Menus.MainMenu')
local MusicPlayer = require('Systems.SFX.MusicPlayer')
local Universe = require('Systems.Universe.Universe')

LTheoryRedux = require('States.Application')

--** LOCAL VARIABLES **--
local newShip = nil
local bShowSystemMap = false
local bSMapAdded = false
local smap = nil

local rng = RNG.FromTime()

--** MAIN CODE **--
function LTheoryRedux:onInit()
    --* Value initializations *--
    self.logo = Tex2D.Load("./res/images/LTR_logo2.png") -- load the LTR logo

    DebugControl.ltheory = self

    -- Read user-defined values and update game variables
    InitFiles:readUserInits()

    -- Initialize Universe
    Universe:Init()

    -- Open Main Menu
    MusicPlayer:Init()
    MainMenu:Open()

    --* Game initializations *--
    self.window:setSize(GameState.render.resX, GameState.render.resY)
    -- self.window:setPosition(WindowPos.Centered, WindowPos.Centered)
    LTheoryRedux:SetFullscreen(GameState.render.fullscreen)

    -- Set the default game control cursor
    -- TODO: self.window:cursor().setIcon(Enums.CursorFilenames[GameState.ui.cursorStyle])
    self.window:setCursorPosition(Vec2f(GameState.ui.cursorX, GameState.ui.cursorY))

    self.player = Entities.Player(GameState.player.humanPlayerName)
    GameState.player.humanPlayer = self.player

    self:generate()
end

function LTheoryRedux:setCursor(cursorStyle, cursorX, cursorY)
    -- Set the game control cursor
    -- TODO: self.window:cursor().setIcon(cursorStyle)
    self.window:setCursorPosition(Vec2f(cursorX, cursorY))
end

function LTheoryRedux:toggleSound()
    GameState.audio.soundEnabled = not GameState.audio.soundEnabled

    if GameState.audio.soundEnabled then
        MusicPlayer:SetVolume(GameState.audio.musicVolume)
    else
        --printf("LTheoryRedux:toggleSound: volume set to 0")
        MusicPlayer:SetVolume(0)
    end
end

function LTheoryRedux:SoundOn()
    GameState.audio.soundEnabled = true
    --printf("LTheoryRedux:SoundOn: volume set to %s", GameState.audio.musicVolume)
    MusicPlayer:SetVolume(GameState.audio.musicVolume)
end

function LTheoryRedux:SoundOff()
    GameState.audio.soundEnabled = false
    --printf("LTheoryRedux:SoundOff: volume set to 0")
    MusicPlayer:SetVolume(0)
end

function LTheoryRedux:ToggleFullscreen()
    GameState.render.fullscreen = not GameState.render.fullscreen
    self.window:setFullscreen(GameState.render.fullscreen)
end

function LTheoryRedux:SetFullscreen(fullscreen)
    GameState.render.fullscreen = fullscreen
    self.window:setFullscreen(fullscreen)
end

function LTheoryRedux:onInput()
    self.canvas:input()

    if GameState:GetCurrentState() == Enums.GameStates.InGame and GameState.player.currentControl == Enums.ControlModes.Ship then
        if Input.GetPressed(Bindings.CameraFirstPerson) then
            if GameState.player.currentCamera ~= Enums.CameraMode.FirstPerson then
                self.gameView:setCameraMode(Enums.CameraMode.FirstPerson)
            end
        elseif Input.GetPressed(Bindings.CameraChase) then
            if GameState.player.currentCamera ~= Enums.CameraMode.Chase then
                self.gameView:setCameraMode(Enums.CameraMode.Chase)
            end
        elseif Input.GetPressed(Bindings.CameraOrbit) then
            --if GameState.player.currentCamera ~= Enums.CameraMode.Orbit then
            --  self.gameView:setCameraMode(Enums.CameraMode.Orbit)
            --end
        end
    end
end

function LTheoryRedux:onDraw()
    -- Check to see whether to draw the System Map or the game world onto the canvas
    if bShowSystemMap then
        if not bSMapAdded then
            self.canvas:remove(self.gameView)
            self.canvas:add(smap)
            bSMapAdded = true
            Input.SetMouseVisible(true)
            print("Draw System View")
        end
    else
        if smap ~= nil then
            self.canvas:remove(smap)
            self.canvas:add(self.gameView)
            bSMapAdded = false
            smap = nil
            Input.SetMouseVisible(false)
            print("Draw Game View")
        end
    end

    self.canvas:draw(self.resX, self.resY)

    HmGui.Draw() -- draw controls
end

function LTheoryRedux:onUpdate(dt)
    -- Routes
    GameState.player.humanPlayer:getRoot():update(dt)
    self.canvas:update(dt)
    MainMenu:OnUpdate(dt)
    MusicPlayer:OnUpdate(dt)
    Universe:OnUpdate(dt)

    -- TODO: Confirm whether this is still needed
    local playerShip = GameState.player.humanPlayer
    if playerShip ~= nil then
        playerShip = GameState.player.currentShip
    end

    if Bindings.All:get() == 1 then
        -- Take down splash text if pretty much any key is pressed
        if MainMenu.currentMode == Enums.MenuMode.Splashscreen then
            MainMenu:SetBackgroundMode(false)
            MainMenu:SetMenuMode(Enums.MenuMode.MainMenu) -- show Main Menu
        end
        MainMenu:ActionRegistered()
    end

    if not MainMenu.enabled and MainMenu.currentMode == Enums.MenuMode.MainMenu then
        MainMenu:Open()
    elseif MainMenu.enabled and MainMenu.currentMode == Enums.MenuMode.Dialog then
        MainMenu:Close(true)
    end

    -- Manage game control screens
    if MainMenu.currentMode ~= Enums.MenuMode.Splashscreen and Input.GetPressed(Bindings.Escape) then
        MainMenu:SetBackgroundMode(false)
        Input.SetMouseVisible(true)
        if GameState:GetCurrentState() == Enums.GameStates.MainMenu then
            MainMenu:SetMenuMode(Enums.MenuMode.MainMenu) -- show Main Menu
        else
            -- First time here, menuMode should be 0 (just starting game), so don't pop up the Flight Mode dialog box
            -- After that, in active Flight Mode, do pop up the Flight Mode dialog box when the player presses ESC
            if MainMenu.currentMode == Enums.MenuMode.Splashscreen then
                GameState:Unpause()
                MainMenu:SetMenuMode(Enums.MenuMode.Dialog) -- show Flight Mode dialog
            elseif MainMenu.currentMode == Enums.MenuMode.Dialog and not MainMenu.seedDialogDisplayed then
                MainMenu.dialogDisplayed = not MainMenu.dialogDisplayed
                Input.SetMouseVisible(MainMenu.dialogDisplayed)

                if MainMenu.dialogDisplayed then
                    GameState:Pause()
                else
                    GameState.panelActive = false
                    GameState:Unpause()
                end
            end
        end
    end

    -- If player pressed the "System Map" key in Flight Mode, toggle the system map's visibility
    if Input.GetPressed(Bindings.SystemMap) and MainMenu.currentMode == Enums.MenuMode.Dialog then
        bShowSystemMap = not bShowSystemMap
        if smap == nil then
            smap = Systems.CommandView.SystemMap(GameState.world.currentSystem)
        end
    end

    -- If in flight mode, engage autopilot
    if Input.GetPressed(Bindings.AutoNav) and MainMenu.currentMode == Enums.MenuMode.Dialog then
        if playerShip ~= nil then
            local target = playerShip:getTarget()
            if target == nil then target = self.focus end
            if not playerShip:isDestroyed() and playerShip:isShipDocked() == nil and target ~= nil and target ~= playerShip then
                if playerShip:getCurrentAction() == nil or not string.find(playerShip:getCurrentAction():getName(), "MoveTo") then
                    -- Move undestroyed, undocked player ship to area of selected target
                    local autodistance = Config.game.autonavRanges[target:getType()]
                    GameState.player.autonavTimestamp = Config.getCurrentTimestamp()
                    GameState.player.playerMoving = true -- must be set to true before pushing the MoveTo action
                    playerShip:pushAction(Actions.MoveTo(target, autodistance))
                end
            end
        end
    end

    -- Disengage autopilot (require a 1-second delay, otherwise keypress turns autopilot on then off instantly)
    if GameState.player.playerMoving then
        if Input.GetPressed(Bindings.AutoNav) and Config.getCurrentTimestamp() - GameState.player.autonavTimestamp > 1 then
            GameState.player.playerMoving = false
        end
    end

    -- If player pressed the "ToggleLights" key in Flight Mode, toggle dynamic lighting on/off
    -- NOTE: Performance is OK for just the player's ship, but adding many lit ships & pulses tanks performance
    if Input.GetPressed(Bindings.ToggleLights) and MainMenu.currentMode == Enums.MenuMode.Dialog then
        GameState.render.thrusterLights = not GameState.render.thrusterLights
        GameState.render.pulseLights    = not GameState.render.pulseLights
    end

    -- Decide which game controls screens (if any) to display on top of the canvas
    HmGui.Begin(self.resX, self.resY)

    if MainMenu.currentMode == Enums.MenuMode.Splashscreen then
        LTheoryRedux:showGameLogo()
    elseif MainMenu.currentMode == Enums.MenuMode.MainMenu then
        if not MainMenu.inBackgroundMode then
            if MainMenu.seedDialogDisplayed then
                MainMenu:ShowSeedDialog()
            elseif MainMenu.settingsScreenDisplayed then
                MainMenu:ShowSettingsScreen()
            else
                MainMenu:ShowGui()
            end
        end
    elseif MainMenu.currentMode == Enums.MenuMode.Dialog then
        if MainMenu.dialogDisplayed then
            MainMenu:ShowFlightDialog()
        elseif MainMenu.seedDialogDisplayed then
            MainMenu:ShowSeedDialog()
        elseif MainMenu.settingsScreenDisplayed then
            MainMenu:ShowSettingsScreen()
        end
    end
    HmGui.End()

    -- If player pressed the "new background" key and we're in startup mode, generate a new star system for a background
    if Input.GetPressed(Bindings.NewBackground) and MainMenu.currentMode == Enums.MenuMode.MainMenu then
        LTheoryRedux:seedStarsystem(Enums.MenuMode.MainMenu)
    end

    -- If player pressed the "toggle audio" key (currently F8), turn audio off if it's on or on if it's off
    -- NOTE: This is now disabled as we can use Settings to control Audio on/off, but I'm
    --       preserving it temporarily in case we want it back for some reason
    -- NOTE 2: This is currently the only place that calls LTheoryRedux:toggleSound(), so it might also be
    --         a candidate for deletion if we do decide to yank the key-based audio toggle
    --  if Input.GetPressed(Bindings.ToggleSound) then
    --    LTheoryRedux:toggleSound()
    --  end
end

function LTheoryRedux:generateNewSeed()
    self.seed = rng:get64()
end

function LTheoryRedux:generate()
    GameState:SetState(Enums.GameStates.Splashscreen) -- start off in Startup Mode

    -- Use random seed for new background star system, and stay in "display game logo" startup mode
    LTheoryRedux:seedStarsystem(Enums.MenuMode.Splashscreen)
end

function LTheoryRedux:seedStarsystem(menuMode)
    self.seed = rng:get64()

    LTheoryRedux:createStarSystem()

    MainMenu:SetMenuMode(menuMode)
end

function LTheoryRedux:createStarSystem()
    if self.backgroundSystem then self.backgroundSystem:delete() end

    print("------------------------")
    if GameState:GetCurrentState() == Enums.GameStates.MainMenu then
        -- Use custom system generation sizes for a nice background star system
        Config.gen.scaleSystem    = Config.gen.scaleSystemBack
        Config.gen.scalePlanet    = Config.gen.scalePlanetBack
        Config.gen.scalePlanetMod = Config.gen.scalePlanetModBack
        GameState.render.zNear    = Config.gen.zNearBack
        GameState.render.zFar     = Config.gen.zFarBack
    else
        -- Use the "real" system generation sizes for a gameplay star system
        Config.gen.scaleSystem    = Config.gen.scaleSystemReal
        Config.gen.scalePlanet    = Config.gen.scalePlanetReal
        Config.gen.scalePlanetMod = Config.gen.scalePlanetModReal
        GameState.render.zNear    = Config.gen.zNearReal
        GameState.render.zFar     = Config.gen.zFarReal
    end

    if GameState:GetCurrentState() == Enums.GameStates.MainMenu or GameState:GetCurrentState() == Enums.GameStates.Splashscreen then
        -- Spawn a new star system
        self.backgroundSystem = System(self.seed)
        GameState.world.currentSystem = self.backgroundSystem -- remember the player's current star system

        -- Background Mode
        -- Generate a new star system with nebulae/dust, a planet, an asteroid field,
        --   a space station, and an invisible rotating ship
        self.backgroundSystem:spawnBackground() -- spawn a ship that can't be seen

        -- Add a planet
        for i = 1, 1 do
            local planet = self.backgroundSystem:spawnPlanet(false) -- no planetary asteroid belt
            local ppos = planet:getPos()
            ppos.x = ppos.x * 2
            ppos.y = ppos.y * 2
            planet:setPos(ppos) -- move planet away from origin for background
        end

        -- Add an asteroid field
        -- NOTE: Must always add asteroid field (a zone) BEFORE space stations
        for i = 1, rng:getInt(0, 1) do                         -- 50/50 chance of having asteroids
            -- Spawn an asteroid field (a zone)
            self.backgroundSystem:spawnAsteroidField(-1, true) -- -1 parameter is a special case meaning background

            -- Add a space station
            self.backgroundSystem:spawnStation(Enums.StationHulls.Small, GameState.player.humanPlayer, nil)
        end
    else
        GameState:SetState(Enums.GameStates.InGame)
        Universe:CreateStarSystem(self.seed)
    end

    -- Insert the game view into the application canvas to make it visible
    self.gameView = Systems.Overlay.GameView(GameState.player.humanPlayer)
    GameState.render.gameView = self.gameView

    self.canvas = UI.Canvas()
    self.canvas
        :add(self.gameView
            :add(Systems.Controls.Controls.MasterControl(self.gameView, GameState.player.humanPlayer))
        )

    if GameState:GetCurrentState() == Enums.GameStates.InGame then
        -- TODO: replace with gamestate event system
        printf("LTheoryRedux: PlayAmbient")
        MusicPlayer:PlayAmbient()

        self.gameView:setCameraMode(GameState.player.startupCamera)
    else
        self.gameView:setCameraMode(Enums.CameraMode.FirstPerson)
    end
end

function LTheoryRedux:showGameLogo()
    -- Draw the LTR game logo on top of the background star system
    local scaleFactor = ((self.resX * self.resY) / (1600 * 900)) ^ 0.5
    local scaleFactorX = self.resX / 1600
    local scaleFactorY = self.resY / 900
    HmGui.Image(self.logo)                                                                  -- draw the LTR logo on top of the canvas
    HmGui.SetStretch(0.76 * scaleFactor / scaleFactorX, 0.243 * scaleFactor / scaleFactorY) -- scale logo (width, height)
    HmGui.SetAlign(0.5, 0.5)                                                                -- align logo
end

function LTheoryRedux:exitGame()
    -- Update Session vars ; temporary until we have a save state
    GameState.player.startupCamera = GameState.player.currentCamera
    -- Write player-specific game variables to preserve them across gameplay sessions
    InitFiles:writeUserInits()

    LTheoryRedux:quit()
end

--** SUPPORT FUNCTIONS **--
function LTheoryRedux:freezeTurrets()
    -- When taking down a dialog, Turret:updateTurret sees the button click input and thinks it means "Fire"
    -- So this routine adds a very brief cooldown to the player ship's turrets
    if GameState.player.currentShip then
        for turret in GameState.player.currentShip:iterSocketsByType(SocketType.Turret) do
            turret:addCooldown(2.0)
        end
        for bay in GameState.player.currentShip:iterSocketsByType(SocketType.Bay) do
            bay:addCooldown(2.0)
        end
    end
end

function LTheoryRedux:sleep(sec)
    os.execute(package.config:sub(1, 1) == "/" and "sleep " .. sec or "timeout " .. sec)
end

return LTheoryRedux
