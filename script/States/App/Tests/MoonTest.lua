-- MoonTest: a shortcut into the game's Moon Showcase scenario
-- (script/Config/Scenarios/MoonShowcase.lua): a generated moon next to your ship. B makes a new one.
--
-- Starts the game (LTheoryRedux) straight into the scenario, skipping the main
-- menu; after that it is the normal game (C cycles the cameras, Escape opens
-- the pause menu, Back to Main Menu leads to the menu).
--
-- Run: ./bin/ltr.exe MoonTest

local Game = require('States.App.LTheoryRedux')

Game.autostartScenario = require('Config.Scenarios').find('MoonShowcase')
Game.autostartSeed = 1 -- the same scene every run (B for another)

return Game
