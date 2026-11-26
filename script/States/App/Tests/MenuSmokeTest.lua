--[[
Menu smoke test: boots the real game (LTheoryRedux) and opens every menu
screen for a few frames: the main menu screens with each Settings tab, then a
new game with its HUD, the pause menu, the Controls screen and the game over
screen (the player's ship at no health). It catches a screen that throws while
it is built or drawn, like a setting that no longer exists.

Then every scenario in New Game's Scenario list (Config/Scenarios) is started
from the main menu and regenerated once (B), so a scenario that throws while
it is built, run, drawn or rebuilt fails the test too. The first one starts
from the game over screen, through the main menu as its button does.

Run it from the repository root after a build:

    ./bin/ltr.exe MenuSmokeTest      (macOS/Linux: ./bin/ltr MenuSmokeTest)

Exit code 0 and "MenuSmokeTest: passed" = every screen built and drew without
a script error. A screen that throws stops the game with its error and
traceback and exit code 1 (`Core.Call`); the last "MenuSmokeTest: checking"
line names the screen.

The test uses a settings folder of its own (`MenuSmokeTest`, next to the
game's), emptied at start: it runs on the default settings and never reads or
writes the player's user.ini (leaving the Settings screen saves it).

It needs a GPU and a window, so CI does not run it.

Not covered: the Ship_Creation and System_Map views of the Gameplay page, which
still use Legacy objects and are not wired into the new game yet.
]]

local Game = require('States.App.LTheoryRedux')
local UIPageMainMenu = require('UI.HmGui.Pages.MainMenu')
local SettingsView = require('UI.HmGui.Views.MainMenu.Settings')
local Scenarios = require('Config.Scenarios')
local CoreComponents = require('Modules.Core.Components')

local FRAMES_PER_SCREEN = 5
local GAME_SEED = 1

-- A settings folder of its own, emptied: default settings, and the player's
-- user.ini stays untouched.
Config.paths.files = Directory.GetPrefPath(Config.org, 'MenuSmokeTest')
os.remove(Config.paths.files .. Config.userInitFilename .. Config.userInitFiletype)
os.remove(Config.paths.files .. Config.userInitFilename .. '_backup' .. Config.userInitFiletype)

--- The screens in the order they are opened: { name, open = function }.
local screens = {}

local function screen(name, open, frames)
    screens[#screens + 1] = { name = name, open = open, frames = frames or FRAMES_PER_SCREEN }
end

for _, view in ipairs({ 'Title', 'Main', 'Newgame', 'Loadgame' }) do
    screen('Main menu > ' .. view, function() UIPageMainMenu:setView(view) end)
end

for tab = 1, SettingsView:getCategoryCount() do
    screen('Main menu > Settings > ' .. SettingsView:getCategoryName(tab), function()
        SettingsView:setCategory(tab)
        if UIPageMainMenu:getCurrentView() ~= SettingsView then
            UIPageMainMenu:setView('Settings')
        end
    end)
end

-- Leaving Settings for Credits also saves the settings (into the test's folder).
for _, view in ipairs({ 'Credits', 'Background' }) do
    screen('Main menu > ' .. view, function() UIPageMainMenu:setView(view) end)
end

screen('Game > HUD', function() Game:startGame(GAME_SEED) end)
screen('Game > Pause menu', function() Game:openPauseMenu() end)
screen('Game > Controls', function() Game:openControls() end)
screen('Game > Game over', function()
    Game:closePauseMenu()
    Game.playerShip:get(CoreComponents.Health):setCurrentHealth(0)
end)

-- From the main menu, as a player would (a frame or more apart: tearing a
-- big star system down and building the next one in one frame needs GPU
-- memory for both). Regenerate gets more frames: it builds a few frames after
-- tearing down.
for _, scenario in ipairs(Scenarios) do
    screen('Scenario > ' .. scenario.name .. ' > Main menu', function() Game:returnToMainMenu() end)
    screen('Scenario > ' .. scenario.name, function() Game:startGame(GAME_SEED, scenario) end)
    screen('Scenario > ' .. scenario.name .. ' > Regenerate', function() Game:regenerateScenario() end, 12)
end

local current = 0
local framesOnScreen = 0

--- Called once per frame: after `FRAMES_PER_SCREEN` frames on a screen, open
--- the next one; after the last one, report the pass and quit.
local function advance()
    framesOnScreen = framesOnScreen + 1
    if current > #screens then
        return -- passed; the engine stops after this frame
    end
    if current > 0 and framesOnScreen < screens[current].frames then
        return
    end

    current = current + 1
    framesOnScreen = 0

    local nextScreen = screens[current]
    if not nextScreen then
        Log.Info('MenuSmokeTest: passed, %d screens', #screens)
        Engine:exit()
        return
    end

    Log.Info('MenuSmokeTest: checking %s (%d/%d)', nextScreen.name, current, #screens)
    nextScreen.open()
end

local gameOnInit = Game.onInit
function Game:onInit()
    gameOnInit(self)

    -- A window instead of the game's borderless fullscreen, and no music.
    GameState.render.fullscreen = false
    Window:setFullscreen(false, false)
    self:soundOff()
end

-- Wrapped before `appInit` subscribes `onPreRender` to the frame event.
local gameOnPreRender = Game.onPreRender
function Game:onPreRender(data)
    gameOnPreRender(self, data)
    advance()
end

return Game
