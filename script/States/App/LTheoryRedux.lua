LimitTheoryRedux = require('States.Application')
local SoundManager = require('Systems.SFX.SoundManager')
local MusicPlayer = require('Systems.SFX.MusicPlayer')
local InitFiles = require('Systems.Files.InitFiles')
local UIRouter = require('UI.HmGui.UICore.UIRouter')
local UIPageMainMenu = require('script.UI.HmGui.Pages.MainMenu')
local UIPageLoadingScreen = require('UI.HmGui.Pages.LoadingScreen')
local UIPageGameplay = require('UI.HmGui.Pages.Gameplay')

local rng = RNG.FromTime()
local Universe = require('Systems.Universe.Universe')
local DebugControl = require('Systems.Controls.Controls.DebugControl')

---@diagnostic disable-next-line: duplicate-set-field
function LimitTheoryRedux:onInit()
    DebugControl.ltheory = self

    SoundManager:init()
    MusicPlayer:Init() --todo: fix all casing errors

    -- Read user-defined values and update game variables
    InitFiles:readUserInits()

    --* Game initializations *--
    WindowInstance:setSize(GameState.render.resX, GameState.render.resY)
    WindowInstance:setCenteredPosition()
    --self:setFullscreen(GameState.render.fullscreen)

    -- Set the default game control cursor
    -- TODO: WindowInstance:cursor().setIcon(Enums.CursorFilenames[GameState.ui.cursorStyle])
    WindowInstance:setCursorPosition(Vec2f(GameState.ui.cursorX, GameState.ui.cursorY))

    self:initMainMenu(true)
end

function LimitTheoryRedux:initMainMenu(isAppInit)
    GameState:SetState(Enums.GameStates.MainMenu)

    -- sizes for background star system
    Config.gen.scaleSystem    = Config.gen.scaleSystemBack
    Config.gen.scalePlanet    = Config.gen.scalePlanetBack
    Config.gen.scalePlanetMod = Config.gen.scalePlanetModBack
    GameState.render.zNear    = Config.gen.zNearBack
    GameState.render.zFar     = Config.gen.zFarBack

    Universe:init(rng:get64())
    Universe:addPlayer(GameState.player.humanPlayer)
    Universe:createStarSystem(false) -- create star system without economy

    self:initGameView()

    -- set initial view
    UIPageMainMenu:setView("Title")

    -- add pages
    if isAppInit then
        UIRouter:addPage(UIPageMainMenu)
        UIRouter:addPage(UIPageLoadingScreen)
        UIRouter:addPage(UIPageGameplay)
    end
    InputInstance:setCursorVisible(true)
    UIRouter:setCurrentPage("Main_Menu")
end

function LimitTheoryRedux:initGameView()
    -- Insert the game view into the application canvas to make it visible
    GameState.render.gameView = Systems.Overlay.GameView(GameState.player.humanPlayer, GameState.audio.manager)

    GameState.render.uiCanvas = UI.Canvas()
    GameState.render.uiCanvas
        :add(GameState.render.gameView
            :add(Systems.Controls.Controls.MasterControl(GameState.render.gameView, GameState.player.humanPlayer))
        )

    GameState.render.gameView:setCameraMode(Enums.CameraMode.FirstPerson)
end

function LimitTheoryRedux:soundOn()
    GameState.audio.soundEnabled = true
    --Log.Debug("LimitTheoryRedux:SoundOn: volume set to %s", GameState.audio.musicVolume)
    MusicPlayer:setVolume(MusicPlayer.lastVolume)
end

function LimitTheoryRedux:soundOff()
    GameState.audio.soundEnabled = false
    --Log.Debug("LimitTheoryRedux:SoundOff: volume set to 0")
    MusicPlayer:setVolume(0)
end

--* any operations we want to do before exiting the game
function LimitTheoryRedux:exit()
    -- Update Session vars ; temporary until we have a save state
    GameState.player.startupCamera = GameState.player.currentCamera
    -- Write player-specific game variables to preserve them across gameplay sessions
    InitFiles:writeUserInits()

    EngineInstance:exit()
end

return LimitTheoryRedux
