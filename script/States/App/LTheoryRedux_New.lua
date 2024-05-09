local LimitTheoryRedux = require('States.Application')
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

    GameState.player.humanPlayer = Entities.Player(GameState.player.humanPlayerName)

    Universe:init(rng:get64())

    -- sizes for background star system
    Config.gen.scaleSystem    = Config.gen.scaleSystemBack
    Config.gen.scalePlanet    = Config.gen.scalePlanetBack
    Config.gen.scalePlanetMod = Config.gen.scalePlanetModBack
    GameState.render.zNear    = Config.gen.zNearBack
    GameState.render.zFar     = Config.gen.zFarBack

    Universe:createStarSystem(false)

    self:initGameView()

    -- set initial view
    UIPageMainMenu:setView("Title")

    -- add pages
    UIRouter:addPage(UIPageMainMenu)
    UIRouter:addPage(UIPageLoadingScreen)
    UIRouter:addPage(UIPageGameplay)
    UIRouter:setCurrentPage("Main_Menu")
end

---@param dt integer
---@diagnostic disable-next-line: duplicate-set-field
function LimitTheoryRedux:onInput()
    GameState.render.uiCanvas:input()
    UIRouter:input()
end

---@param dt integer
---@diagnostic disable-next-line: duplicate-set-field
function LimitTheoryRedux:onUpdate(dt)
    GameState.player.humanPlayer:getRoot():update(dt)
    Universe:onUpdate(dt)
    SoundManager:clean(dt)
    MusicPlayer:OnUpdate(dt) --todo fix casing

    GameState.render.uiCanvas:update(dt)
    Gui:beginGui(self.resX, self.resY, InputInstance)
    UIRouter:update(dt)
    Gui:endGui(InputInstance)
end

---@diagnostic disable-next-line: duplicate-set-field
function LimitTheoryRedux:onDraw()
    GameState.render.uiCanvas:draw(self.resX, self.resY)
    Gui:draw()
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

return LimitTheoryRedux
