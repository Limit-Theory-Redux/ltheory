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
local SoundManager = require("Systems.SFX.SoundManager")
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
    self.logo            = Tex2D.Load("./res/images/LTR_logo2.png") -- load the full LTR logo
    self.logoname        = Tex2D.Load("./res/images/LTR-logo-name.png")
    self.logoicon        = Tex2D.Load("./res/images/LTR-logo-icon.png")

    DebugControl.ltheory = self

    SoundManager:init()

    -- Load Soundtracks before config
    MusicPlayer:Init()

    -- Read user-defined values and update game variables
    InitFiles:readUserInits()

    -- Initialize Universe
    Universe:Init()

    -- Open Main Menu
    MainMenu:Open()

    --* Game initializations *--
    WindowInstance:setSize(GameState.render.resX, GameState.render.resY)
    WindowInstance:setCenteredPosition()
    LTheoryRedux:SetFullscreen(GameState.render.fullscreen)

    -- Set the default game control cursor
    -- TODO: WindowInstance:cursor().setIcon(Enums.CursorFilenames[GameState.ui.cursorStyle])
    WindowInstance:setCursorPosition(Vec2f(GameState.ui.cursorX, GameState.ui.cursorY))

    self.player = Entities.Player(GameState.player.humanPlayerName)
    GameState.player.humanPlayer = self.player

    -- temporary
    -- TODO: allow player to join other factions if they want
    self.player:setFaction(Entities.Faction({
        name = GameState.player.playerFactionName,
        owner = self.player,
        type = Enums.FactionType.Player
    }))

    self:generate()
end

function LTheoryRedux:setCursor(cursorStyle, cursorX, cursorY)
    -- Set the game control cursor
    -- TODO: WindowInstance:cursor().setIcon(cursorStyle)

    if cursorX and cursorY then
        WindowInstance:setCursorPosition(Vec2f(cursorX, cursorY))
    end
end

function LTheoryRedux:toggleSound()
    if GameState.audio.soundEnabled then
        self:SoundOff()
    else
        self:SoundOn()
    end
end

function LTheoryRedux:SoundOn()
    GameState.audio.soundEnabled = true
    --Log.Debug("LTheoryRedux:SoundOn: volume set to %s", GameState.audio.musicVolume)
    MusicPlayer:SetVolume(MusicPlayer.lastVolume)
end

function LTheoryRedux:SoundOff()
    GameState.audio.soundEnabled = false
    --Log.Debug("LTheoryRedux:SoundOff: volume set to 0")
    MusicPlayer:SetVolume(0)
end

function LTheoryRedux:ToggleFullscreen()
    GameState.render.fullscreen = not GameState.render.fullscreen
    WindowInstance:setFullscreen(GameState.render.fullscreen)
end

function LTheoryRedux:SetFullscreen(fullscreen)
    GameState.render.fullscreen = fullscreen
    WindowInstance:setFullscreen(fullscreen)
end

