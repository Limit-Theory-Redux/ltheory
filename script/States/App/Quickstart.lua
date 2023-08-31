local DebugControl = require('Systems.Controls.Controls.DebugControl')
local InitFiles = require('Systems.Files.InitFiles')
local Universe = require('Systems.Universe.Universe')
local MusicPlayer = require('Systems.SFX.MusicPlayer')
local MainMenu = require('Systems.Menus.MainMenu')

local LTheoryRedux = require('States.App.LTheoryRedux')

-- Loads the user directly into spaceflight, skipping the main menu.
function LTheoryRedux:onInit()
    --* Value initializations *--
    self.logo = Tex2D.Load("./res/images/LTR_logo2.png") -- load the LTR logo

    DebugControl.ltheory = self

    --[[
    User-defined values are ignored to enable quicker debug changes,
    without impacting user settings for the main game.
    -- InitFiles:readUserInits()
    ]]

    GameState.ui.hudStyle = Enums.HudStyles.Wide
    GameState.ui.sensorsDisplayed = true
    GameState.ui.showTrackers = true
    GameState.audio.musicVolume = 0

    -- Initialize Universe
    Universe:Init()

    -- Open Main Menu
    MusicPlayer:Init()

    --* Game initializations *--
    self.window:setSize(GameState.render.resX, GameState.render.resY)
    Window.SetPosition(self.window, WindowPos.Centered, WindowPos.Centered)
    self:SetFullscreen(GameState.render.fullscreen)
    -- Set the default game control cursor
    self:setCursor(
        Enums.CursorFilenames[GameState.ui.cursorStyle],
        GameState.ui.cursorX,
        GameState.ui.cursorY)

    self.player = Entities.Player(GameState.player.humanPlayerName)
    GameState.player.humanPlayer = self.player

    -- Jump right to ingame
    MainMenu:SetMenuMode(Enums.MenuMode.Dialog)
    GameState:Unpause()
    GameState.player.currentControl = Enums.ControlModes.Ship
    Input:setCursorVisible(false)
    GameState:SetState(Enums.GameStates.InGame)
    self:seedStarsystem(Enums.MenuMode.Dialog)
end

return LTheoryRedux
