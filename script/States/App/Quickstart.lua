local DebugControl = require('Systems.Controls.Controls.DebugControl')
local InitFiles = require('Systems.Files.InitFiles')
local Universe = require('Systems.Universe.Universe')
local MusicPlayer = require('Systems.SFX.MusicPlayer')
local MainMenu = require('Systems.Menus.MainMenu')

local LTheoryRedux = require('States.App.LTheoryRedux')

function LTheoryRedux:onInit()
    --* Value initializations *--
    self.logo = Tex2D.Load("./res/images/LTR_logo2.png") -- load the LTR logo

    DebugControl.ltheory = self

    -- Read user-defined values and update game variables
--    InitFiles:readUserInits()

    --* Audio initializations *--
    Audio.Init()
    Audio.Set3DSettings(0.0, 10, 2);

    if Config.audio.pulseFire then Sound.SetVolume(Config.audio.pulseFire, Config.audio.soundMax) end

    -- Initialize Universe
    Universe:Init()

    -- Open Main Menu
    MusicPlayer:Init()

    --* Game initializations *--
    self.window:setSize(GameState.render.resX, GameState.render.resY)
    Window.SetPosition(self.window, WindowPos.Centered, WindowPos.Centered)
    self:SetFullscreen(GameState.render.fullscreen)

    GameState.ui.hudStyle = Enums.HudStyles.Wide
    GameState.ui.sensorsDisplayed = true
    GameState.ui.showTrackers = true
    GameState.audio.musicVolume = 0

    -- Set the default game control cursor
    self:setCursor(Enums.CursorFilenames[GameState.ui.cursorStyle], GameState.ui.cursorX, GameState.ui.cursorY)

    self.player = Entities.Player(GameState.player.humanPlayerName)
    GameState.player.humanPlayer = self.player

    -- Jump right to ingame
    MainMenu:SetMenuMode(Enums.MenuMode.Dialog)
    GameState:Unpause()
    GameState.player.currentControl = Enums.ControlModes.Ship
    Input.SetMouseVisible(false)
    GameState:SetState(Enums.GameStates.InGame)
    self:seedStarsystem(Enums.MenuMode.Dialog)
end

return LTheoryRedux