function LTheoryRedux:onInput()
    self.canvas:input()

    if GameState:GetCurrentState() == Enums.GameStates.InGame and GameState.player.currentControl == Enums.ControlModes.Ship then
        if InputInstance:isPressed(Bindings.CameraFirstPerson) then
            if GameState.player.currentCamera ~= Enums.CameraMode.FirstPerson then
                self.gameView:setCameraMode(Enums.CameraMode.FirstPerson)
            end
        elseif InputInstance:isPressed(Bindings.CameraChase) then
            if GameState.player.currentCamera ~= Enums.CameraMode.Chase then
                self.gameView:setCameraMode(Enums.CameraMode.Chase)
            end
        elseif InputInstance:isPressed(Bindings.CameraOrbit) then
            -- if GameState.player.currentCamera ~= Enums.CameraMode.Orbit then
            --     self.gameView:setCameraMode(Enums.CameraMode.Orbit)
            -- end
        end
    elseif GameState:GetCurrentState() == Enums.GameStates.ShipCreation then
        --! i see all of this as a temporary feature addition - this should all be handled by a proper system later. ~ Jack
        if InputInstance:isPressed(Button.KeyboardB) then
            if GameState.player.currentShip then
                GameState.player.currentShip:delete()
            end

            local shipObject = {
                owner = GameState.player.humanPlayer,
                shipName = GameState.player.humanPlayerShipName,
                friction = 0,
                sleepThreshold = {
                    [1] = 0,
                    [2] = 0
                }
            }

            GameState.player.currentShip = Universe:CreateShip(GameState.world.currentSystem, nil, shipObject)
        end

        if InputInstance:isPressed(Button.KeyboardF) then
            -- Insert the game view into the application canvas to make it visible
            self.gameView = Systems.Overlay.GameView(self.player, self.audio)
            GameState.render.gameView = self.gameView

            self.canvas = UI.Canvas()
            self.canvas
                :add(self.gameView
                    :add(Systems.Controls.Controls.MasterControl(self.gameView, self.player))
                )
            self.gameView:setCameraMode(Enums.CameraMode.FirstPerson)

            GameState:SetState(Enums.GameStates.InGame)
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
            InputInstance:setCursorVisible(true)
            Log.Debug("Draw System View")
        end
    else
        if smap ~= nil then
            self.canvas:remove(smap)
            self.canvas:add(self.gameView)
            bSMapAdded = false
            smap = nil
            InputInstance:setCursorVisible(false)
            Log.Debug("Draw Game View")
        end
    end

    self.canvas:draw(self.resX, self.resY)

    Gui:draw() -- draw controls
end

function LTheoryRedux:onUpdate(dt)
    -- Routes
    GameState.player.humanPlayer:getRoot():update(dt)
    self.canvas:update(dt)
    MainMenu:OnUpdate(dt)
    SoundManager:clean(dt)
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
    if MainMenu.currentMode ~= Enums.MenuMode.Splashscreen and InputInstance:isPressed(Bindings.Escape) then
        MainMenu:SetBackgroundMode(false)
        InputInstance:setCursorVisible(true)
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
                InputInstance:setCursorVisible(MainMenu.dialogDisplayed)

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
    if InputInstance:isPressed(Bindings.SystemMap) and MainMenu.currentMode == Enums.MenuMode.Dialog then
        bShowSystemMap = not bShowSystemMap
        if smap == nil then
            smap = Systems.CommandView.SystemMap(GameState.world.currentSystem)
        end
    end

    -- If in flight mode, engage autopilot
    if InputInstance:isPressed(Bindings.AutoNav) and MainMenu.currentMode == Enums.MenuMode.Dialog then
        if playerShip ~= nil then
            local target = playerShip:getTarget()
            if target == nil then target = self.focus end
            if not playerShip:isDestroyed() and playerShip:isShipDocked() == nil and target ~= nil and target ~= playerShip then
                if playerShip:getCurrentAction() == nil or not string.find(playerShip:getCurrentAction():getName(), "MoveTo") then
                    -- Move undestroyed, undocked player ship to area of selected target
                    local autodistance = Config.game.autonavRanges[target:getType()]
                    GameState.player.autonavTimestamp = Config.getCurrentTimestamp()
                    GameState.player.playerMoving = true -- must be set to true before pushing the MoveTo action
                    playerShip:pushAction(Actions.MoveTo(target, autodistance, true))
                end
            end
        end
    end

    -- Disengage autopilot (require a 1-second delay, otherwise keypress turns autopilot on then off instantly)
    if GameState.player.playerMoving then
        if InputInstance:isPressed(Bindings.AutoNav) and Config.getCurrentTimestamp() - GameState.player.autonavTimestamp > 1 then
            GameState.player.playerMoving = false
        end
    end

    -- If player pressed the "ToggleLights" key in Flight Mode, toggle dynamic lighting on/off
    -- NOTE: Performance is OK for just the player's ship, but adding many lit ships & pulses tanks performance
    if InputInstance:isPressed(Bindings.ToggleLights) and MainMenu.currentMode == Enums.MenuMode.Dialog then
        GameState.render.thrusterLights = not GameState.render.thrusterLights
        GameState.render.pulseLights    = not GameState.render.pulseLights
    end

    -- Decide which game controls screens (if any) to display on top of the canvas
    Gui:beginGui(self.resX, self.resY, InputInstance)

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

    --! temp hacking this in here
    if GameState:GetCurrentState() == Enums.GameStates.ShipCreation then
        LTheoryRedux:showShipCreationHint()
    end
    Gui:endGui(InputInstance)

    -- If player pressed the "new background" key and we're in startup mode, generate a new star system for a background
    if InputInstance:isPressed(Bindings.NewBackground) and MainMenu.currentMode == Enums.MenuMode.MainMenu then
        LTheoryRedux:seedStarsystem(Enums.MenuMode.MainMenu)
    end

    -- If player pressed the "toggle audio" key (currently F8), turn audio off if it's on or on if it's off
    -- NOTE: This is now disabled as we can use Settings to control Audio on/off, but I'm
    -- preserving it temporarily in case we want it back for some reason
    -- NOTE 2: This is currently the only place that calls LTheoryRedux:toggleSound(), so it might also be
    -- a candidate for deletion if we do decide to yank the key-based audio toggle
    -- if InputInstance:isPressed(Bindings.ToggleSound) then
    -- LTheoryRedux:toggleSound()
    -- end
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

    Log.Debug("------------------------")
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
        -- a space station, and an invisible rotating ship
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
        -- Quickstart if forced to ingame
        if GameState:GetCurrentState() ~= Enums.GameStates.Quickstart then
            GameState:SetState(Enums.GameStates.ShipCreation)
        end
        Universe:CreateStarSystem(self.seed)
    end

    if GameState:GetCurrentState() == Enums.GameStates.ShipCreation then
        -- TODO: replace with gamestate event system
        Log.Debug("LTheoryRedux: PlayAmbient")
        MusicPlayer:PlayAmbient()

        DebugControl.ltheory = self
        self.gameView = Systems.Overlay.GameView(GameState.player.humanPlayer, self.audio)
        GameState.render.gameView = self.gameView
        self.canvas = UI.Canvas()
        self.canvas
            :add(self.gameView
                :add(Systems.Controls.Controls.GenTestControl(self.gameView, GameState.player.humanPlayer)))

        InputInstance:setCursorVisible(true)
    else
        -- Insert the game view into the application canvas to make it visible
        self.gameView = Systems.Overlay.GameView(GameState.player.humanPlayer, self.audio)
        GameState.render.gameView = self.gameView

        self.canvas = UI.Canvas()
        self.canvas
            :add(self.gameView
                :add(Systems.Controls.Controls.MasterControl(self.gameView, GameState.player.humanPlayer))
            )
        self.gameView:setCameraMode(Enums.CameraMode.FirstPerson)
    end
end

function LTheoryRedux:showGameLogo()
    -- Draw the LTR game logo on top of the background star system
    local scaleFactor = ((self.resX * self.resY) / (1600 * 900)) ^ 0.5
    local scaleFactorX = self.resX / 1600
    local scaleFactorY = self.resY / 900

    Gui:image(self.logo) -- draw the LTR logo on top of the canvas
    Gui:setPercentSize(76.0 * scaleFactor / scaleFactorX, 24.3 * scaleFactor / scaleFactorY)
    Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
end

function LTheoryRedux:showShipCreationHint()
    Gui:beginStackContainer()
    Gui:setFixedHeight(100)
    Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Bottom)
    Gui:setChildrenVerticalAlignment(AlignVertical.Center)
    Gui:textEx(Cache.Font('Exo2', 32), '[B]: Random Ship | [F]: Spawn', Color(1.0, 1.0, 1.0, 1.0))
    Gui:endContainer()
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
